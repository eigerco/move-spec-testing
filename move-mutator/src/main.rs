// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::Parser;
use move_mutator::{
    cli::{CLIOptions, PackagePathCheck},
    run_move_mutator,
};
use move_package::BuildConfig;
use std::path::PathBuf;

#[derive(Default, Parser, Debug, Clone)]
pub struct Opts {
    /// The path to the target Move package.
    #[clap(long, value_parser)]
    pub package_dir: Option<PathBuf>,

    /// Command line options for mutator.
    #[clap(flatten)]
    pub cli_options: CLIOptions,

    /// The build configuration for the Move package.
    #[clap(flatten)]
    pub build_config: BuildConfig,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let package_path = opts.cli_options.resolve(opts.package_dir)?;

    run_move_mutator(opts.cli_options, &opts.build_config, &package_path)
}
