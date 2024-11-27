// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use fs_extra::dir::CopyOptions;
use move_mutator::cli::{CLIOptions, FunctionFilter, ModuleFilter};
use move_package::BuildConfig;
use std::{fs, path::PathBuf};
use tempfile::tempdir;

fn clone_project(move_asset_project: &str) -> PathBuf {
    let outdir = tempdir().unwrap().into_path();
    let options = CopyOptions::new().content_only(true);

    if let Err(e) = fs_extra::dir::copy(move_asset_project, &outdir, &options) {
        panic!("failed to clone {move_asset_project:?} to {outdir:?}: {e}");
    }

    outdir
}

fn quick_build_config() -> BuildConfig {
    BuildConfig {
        // Let's make it faster.
        skip_fetch_latest_git_deps: true,
        ..Default::default()
    }
}

const PACKAGE_PATHS: &[&str] = &[
    "tests/move-assets/breakcontinue",
    "tests/move-assets/poor_spec",
    "tests/move-assets/basic_coin",
    "tests/move-assets/relative_dep",
    "tests/move-assets/same_names",
    "tests/move-assets/simple",
    "tests/move-assets/skip_mutation_examples",
    "tests/move-assets/check_swap_operator",
];

// Check if the mutator works correctly on the basic packages.
// It should generate a report with mutants.
#[test]
fn check_mutator_works_correctly() {
    let config = quick_build_config();

    for project in PACKAGE_PATHS {
        let mut package_path = clone_project(project);

        if *project == "tests/move-assets/relative_dep" {
            package_path = package_path.join("p2");
        }

        let outdir = package_path.join("outdir");

        let options = CLIOptions {
            out_mutant_dir: Some(outdir.clone()),
            ..Default::default()
        };

        let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
        assert!(result.is_ok());

        let report_path = outdir.join("report.json");
        assert!(report_path.exists());

        let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();
        assert!(!report.get_mutants().is_empty());
        fs::remove_dir_all(package_path).unwrap();
    }
}

#[test]
fn check_mutator_verify_mutants_correctly() {
    let package_path = clone_project("tests/move-assets/poor_spec");
    let outdir = package_path.join("outdir");

    let options = CLIOptions {
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: true,
        ..Default::default()
    };

    let config = quick_build_config();

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
    assert!(result.is_ok());

    let report_path = outdir.join("report.json");
    assert!(report_path.exists());

    let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();
    assert!(!report.get_mutants().is_empty());
    fs::remove_dir_all(package_path).unwrap();
}

// Check if the mutator fails on non-existing input path.
#[test]
fn check_mutator_fails_on_non_existing_path() {
    let outdir = tempdir().unwrap().into_path();

    let options = CLIOptions {
        out_mutant_dir: Some(outdir.clone()),
        ..Default::default()
    };

    let config = quick_build_config();

    let package_path = PathBuf::from("/very/random/path");

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
    assert!(result.is_err());
}

// Check if the mutator fails on non-existing output path.
#[test]
fn check_mutator_fails_on_non_existing_output_path() {
    let options = CLIOptions {
        out_mutant_dir: Some("/very/bad/path".into()),
        ..Default::default()
    };

    let config = quick_build_config();

    let package_path = clone_project("tests/move-assets/poor_spec");

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
    assert!(result.is_err());
    fs::remove_dir_all(package_path).unwrap();
}

// Check if the mutator works with single files that do not require deps/address resolving.
#[test]
fn check_mutator_works_with_simple_single_files() {
    let package_path = clone_project("tests/move-assets/file_without_package");
    let outdir = package_path.join("outdir");

    let options = CLIOptions {
        move_sources: vec![package_path.join("Sub.move")],
        out_mutant_dir: Some(outdir.clone()),
        ..Default::default()
    };

    let config = quick_build_config();

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
    assert!(result.is_ok());

    let report_path = outdir.join("report.json");
    assert!(report_path.exists());

    let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();
    assert!(!report.get_mutants().is_empty());
    fs::remove_dir_all(package_path).unwrap();
}

// Check if the mutator fails properly with single files that do require deps/address resolving.
#[test]
fn check_mutator_properly_fails_with_single_files_that_require_dep_or_addr_resolving() {
    let package_path = clone_project("tests/move-assets/simple");
    let outdir = package_path.join("outdir");

    let options = CLIOptions {
        move_sources: vec![package_path.join("sources/Sum.move")],
        out_mutant_dir: Some(outdir.clone()),
        ..Default::default()
    };

    let config = quick_build_config();

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
    assert!(result.is_err());

    let report_path = outdir.join("report.json");
    assert!(!report_path.exists());
    fs::remove_dir_all(package_path).unwrap();
}

// Check if the mutator produce zero mutants if verification is enabled for
// files without any package (we're unable to verify such files successfully).
#[test]
fn check_mutator_fails_verify_file_without_package() {
    let package_path = clone_project("tests/move-assets/file_without_package");
    let outdir = package_path.join("outdir");

    let options = CLIOptions {
        move_sources: vec![package_path.join("Sub.move")],
        out_mutant_dir: Some(outdir.clone()),
        verify_mutants: true,
        ..Default::default()
    };

    let config = quick_build_config();

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
    assert!(result.is_ok());

    let report_path = outdir.join("report.json");
    assert!(report_path.exists());

    let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();
    assert!(report.get_mutants().is_empty());
    fs::remove_dir_all(package_path).unwrap();
}

// Check that the mutator will apply function-filters correctly and generate mutants only for
// specified functions.
#[test]
fn check_mutator_cli_filters_functions_properly() {
    let package_path = clone_project("tests/move-assets/simple");
    let outdir = package_path.join("outdir");

    // All these functions exist in the project `simple`.
    let target_function_1 = "or";
    let target_function_2 = "sum";
    let not_included = "and";

    let options = CLIOptions {
        mutate_functions: FunctionFilter::Selected(vec![
            target_function_1.into(),
            target_function_2.into(),
        ]),
        out_mutant_dir: Some(outdir.clone()),
        ..Default::default()
    };

    let config = quick_build_config();

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
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
    fs::remove_dir_all(package_path).unwrap();
}

// This test runs a mutator multiple times and checks number of binary swap operator mutants for
// each run on specific function.
#[test]
fn check_mutator_swap_operator_works_correctly_for_corner_cases() {
    let config = quick_build_config();

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
        let package_path = clone_project("tests/move-assets/check_swap_operator");
        let outdir = package_path.join("outdir");

        let options = CLIOptions {
            mutate_functions: FunctionFilter::Selected(vec![fn_name.into()]),
            out_mutant_dir: Some(outdir.clone()),
            ..Default::default()
        };

        let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
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
        fs::remove_dir_all(package_path).unwrap();
    }
}

// This test runs a mutator multiple times and checks number of binary swap operator mutants for
// each run on specific function.
#[test]
fn check_mutator_binary_replacement_operator_works_correctly_for_corner_cases_v1() {
    let config = quick_build_config();

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
        let package_path = clone_project("tests/move-assets/simple");
        let outdir = package_path.join("outdir");

        let options = CLIOptions {
            mutate_modules: ModuleFilter::Selected(vec!["BinaryReplacement".to_owned()]),
            mutate_functions: FunctionFilter::Selected(vec![fn_name.into()]),
            out_mutant_dir: Some(outdir.clone()),
            ..Default::default()
        };

        let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
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
        fs::remove_dir_all(package_path).unwrap();
    }
}

#[test]
fn check_mutator_uses_skip_mutation_attribute_properly() {
    let config = quick_build_config();

    let expected_module = "BasicOps";
    let expected_fn = "sum";
    let skipped_module = "SkipSum";
    let skipped_fn = ["skip_sub", "skip_sum"];

    let package_path = clone_project("tests/move-assets/skip_mutation_examples");
    let outdir = package_path.join("outdir");

    let options = CLIOptions {
        out_mutant_dir: Some(outdir.clone()),
        ..Default::default()
    };

    let result = move_mutator::run_move_mutator(options.clone(), &config, &package_path);
    assert!(result.is_ok());

    let report_path = outdir.join("report.json");
    let report = move_mutator::report::Report::load_from_json_file(&report_path).unwrap();

    // Ensure that nobody has deleted our Move test.
    assert!(!report.get_mutants().is_empty());

    for mutant in report.get_mutants() {
        let module_name = mutant.get_module_name();
        let fn_name = mutant.get_function_name();

        // A few redundant checks, but it depicts the idea of the test.
        assert!(fn_name == expected_fn);
        assert!(!skipped_fn.contains(&fn_name));
        assert!(module_name == expected_module);
        assert!(module_name != skipped_module);
    }
    fs::remove_dir_all(package_path).unwrap();
}
