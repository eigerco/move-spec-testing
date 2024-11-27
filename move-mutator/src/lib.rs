// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod cli;
pub mod compiler;

mod mutate;

pub mod configuration;
pub(crate) mod coverage;
mod mutant;
mod operator;
mod operators;
mod output;
pub mod report;

use crate::{
    compiler::{generate_ast, verify_mutant},
    configuration::Configuration,
    report::{MutationReport, Report},
};
use move_package::BuildConfig;
use mutator_common::tmp_package_dir::setup_outdir_and_package_path;
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Runs the Move mutator tool.
/// Entry point for the Move mutator tool both for the CLI and the Rust API.
///
/// # Arguments
///
/// * `options` - Command line options passed to the Move mutator tool.
/// * `config` - The build configuration for the Move package.
/// * `package_path` - The path to the Move package.
///
/// # Errors
/// Any error that occurs during the mutation process will be returned as an `anyhow::Error` with a description of the error.
///
/// # Panics
///
/// The function will panic if `downsampling_ratio_percentage` is not in the range 0..=100.
///
/// # Returns
///
/// * `anyhow::Result<()>` - Returns `Ok(())` if the mutation process completes successfully, or an error if any error occurs.
pub fn run_move_mutator(
    options: cli::CLIOptions,
    config: &BuildConfig,
    package_path: &Path,
) -> anyhow::Result<()> {
    // We need to initialize logger using try_init() as it might be already initialized in some other tool
    // (e.g. spec-test). If we use init() instead, we will get an abort.
    let _ = pretty_env_logger::try_init();

    info!(
        "Executed move-mutator with the following options: {options:?} \n config: {config:?} \n package path: {package_path:?}"
    );

    // Setup output dir and clone package path there.
    let original_package_path = package_path.canonicalize()?;
    let (_, package_path) = if options.move_sources.is_empty() {
        setup_outdir_and_package_path(&original_package_path)?
    } else {
        (PathBuf::new(), package_path.to_owned())
    };

    let mut mutator_configuration =
        Configuration::new(options, Some(original_package_path.to_owned()));

    trace!("Mutator configuration: {mutator_configuration:?}");

    let package_path = mutator_configuration
        .project_path
        .clone()
        .unwrap_or(package_path.to_owned());
    let env = generate_ast(&mutator_configuration, config, &package_path)?;

    info!("Generated AST");

    // For the next compilation steps, we don't need to fetch git deps again.
    let mut config = config.clone();
    config.skip_fetch_latest_git_deps = true;
    config.compiler_config.skip_attribute_checks = true;

    if mutator_configuration.project.apply_coverage {
        // This implies additional compilation inside.
        mutator_configuration
            .coverage
            .compute_coverage(&config, &package_path)?;
    }

    let mutants = mutate::mutate(&env, &mutator_configuration)?;
    let output_dir = output::setup_output_dir(&mutator_configuration)?;

    // Generate mutants and extract all info needed for rayon threads below.
    let mut transformed_mutants: Vec<_> = mutants
        .into_iter()
        .flat_map(|mutant| {
            let file_id = mutant.get_file_id();
            let original_source = env.get_file_source(file_id);
            let filename = env.get_file(file_id);
            let path = Path::new(filename)
                .canonicalize()
                .expect("canonicalizing failed");
            let fn_name = mutant.get_function_name().unwrap_or_default();
            let mod_name = mutant.get_module_name().unwrap_or("script".to_owned());

            mutant
                .apply(original_source)
                .into_iter()
                .map(|mutant_info| {
                    (
                        mutant_info,
                        fn_name.clone(),
                        mod_name.clone(),
                        path.clone(),
                        original_source,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // If the downsample ratio is set, we need to downsample the mutants.
    if let Some(percentage) = mutator_configuration.project.downsampling_ratio_percentage {
        let total_mutants = transformed_mutants.len();

        let no_of_mutants_to_keep =
            total_mutants.saturating_sub((total_mutants * percentage).div_ceil(100));
        assert!(
            no_of_mutants_to_keep <= total_mutants,
            "Invalid downsampling ratio"
        );

        // Delete randomly elements from the vector.
        let mut rng = thread_rng();
        transformed_mutants = transformed_mutants
            .choose_multiple(&mut rng, no_of_mutants_to_keep)
            .cloned()
            .collect();
    }

    let mutation_reports: Vec<MutationReport> = transformed_mutants
        .into_par_iter()
        .map(|(mutated_info, function, module, path, original_source)| {
            // An informative description for the mutant.
            let mutant = format!("{module}::{function}: {:?}", mutated_info.mutation);

            // In case the number of mutants is very low, a single thread might be used.
            let rayon_tid = rayon::current_thread_index().unwrap_or(0);
            info!("job_{rayon_tid}: Checking mutant {mutant}");

            if mutator_configuration.project.verify_mutants {
                let res = verify_mutant(&config, &mutated_info.mutated_source, &path);

                // In case the mutant is not a valid Move file, skip the mutant (do not save it).
                if let Err(e) = res {
                    info!("job_{rayon_tid}: Mutant {mutant} is invalid and will not be generated: {e:?}");
                    return None;
                }
            }

            let mutant_id = mutated_info.unique_id();
            let Ok(mutant_path) = output::setup_mutant_path(&output_dir, &path, mutant_id) else {
                // If we cannot set up the mutant path, we skip the mutant.
                trace!("Cannot set up mutant path for {path:?}");
                return None;
            };

            // Should never fail.
            fs::write(&mutant_path, &mutated_info.mutated_source)
                .expect("failed to write mutant to a file");

            info!(
                "job_{rayon_tid}: {mutant} written to {}",
                mutant_path.display()
            );
            let mut entry = report::MutationReport::new(
                mutant_path.as_path(),
                &path,
                &module,
                &function,
                &mutated_info.mutated_source,
                original_source,
            );

            entry.add_modification(mutated_info.mutation);
            Some(entry)
        })
        .flatten()
        .collect();

    let mut report: Report = Report::new();
    for entry in mutation_reports {
        report.add_entry(entry);
    }

    trace!("Saving reports to: {output_dir:?}");
    report.save_to_json_file(output_dir.join(Path::new("report.json")).as_path())?;
    report.save_to_text_file(output_dir.join(Path::new("report.txt")).as_path())?;

    info!("Mutator generation is completed");
    Ok(())
}
