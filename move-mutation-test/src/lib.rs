// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

mod benchmark;
pub mod cli;
mod mutation_test;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::{
    benchmark::{Benchmark, Benchmarks},
    mutation_test::run_tests,
};
use anyhow::anyhow;
use cli::TestBuildConfig;
use move_package::{source_package::layout::SourcePackageLayout, BuildConfig};
use mutator_common::report::{MiniReport, MutantStatus, Report};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// This function runs the mutation testing, which is a combination of the mutator tool and the test tool.
///
/// It takes the CLI options and constructs appropriate options for the
/// Move Mutator tool and Move Mutation Test tool. Then it mutates the code storing
/// results in a temporary directory. Then it runs tests on the mutated
/// code and stores the results, using them to generate the report at the end.
///
/// # Arguments
///
/// * `options` - A `cli::Options` representing the options for the mutation test tool.
/// * `test_config` - A `TestBuildConfig` representing the test configuration.
///
/// # Errors
///
/// Errors are returned as `anyhow::Result`.
///
/// # Returns
///
/// * `anyhow::Result<()>` - The result of the mutation test.
pub fn run_mutation_test(
    options: &cli::CLIOptions,
    test_config: &TestBuildConfig,
) -> anyhow::Result<()> {
    // We need to initialize logger using try_init() as it might be already initialized in some other tool
    // (e.g. move-mutator). If we use init() instead, we will get an abort.
    let _ = pretty_env_logger::try_init();

    let mut error_writer = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);

    // Check if package is correctly structured.
    let package_path = SourcePackageLayout::try_find_root(
        &test_config.move_pkg.get_package_path()?.canonicalize()?,
    )?;

    info!("Found package path: {package_path:?}");
    info!("Running tool the following options: {options:?} and test config: {test_config:?}");

    // Always create and use benchmarks.
    // Benchmarks call only time getting functions, so it's safe to use them in any case and
    // they are not expensive to create (won't hit the performance).
    let mut benchmarks = Benchmarks::new();
    benchmarks.total_duration.start();

    // Run original tests to ensure the original tests are working:

    // We need to check for the latest git deps only for the first time we run the test.
    // All subsequent runs with this tool will then have the latest deps fetched.
    let skip_fetch_deps = false;
    // TODO: use this one instead once it is available in the aptos-core.
    //let skip_fetch_deps = test_config.move_pkg.skip_fetch_latest_git_deps; // false by default
    let result = run_tests(
        test_config,
        &package_path,
        skip_fetch_deps,
        &mut error_writer,
    );

    if let Err(e) = result {
        let msg =
            format!("Test suit is failing for the original code! Unit test failed with error: {e}");
        error!("{msg}");
        return Err(anyhow!(msg));
    }

    // Create mutants:

    // Setup temporary directory structure.
    let outdir = tempfile::tempdir()?.into_path();
    let outdir_original = outdir.join("base");

    fs::create_dir_all(outdir_original)?;

    let outdir_mutant = if let Some(mutant_path) = &options.use_generated_mutants {
        mutant_path.clone()
    } else {
        benchmarks.mutator.start();
        let mutator_config = BuildConfig {
            dev_mode: test_config.move_pkg.dev,
            additional_named_addresses: test_config.move_pkg.named_addresses(),
            // No need to fetch latest deps again.
            skip_fetch_latest_git_deps: true,
            compiler_config: test_config.compiler_config(),
            ..Default::default()
        };
        let outdir_mutant = run_mutator(
            options,
            test_config.apply_coverage,
            &mutator_config,
            &package_path,
            &outdir,
        )?;
        benchmarks.mutator.stop();
        outdir_mutant
    };

    let report =
        move_mutator::report::Report::load_from_json_file(&outdir_mutant.join("report.json"))?;

    // Run tests on mutants:

    // Do not calculate the coverage on mutants.
    let test_config = test_config.disable_coverage();

    benchmarks.mutation_test.start();
    let (mutation_test_benchmarks, mini_reports): (Vec<Benchmark>, Vec<MiniReport>) = report
        .get_mutants()
        .par_iter()
        .map(|elem| {
            let mut benchmark = Benchmark::new();

            let mutant_file = elem.mutant_path();
            let rayon_thread_id =
                rayon::current_thread_index().expect("failed to fetch rayon thread id");
            info!(
                "job_{rayon_thread_id}: Running tests for mutant {}",
                mutant_file.display()
            );

            // Strip prefix to get the path relative to the package directory (or take that path if it's already relative).
            let original_file = elem
                .original_file_path()
                .strip_prefix(&package_path)
                .unwrap_or(elem.original_file_path());
            let job_work_dir = format!("mutation_test_{rayon_thread_id}");
            let outdir = outdir.join(job_work_dir);

            let _ = fs::remove_dir_all(&outdir);
            move_mutator::compiler::copy_dir_all(&package_path, &outdir)
                .expect("copying directory failed");

            trace!(
                "Copying mutant file {:?} to the package directory {:?}",
                mutant_file,
                outdir.join(original_file)
            );

            // Should never fail, since files will always exists.
            let _ = fs::copy(mutant_file, outdir.join(original_file));

            if let Err(e) =
                move_mutator::compiler::rewrite_manifest_for_mutant(&package_path, &outdir)
            {
                panic!("rewriting manifest for mutant failed: {e}");
            }

            benchmark.start();

            // No need to fetch latest deps again.
            let skip_fetch_deps = true;
            // No need to print anything to the screen, due to many threads, it might be messy and slow.
            let mut error_writer = std::io::sink();
            let result = run_tests(&test_config, &outdir, skip_fetch_deps, &mut error_writer);
            benchmark.stop();

            let mutant_status = if let Err(e) = result {
                trace!("Mutant killed! Unit test failed with error: {e}");
                MutantStatus::Killed
            } else {
                trace!("Mutant {} hasn't been killed!", mutant_file.display());
                MutantStatus::Alive
            };

            let diff = elem.get_diff().to_owned();

            // Qualified name for the function.
            let mut qname = elem.get_module_name().to_owned();
            qname.push_str("::");
            qname.push_str(elem.get_function_name());

            (
                benchmark,
                MiniReport::new(original_file.to_path_buf(), qname, mutant_status, diff),
            )
        })
        .collect::<Vec<(_, _)>>()
        .into_iter()
        .unzip();

    benchmarks.mutation_test.stop();
    benchmarks.mutation_test_results = mutation_test_benchmarks;

    // Prepare a report.
    let mut test_report = Report::new(package_path.to_owned());
    for MiniReport {
        original_file,
        qname,
        mutant_status,
        diff,
    } in mini_reports
    {
        test_report.increment_mutants_tested(&original_file, &qname);
        if let MutantStatus::Alive = mutant_status {
            test_report.add_mutants_alive_diff(&original_file, &qname, &diff);
        } else {
            test_report.increment_mutants_killed(&original_file, &qname);
            test_report.add_mutants_killed_diff(&original_file, &qname, &diff);
        }
    }

    if let Some(outfile) = &options.output {
        test_report.save_to_json_file(outfile)?;
    }
    println!("\nTotal mutants tested: {}", test_report.mutants_tested());
    println!("Total mutants killed: {}\n", test_report.mutants_killed());
    test_report.print_table();

    benchmarks.total_duration.stop();
    benchmarks.display();

    Ok(())
}

/// This function runs the Move Mutator tool.
fn run_mutator(
    options: &cli::CLIOptions,
    apply_coverage: bool,
    config: &BuildConfig,
    package_path: &Path,
    outdir: &Path,
) -> anyhow::Result<PathBuf> {
    debug!("Running the move mutator tool");
    let mut mutator_conf = cli::create_mutator_options(options, apply_coverage);

    let outdir_mutant = if let Some(path) = cli::check_mutator_output_path(&mutator_conf) {
        path
    } else {
        mutator_conf.out_mutant_dir = Some(outdir.join("mutants"));
        mutator_conf.out_mutant_dir.clone().unwrap()
    };

    fs::create_dir_all(&outdir_mutant)?;
    move_mutator::run_move_mutator(mutator_conf, config, package_path)?;

    Ok(outdir_mutant)
}
