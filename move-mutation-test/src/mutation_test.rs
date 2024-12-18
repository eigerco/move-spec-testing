// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::cli::TestBuildConfig;
use anyhow::{anyhow, Error};
use aptos::move_tool::aptos_debug_natives::aptos_debug_natives;
use aptos_gas_schedule::{MiscGasParameters, NativeGasParameters};
use aptos_types::on_chain_config::aptos_test_feature_flags_genesis;
use move_cli::base::test::UnitTestResult;
use move_command_line_common::address::NumericalAddress;
use move_package::BuildConfig;
use move_unit_test::UnitTestingConfig;
use std::{fs, path::Path};
use termcolor::WriteColor;

/// Runs tests on the original code and produces a nice informative output.
///
/// # Arguments
///
/// * `cfg` - A `TestBuildConfig` representing the test configuration.
/// * `package_path` - A `Path` to the package.
///
/// # Returns
///
/// * `anyhow::Result<()>` - The result of the test suite for the package.
pub(crate) fn run_tests_on_original_code(
    cfg: &TestBuildConfig,
    package_path: &Path,
) -> anyhow::Result<()> {
    let mut error_writer = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);

    // Show informative statistics to users.
    let report_statistics = true;

    // We need to check for the latest git deps only for the first time we run the test.
    let skip_fetch_deps = false;

    let result = run_tests(
        cfg,
        package_path,
        skip_fetch_deps,
        report_statistics,
        &mut error_writer,
    );

    if let Err(e) = result {
        let msg = format!(
            "Test suite is failing for the original code! Unit test failed with error: {e}"
        );
        error!("{msg}");
        return Err(anyhow!(msg));
    }

    Ok(())
}

/// Runs tests on the mutated code.
///
/// This test run avoids generating output to the screen and fetching the latest dependency since
/// that should be handled by the `run_tests_on_original_code` function, which should be executed
/// before.
///
/// # Arguments
///
/// * `cfg` - A `TestBuildConfig` representing the test configuration.
/// * `package_path` - A `Path` to the package.
///
/// # Returns
///
/// * `anyhow::Result<()>` - The result of the test suite for the package.
pub(crate) fn run_tests_on_mutated_code(
    cfg: &TestBuildConfig,
    package_path: &Path,
) -> anyhow::Result<()> {
    // Ignore statistics on mutants.
    let report_statistics = false;

    // No need to fetch latest deps again.
    let skip_fetch_deps = true;

    // No need to print anything to the screen, due to many threads, it might be messy and slow.
    let mut error_writer = std::io::sink();

    // Do not calculate the coverage on mutants.
    let test_config = cfg.disable_coverage();

    run_tests(
        &test_config,
        package_path,
        skip_fetch_deps,
        report_statistics,
        &mut error_writer,
    )
}

/// The `run_tests` function is responsible for running the tests for the provided package.
// This function is based upon the `execute` method for the `TestPackage` struct in
// aptos-core/crates/aptos/src/move_tool/mod.rs file.
fn run_tests<W: WriteColor + Send>(
    cfg: &TestBuildConfig,
    package_path: &Path,
    skip_fetch_latest_git_deps: bool,
    report_statistics: bool,
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
    // By using a reasonably large value, we ensure the original test suite will always pass,
    // while mutants with infinite loops will be killed quite quickly.
    let gas_limit = Some(cfg.gas_limit);

    let result = move_cli::base::test::run_move_unit_tests(
        package_path,
        config.clone(),
        UnitTestingConfig {
            filter: cfg.filter.clone(),
            report_storage_on_error: cfg.dump_state,
            ignore_compile_warnings: cfg.ignore_compile_warnings,
            report_statistics,
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

    // Disk space optimization:
    if cfg.apply_coverage {
        let trace_path = package_path.join(".trace");
        info!("removing {}", trace_path.display());
        // Our tool doesn't use the .trace file at all, only the .coverage_map.mvcov file, and
        // since the tool copy package directory to temp directories for when running tests,
        // so let's keep copied directory as small as possible.
        let _ = fs::remove_file(trace_path);
    }

    match result {
        UnitTestResult::Success => Ok(()),
        UnitTestResult::Failure => Err(Error::msg("Move unit test error")),
    }
}
