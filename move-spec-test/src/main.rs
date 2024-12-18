// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use move_mutator::cli::PackagePathCheck;
use move_package::BuildConfig;
use move_spec_test::{cli::CLIOptions, run_spec_test};
use mutator_common::display_report::DisplayReportOptions;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Opts {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs the specification test tool.
    Run {
        /// The path to the target Move package.
        #[clap(long, value_parser)]
        package_dir: Option<PathBuf>,

        /// Command line options for specification tester.
        #[clap(flatten)]
        cli_options: CLIOptions,

        /// The build configuration options.
        #[clap(flatten)]
        build_config: BuildConfig,
    },

    /// Display the report in a more readable format.
    DisplayReport(DisplayReportOptions),
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    match opts.command {
        Commands::Run {
            package_dir,
            cli_options,
            build_config,
        } => {
            let package_path = cli_options.resolve(package_dir)?;
            run_spec_test(&cli_options, &build_config, &package_path)
        },
        Commands::DisplayReport(display_report) => display_report.execute(),
    }
}
