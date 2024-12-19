// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use std::{path::PathBuf, str::FromStr};

pub const DEFAULT_OUTPUT_DIR: &str = "mutants_output";

/// Command line options for mutator
#[derive(Parser, Debug, Clone)]
pub struct CLIOptions {
    /// The paths to the Move sources.
    #[clap(long, value_parser)]
    pub move_sources: Vec<PathBuf>,

    /// Module names to be mutated.
    #[clap(long, value_parser, default_value = "all")]
    pub mutate_modules: ModuleFilter,

    /// Function names to be mutated.
    #[clap(long, value_parser, default_value = "all")]
    pub mutate_functions: FunctionFilter,

    /// The path where to put the output files.
    #[clap(long, value_parser)]
    pub out_mutant_dir: Option<PathBuf>,

    /// Indicates if mutants should be verified and made sure mutants can compile.
    #[clap(long, default_value = "false", conflicts_with = "move_sources")]
    pub verify_mutants: bool,

    /// Indicates if the output files should be overwritten.
    #[clap(long, default_value = "false")]
    pub no_overwrite: bool,

    /// Remove averagely given percentage of mutants. See the doc for more details.
    #[clap(long)]
    pub downsampling_ratio_percentage: Option<usize>,

    /// Use the unit test coverage report to generate mutants for source code with unit test coverage.
    #[clap(long = "coverage", conflicts_with = "move_sources")]
    pub apply_coverage: bool,
}

/// Checker for conflicts with CLI arguments.
pub trait PackagePathCheck<'a> {
    /// Gets the paths to the Move sources.
    fn get_move_sources(&'a self) -> &'a Vec<PathBuf>;

    /// Returns package path after checking for conflicts.
    fn resolve(&'a self, package_path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
        if let Some(path) = package_path {
            if !self.get_move_sources().is_empty() {
                anyhow::bail!("the '--move-sources <MOVE_SOURCES>' is not compatible with the '--package_path <PACKAGE_PATH>' argument");
            }

            return Ok(path);
        }

        Ok(PathBuf::from("."))
    }
}

impl<'a> PackagePathCheck<'a> for CLIOptions {
    fn get_move_sources(&'a self) -> &'a Vec<PathBuf> {
        &self.move_sources
    }
}

impl Default for CLIOptions {
    // We need to implement default just because we need to specify the default value for out_mutant_dir.
    // Otherwise, out_mutant_dir would be empty. This is special case, when user won't specify any Options
    // (so the default value would be used), but define package_path (which is passed using other mechanism).
    fn default() -> Self {
        Self {
            move_sources: vec![],
            mutate_modules: ModuleFilter::All,
            mutate_functions: FunctionFilter::All,
            out_mutant_dir: Some(PathBuf::from(DEFAULT_OUTPUT_DIR)),
            verify_mutants: false,
            no_overwrite: false,
            apply_coverage: false,
            downsampling_ratio_percentage: None,
        }
    }
}

/// Filter allowing to select modules to be mutated.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum ModuleFilter {
    #[default]
    All,
    Selected(Vec<String>),
}

impl FromStr for ModuleFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(ModuleFilter::All),
            _ => Ok(ModuleFilter::Selected(
                s.split(&[';', '-', ',']).map(String::from).collect(),
            )),
        }
    }
}

/// Filter for the functions to mutate.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum FunctionFilter {
    #[default]
    All,
    Selected(Vec<String>),
}

impl FromStr for FunctionFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(FunctionFilter::All),
            _ => Ok(FunctionFilter::Selected(
                s.split(&[';', '-', ',']).map(String::from).collect(),
            )),
        }
    }
}
