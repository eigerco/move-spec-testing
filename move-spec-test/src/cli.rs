// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_mutator::cli::{FunctionFilter, ModuleFilter, PackagePathCheck};
use std::path::PathBuf;

/// Command line options for specification test tool.
#[derive(Parser, Default, Debug, Clone)]
pub struct CLIOptions {
    /// The paths to the Move sources.
    #[clap(long, value_parser)]
    pub move_sources: Vec<PathBuf>,

    /// Work only over specified modules.
    #[clap(
        long,
        value_parser,
        default_value = "all",
        conflicts_with = "use_generated_mutants"
    )]
    pub mutate_modules: ModuleFilter,

    /// Work only over specified functions (these are not qualifed functions).
    #[clap(
        long,
        value_parser,
        default_value = "all",
        conflicts_with = "use_generated_mutants"
    )]
    pub mutate_functions: FunctionFilter,

    /// Optional configuration file for mutator tool.
    #[clap(long, value_parser, conflicts_with = "use_generated_mutants")]
    pub mutator_conf: Option<PathBuf>,

    /// Optional configuration file for prover tool.
    #[clap(long, value_parser)]
    pub prover_conf: Option<PathBuf>,

    /// Save report to a JSON file.
    #[clap(long, value_parser)]
    pub output: Option<PathBuf>,

    /// Use previously generated mutants.
    #[clap(long, value_parser)]
    pub use_generated_mutants: Option<PathBuf>,

    /// Indicates if mutants should be verified and made sure mutants can compile.
    #[clap(
        long,
        default_value = "false",
        conflicts_with = "use_generated_mutants"
    )]
    pub verify_mutants: bool,

    /// Extra arguments to pass to the prover.
    #[clap(long, value_parser)]
    pub extra_prover_args: Option<Vec<String>>,

    /// Remove averagely given percentage of mutants. See the doc for more details.
    #[clap(long, conflicts_with = "use_generated_mutants")]
    pub downsampling_ratio_percentage: Option<usize>,
}

impl<'a> PackagePathCheck<'a> for CLIOptions {
    fn get_move_sources(&'a self) -> &'a Vec<PathBuf> {
        &self.move_sources
    }
}

/// This function creates a mutator CLI options from the given spec-test options.
#[must_use]
pub fn create_mutator_options(options: &CLIOptions) -> move_mutator::cli::CLIOptions {
    move_mutator::cli::CLIOptions {
        move_sources: options.move_sources.clone(),
        mutate_modules: options.mutate_modules.clone(),
        mutate_functions: options.mutate_functions.clone(),
        verify_mutants: options.verify_mutants,
        downsampling_ratio_percentage: options.downsampling_ratio_percentage,
        ..Default::default()
    }
}

/// This function generates a prover CLI options from the given spec-test options.
///
/// # Errors
/// Errors are returned as `anyhow::Result`.
pub fn generate_prover_options(options: &CLIOptions) -> anyhow::Result<move_prover::cli::Options> {
    let prover_conf = if let Some(conf) = &options.prover_conf {
        move_prover::cli::Options::create_from_toml_file(conf.to_str().unwrap_or(""))?
    } else if let Some(args) = &options.extra_prover_args {
        move_prover::cli::Options::create_from_args(args)?
    } else {
        move_prover::cli::Options::default()
    };

    Ok(prover_conf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn cli_options_starts_empty() {
        let options = CLIOptions::default();
        assert!(options.move_sources.is_empty());
        assert_eq!(ModuleFilter::All, options.mutate_modules);
        assert_eq!(FunctionFilter::All, options.mutate_functions);
        assert!(options.mutator_conf.is_none());
        assert!(options.prover_conf.is_none());
        assert!(options.output.is_none());
        assert!(options.extra_prover_args.is_none());
    }

    #[test]
    fn create_mutator_options_copies_fields() {
        let mut options = CLIOptions::default();
        options.move_sources.push(PathBuf::from("path/to/file"));
        options.mutate_modules =
            ModuleFilter::Selected(vec!["mod1".to_string(), "mod2".to_string()]);
        options.mutate_functions =
            FunctionFilter::Selected(vec!["func1".to_string(), "func2".to_string()]);
        options.mutator_conf = Some(PathBuf::from("path/to/mutator/conf"));

        let mutator_options = create_mutator_options(&options);

        assert_eq!(mutator_options.move_sources, options.move_sources);
        assert_eq!(mutator_options.mutate_modules, options.mutate_modules);
        assert_eq!(mutator_options.mutate_functions, options.mutate_functions);
    }

    #[test]
    fn generate_prover_options_creates_from_conf_when_conf_exists() {
        let toml_content = r#"
            [backend]
            boogie_exe = "/path/to/boogie"
            z3_exe = "/path/to/z3"
        "#;

        fs::write("test_prover_conf.toml", toml_content).unwrap();

        let options = CLIOptions {
            prover_conf: Some(PathBuf::from("test_prover_conf.toml")),
            ..Default::default()
        };

        let prover_options = generate_prover_options(&options).unwrap();
        fs::remove_file("test_prover_conf.toml").unwrap();

        assert_eq!(
            prover_options.backend.boogie_exe,
            "/path/to/boogie".to_owned()
        );
        assert_eq!(prover_options.backend.z3_exe, "/path/to/z3".to_owned());
    }
}
