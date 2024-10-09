// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use move_mutator::{
    cli::{CLIOptions, ModuleFilter},
    configuration::FunctionFilter,
};
use move_package::BuildConfig;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const PACKAGE_PATHS: &[&str] = &[
    "tests/move-assets/breakcontinue",
    "tests/move-assets/poor_spec",
    "tests/move-assets/basic_coin",
    "tests/move-assets/relative_dep/p2",
    "tests/move-assets/same_names",
    "tests/move-assets/simple",
];

// Check if the mutator works correctly on the basic packages.
// It should generate a report with mutants.
#[test]
fn check_mutator_works_correctly() {
    let outdir = tempdir().unwrap().into_path();

    let options = CLIOptions {
        move_sources: vec![],
        mutate_modules: ModuleFilter::All,
        mutate_functions: FunctionFilter::All,
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: false,
        no_overwrite: false,
        downsample_filter: None,
        downsampling_ratio_percentage: None,
        configuration_file: None,
        apply_coverage: false,
    };

    let config = BuildConfig {
        // Let's make it faster.
        skip_fetch_latest_git_deps: true,
        ..Default::default()
    };

    for package_path in PACKAGE_PATHS {
        let package_path = Path::new(package_path);

        let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
        assert!(result.is_ok());

        let report_path = outdir.join("report.json");
        assert!(report_path.exists());

        let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();
        assert!(!report.get_mutants().is_empty());
    }
}

#[test]
fn check_mutator_verify_mutants_correctly() {
    let outdir = tempdir().unwrap().into_path();

    let options = CLIOptions {
        move_sources: vec![],
        mutate_modules: ModuleFilter::All,
        mutate_functions: FunctionFilter::All,
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: true,
        no_overwrite: false,
        downsample_filter: None,
        downsampling_ratio_percentage: None,
        configuration_file: None,
        apply_coverage: false,
    };

    let config = BuildConfig::default();

    let package_path = Path::new(PACKAGE_PATHS[1]);

    let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
    assert!(result.is_ok());

    let report_path = outdir.join("report.json");
    assert!(report_path.exists());

    let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();
    assert!(!report.get_mutants().is_empty());
}

// Check if the mutator fails on non-existing input path.
#[test]
fn check_mutator_fails_on_non_existing_path() {
    let outdir = tempdir().unwrap().into_path();

    let options = CLIOptions {
        move_sources: vec![],
        mutate_modules: ModuleFilter::All,
        mutate_functions: FunctionFilter::All,
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: false,
        no_overwrite: false,
        downsample_filter: None,
        downsampling_ratio_percentage: None,
        configuration_file: None,
        apply_coverage: false,
    };

    let config = BuildConfig::default();

    let package_path = PathBuf::from("/very/random/path");

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
    assert!(result.is_err());
}

// Check if the mutator fails on non-existing output path.
#[test]
fn check_mutator_fails_on_non_existing_output_path() {
    let options = CLIOptions {
        move_sources: vec![],
        mutate_modules: ModuleFilter::All,
        mutate_functions: FunctionFilter::All,
        out_mutant_dir: Some("/very/bad/path".into()),
        verify_mutants: false,
        no_overwrite: false,
        downsample_filter: None,
        downsampling_ratio_percentage: None,
        configuration_file: None,
        apply_coverage: false,
    };

    let config = BuildConfig::default();

    let package_path = Path::new(PACKAGE_PATHS[0]);

    let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
    assert!(result.is_err());
}

// Check if the mutator works with single files that do not require deps/address resolving.
#[test]
fn check_mutator_works_with_simple_single_files() {
    let outdir = tempdir().unwrap().into_path();

    let options = CLIOptions {
        move_sources: vec!["tests/move-assets/file_without_package/Sub.move".into()],
        mutate_modules: ModuleFilter::All,
        mutate_functions: FunctionFilter::All,
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: false,
        no_overwrite: false,
        downsample_filter: None,
        downsampling_ratio_percentage: None,
        configuration_file: None,
        apply_coverage: false,
    };

    let config = BuildConfig::default();

    let package_path = Path::new(".");

    let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
    assert!(result.is_ok());

    let report_path = outdir.join("report.json");
    assert!(report_path.exists());

    let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();
    assert!(!report.get_mutants().is_empty());
}

// Check if the mutator fails properly with single files that do require deps/address resolving.
#[test]
fn check_mutator_properly_fails_with_single_files_that_require_dep_or_addr_resolving() {
    let outdir = tempdir().unwrap().into_path();

    let options = CLIOptions {
        move_sources: vec!["tests/move-assets/simple/sources/Sum.move".into()],
        mutate_modules: ModuleFilter::All,
        mutate_functions: FunctionFilter::All,
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: false,
        no_overwrite: false,
        downsample_filter: None,
        downsampling_ratio_percentage: None,
        configuration_file: None,
        apply_coverage: false,
    };

    let config = BuildConfig::default();

    let package_path = Path::new(".");

    let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
    assert!(result.is_err());

    let report_path = outdir.join("report.json");
    assert!(!report_path.exists());
}

// Check if the mutator produce zero mutants if verification is enabled for
// files without any package (we're unable to verify such files successfully).
#[test]
fn check_mutator_fails_verify_file_without_package() {
    let outdir = tempdir().unwrap().into_path();

    let options = CLIOptions {
        move_sources: vec!["tests/move-assets/file_without_package/Sub.move".into()],
        mutate_modules: ModuleFilter::All,
        mutate_functions: FunctionFilter::All,
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: true,
        no_overwrite: false,
        downsample_filter: None,
        downsampling_ratio_percentage: None,
        configuration_file: None,
        apply_coverage: false,
    };

    let config = BuildConfig::default();

    let package_path = Path::new(".");

    let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
    assert!(result.is_ok());

    let report_path = outdir.join("report.json");
    assert!(report_path.exists());

    let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();
    assert!(report.get_mutants().is_empty());
}

// Check that the mutator will apply function-filters correctly and generate mutants only for
// specified functions.
#[test]
fn check_mutator_cli_filters_functions_properly() {
    let outdir = tempdir().unwrap().into_path();

    // All these functions exist in the project `simple`.
    let target_function_1 = "or";
    let target_function_2 = "sum";
    let not_included = "and";

    let options = CLIOptions {
        move_sources: vec![],
        mutate_modules: ModuleFilter::All,
        mutate_functions: FunctionFilter::Selected(vec![
            target_function_1.into(),
            target_function_2.into(),
        ]),
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: false, // skip verifying, this is a huge project
        no_overwrite: false,
        downsample_filter: None,
        downsampling_ratio_percentage: None,
        configuration_file: None,
        apply_coverage: false,
    };

    let config = BuildConfig::default();
    let package_path = Path::new("tests/move-assets/simple");

    let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
    assert!(result.is_ok());

    let report_path = outdir.join("report.json");
    assert!(report_path.exists());

    let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();

    // Ensure the report list is not empty.
    assert!(!report.get_mutants().is_empty());

    // Ensure that all mutations belong to a single function.
    for mutant in report.get_mutants() {
        let mutated_func = mutant.get_function_name();
        assert_ne!(mutated_func, not_included);

        // We expect that the mutated function must be one of the following target functions:
        assert!((target_function_1 == mutated_func) ^ (target_function_2 == mutated_func));
    }
}

// This test runs a mutator multiple times and checks number of binary swap operator mutants for
// each run on specific function.
#[test]
fn check_mutator_swap_operator_works_correctly_for_corner_cases() {
    let config = BuildConfig {
        // Let's make it faster.
        skip_fetch_latest_git_deps: true,
        ..Default::default()
    };

    // Function names and number of swap mutation operations
    let functions_and_exected_swap_op_count = [
        ("swap_op_should_mutate_once_v1", 1),
        ("swap_op_should_mutate_once_v2", 1),
        ("swap_op_should_mutate_once_v3", 1),
        ("swap_op_should_mutate_seven_times", 7),
        ("swap_op_should_mutate_three_times", 3),
        ("swap_op_should_not_mutate", 0),
    ];

    for (fn_name, expected_swap_op_count) in functions_and_exected_swap_op_count {
        let outdir = tempdir().unwrap().into_path();

        let options = CLIOptions {
            move_sources: vec![],
            mutate_modules: ModuleFilter::All,
            mutate_functions: FunctionFilter::Selected(vec![fn_name.into()]),
            out_mutant_dir: Some(outdir.clone()),
            verify_mutants: false,
            no_overwrite: false,
            downsample_filter: None,
            downsampling_ratio_percentage: None,
            configuration_file: None,
            apply_coverage: false,
        };
        let package_path = Path::new("tests/move-assets/check_swap_operator");

        let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
        assert!(result.is_ok());

        let report_path = outdir.join("report.json");
        let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();

        // Ensure that all mutations belong to a single function.
        let mut total_swap_op_count = 0;
        for mutant in report.get_mutants() {
            assert!(mutant.get_function_name() == fn_name);

            total_swap_op_count += mutant
                .get_mutations()
                .iter()
                // Note: we could expose operator names publicly.
                .filter(|m| m.get_operator_name() == "binary_operator_swap")
                .count();
        }

        assert_eq!(
            total_swap_op_count, expected_swap_op_count,
            "failed for function {fn_name}"
        );
    }
}

// This test runs a mutator multiple times and checks number of binary swap operator mutants for
// each run on specific function.
#[test]
fn check_mutator_binary_replacement_operator_works_correctly_for_corner_cases_v1() {
    let config = BuildConfig {
        // Let's make it faster.
        skip_fetch_latest_git_deps: true,
        ..Default::default()
    };

    // Function names and number of swap mutation operations
    let functions_cur_op_forbidden_ops = [
        // (<function_name>, <original_operator>, <forbidden_mutations>)
        ("is_x_eq_to_zero", "==", vec!["<="]),
        ("is_zero_eq_to_x", "==", vec![">="]),
        ("is_x_neq_to_zero", "!=", vec![">"]),
        ("is_zero_neq_to_x", "!=", vec!["<"]),
        ("is_x_gt_zero", ">", vec!["!="]),
        ("is_zero_lt_x", "<", vec!["!="]),
    ];

    for (fn_name, orig_op, forbidden_ops) in functions_cur_op_forbidden_ops {
        let outdir = tempdir().unwrap().into_path();

        let options = CLIOptions {
            move_sources: vec![],
            mutate_modules: ModuleFilter::Selected(vec!["BinaryReplacement".to_owned()]),
            mutate_functions: FunctionFilter::Selected(vec![fn_name.into()]),
            out_mutant_dir: Some(outdir.clone()),
            verify_mutants: false,
            no_overwrite: false,
            downsample_filter: None,
            downsampling_ratio_percentage: None,
            configuration_file: None,
            apply_coverage: false,
        };
        let package_path = Path::new("tests/move-assets/simple");

        let result = move_mutator::run_move_mutator(options.clone(), &config, package_path);
        assert!(result.is_ok());

        let report_path = outdir.join("report.json");
        let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();

        // Ensure that nobody has deleted our Move test.
        assert!(!report.get_mutants().is_empty());

        for mutant in report.get_mutants() {
            // Ensure that all mutations belong to a single function.
            assert!(mutant.get_function_name() == fn_name);

            let has_forbidden_op = mutant
                .get_mutations()
                .iter()
                .filter(|m| {
                    m.get_operator_name() == "binary_operator_replacement"
                        && m.get_original_value() == orig_op
                })
                .find(|m| forbidden_ops.contains(&m.get_new_value()));
            assert!(has_forbidden_op.is_none());
        }
    }
}
