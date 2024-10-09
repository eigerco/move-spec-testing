// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::cli::TestBuildConfig;
use anyhow::Error;
use aptos::move_tool::aptos_debug_natives::aptos_debug_natives;
use aptos_gas_schedule::{MiscGasParameters, NativeGasParameters};
use aptos_types::on_chain_config::aptos_test_feature_flags_genesis;
use move_cli::base::test::UnitTestResult;
use move_command_line_common::address::NumericalAddress;
use move_package::BuildConfig;
use move_unit_test::UnitTestingConfig;
use std::path::Path;
use termcolor::WriteColor;

/// The `run_tests` function is responsible for running the tests for the provided package.
///
/// # Arguments
///
/// * `cfg` - A `TestBuildConfig` representing the test configuration.
/// * `package_path` - A `Path` to the package.
/// * `error_writer` - `&mut dyn std::io::Write` the error writer.
///
/// # Returns
///
/// * `anyhow::Result<()>` - The result of the test suite for the package.
// This function is based upon the `execute` method for the `TestPacakge` struct in
// aptos-core/crates/aptos/src/move_tool/mod.rs file.
pub(crate) fn run_tests<W: WriteColor + Send>(
    cfg: &TestBuildConfig,
    package_path: &Path,
    skip_fetch_latest_git_deps: bool,
    mut error_writer: &mut W,
) -> anyhow::Result<()> {
    let config = BuildConfig {
        dev_mode: cfg.move_pkg.dev,
        additional_named_addresses: cfg.move_pkg.named_addresses(),
        test_mode: true,
        full_model_generation: cfg.move_pkg.check_test_code,
        install_dir: cfg.move_pkg.output_dir.clone(),
        skip_fetch_latest_git_deps,
        compiler_config: cfg.compiler_config(),
        ..Default::default()
    };

    let natives = aptos_debug_natives(NativeGasParameters::zeros(), MiscGasParameters::zeros());
    let cost_table = None;
    let gas_limit = None; // unlimited.

    let result = move_cli::base::test::run_move_unit_tests(
        package_path,
        config.clone(),
        UnitTestingConfig {
            filter: cfg.filter.clone(),
            report_stacktrace_on_abort: true,
            report_storage_on_error: cfg.dump_state,
            ignore_compile_warnings: cfg.ignore_compile_warnings,
            named_address_values: cfg
                .move_pkg
                .named_addresses()
                .iter()
                .map(|(name, account_address)| {
                    (
                        name.clone(),
                        NumericalAddress::from_account_address(*account_address),
                    )
                })
                .collect(),
            ..UnitTestingConfig::default()
        },
        natives,
        aptos_test_feature_flags_genesis(),
        gas_limit,
        cost_table,
        cfg.apply_coverage,
        &mut error_writer,
    )
    .map_err(|err| Error::msg(format!("failed to run unit tests: {err:#}")))?;

    match result {
        UnitTestResult::Success => Ok(()),
        UnitTestResult::Failure => Err(Error::msg("Move unit test error")),
    }
}
