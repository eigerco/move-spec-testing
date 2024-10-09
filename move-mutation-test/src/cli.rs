// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos::{common::types::MovePackageDir, move_tool::experiments_from_opt_level};
use aptos_framework::extended_checks;
use clap::Parser;
use move_model::metadata::LanguageVersion;
use move_mutator::cli::{FunctionFilter, ModuleFilter};
use move_package::CompilerConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Command line options for mutation test tool.
#[derive(Parser, Default, Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CLIOptions {
    /// Work only over specified modules.
    #[clap(long, value_parser, default_value = "all")]
    pub mutate_modules: ModuleFilter,

    /// Work only over specified functions (these are not qualifed functions).
    #[clap(short = 'f', long, value_parser, default_value = "all")]
    pub mutate_functions: FunctionFilter,

    /// Optional configuration file for mutator tool.
    #[clap(long, value_parser)]
    pub mutator_conf: Option<PathBuf>,

    /// Save report to a JSON file.
    #[clap(short, long, value_parser)]
    pub output: Option<PathBuf>,

    /// Use previously generated mutants.
    #[clap(long, short, value_parser)]
    pub use_generated_mutants: Option<PathBuf>,
}

/// This function creates a mutator CLI options from the given mutation-test options.
#[must_use]
pub fn create_mutator_options(
    options: &CLIOptions,
    apply_coverage: bool,
) -> move_mutator::cli::CLIOptions {
    move_mutator::cli::CLIOptions {
        mutate_functions: options.mutate_functions.clone(),
        mutate_modules: options.mutate_modules.clone(),
        configuration_file: options.mutator_conf.clone(),
        apply_coverage,
        // To run tests, compilation must succeed
        verify_mutants: true,
        ..Default::default()
    }
}

/// This function checks if the mutator output path is provided in the configuration file.
///
/// We don't need to check if the mutator output path is provided in the options as they were created
/// from the mutation-test options which does not allow setting it.
#[must_use]
pub fn check_mutator_output_path(options: &move_mutator::cli::CLIOptions) -> Option<PathBuf> {
    options
        .configuration_file
        .as_ref()
        .and_then(|conf| move_mutator::configuration::Configuration::from_file(conf).ok())
        .and_then(|c| c.project.out_mutant_dir)
}

/// The configuration options for running the tests.
// Info: this set struct is based on TestPackage in `aptos-core/crates/aptos/src/move_tool/mod.rs`.
#[derive(Parser, Debug, Clone)]
pub struct TestBuildConfig {
    /// Options for compiling a move package dir.
    // We might move some options out and have our own option struct here - not all options are
    // needed for mutation testing.
    #[clap(flatten)]
    pub move_pkg: MovePackageDir,

    /// Dump storage state on failure.
    #[clap(long = "dump")]
    pub dump_state: bool,

    /// A filter string to determine which unit tests to run.
    #[clap(long, short)]
    pub filter: Option<String>,

    /// A boolean value to skip warnings.
    #[clap(long)]
    pub ignore_compile_warnings: bool,

    /// Compute and then use unit test computed coverage to generate mutants only for covered code.
    #[clap(long = "coverage")]
    pub apply_coverage: bool,
}

impl TestBuildConfig {
    /// Create a [`CompilerConfig`] from the [`TestBuildConfig`].
    pub fn compiler_config(&self) -> CompilerConfig {
        let known_attributes = extended_checks::get_all_attribute_names().clone();
        CompilerConfig {
            known_attributes,
            skip_attribute_checks: self.move_pkg.skip_attribute_checks,
            bytecode_version: get_bytecode_version(
                self.move_pkg.bytecode_version,
                self.move_pkg.language_version,
            ),
            compiler_version: self.move_pkg.compiler_version,
            language_version: self.move_pkg.language_version,
            experiments: experiments_from_opt_level(&self.move_pkg.optimize),
        }
    }

    /// Returns [`TestBuildConfig`] with the coverage option disabled.
    pub fn disable_coverage(&self) -> Self {
        let mut cfg = self.clone();
        cfg.apply_coverage = false;
        cfg
    }
}

/// Get bytecode version.
fn get_bytecode_version(
    bytecode_version_in: Option<u32>,
    language_version: Option<LanguageVersion>,
) -> Option<u32> {
    bytecode_version_in.or_else(|| language_version.map(|lv| lv.infer_bytecode_version(None)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn cli_options_starts_empty() {
        let options = CLIOptions::default();
        assert_eq!(ModuleFilter::All, options.mutate_modules);
        assert_eq!(FunctionFilter::All, options.mutate_functions);
        assert!(options.mutator_conf.is_none());
        assert!(options.output.is_none());
    }

    #[test]
    fn create_mutator_options_copies_fields() {
        let options = crate::cli::CLIOptions {
            mutate_modules: ModuleFilter::Selected(vec!["mod1".to_string(), "mod2".to_string()]),
            mutate_functions: FunctionFilter::Selected(vec![
                "func1".to_string(),
                "func2".to_string(),
            ]),
            mutator_conf: Some(PathBuf::from("path/to/mutator/conf")),
            ..Default::default()
        };

        let no_apply_coverage = false;
        let mutator_options = create_mutator_options(&options, no_apply_coverage);

        assert_eq!(mutator_options.mutate_modules, options.mutate_modules);
        assert_eq!(mutator_options.configuration_file, options.mutator_conf);
    }

    #[test]
    fn check_mutator_output_path_returns_none_when_no_conf() {
        let options = move_mutator::cli::CLIOptions::default();
        assert!(check_mutator_output_path(&options).is_none());
    }

    #[test]
    fn check_mutator_output_path_returns_path_when_conf_exists() {
        let json_content = r#"
            {
                "project": {
                    "out_mutant_dir": "path/to/out_mutant_dir"
                },
                "project_path": "/path/to/project",
                "individual": []
            }
        "#;

        fs::write("test_mutator_conf.json", json_content).unwrap();

        let options = move_mutator::cli::CLIOptions {
            configuration_file: Some(PathBuf::from("test_mutator_conf.json")),
            ..Default::default()
        };

        let path = check_mutator_output_path(&options);
        fs::remove_file("test_mutator_conf.json").unwrap();

        assert!(path.is_some());
        assert_eq!(path.unwrap(), PathBuf::from("path/to/out_mutant_dir"));
    }
}
