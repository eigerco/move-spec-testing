//! A module for displaying reports in a nice fashion.
// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use super::report::{MutantStats, Report};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use diffy::{Line, Patch, PatchFormatter};
use prettytable::{
    color,
    format::{self, Alignment, LinePosition, LineSeparator},
    Attr, Cell, Row, Table,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    str::FromStr,
};

const COLOR_HAPPY: Option<Attr> = Some(Attr::ForegroundColor(color::GREEN));
const COLOR_WARN: Option<Attr> = Some(Attr::ForegroundColor(color::BRIGHT_YELLOW));
const COLOR_CRITICAL: Option<Attr> = Some(Attr::ForegroundColor(color::RED));
const COLOR_NONE: Option<Attr> = None;

#[derive(Subcommand)]
pub enum DisplayReportCmd {
    /// Summarize the report.
    Summary,

    /// Display report in the coverage format.
    Coverage {
        /// Include specified modules in the report.
        #[clap(long, value_parser, default_value = "all")]
        modules: ModuleFilter,
    },

    /// Display mutants.
    Mutants {
        /// Include specified modules in the report.
        #[clap(long, value_parser, default_value = "all")]
        modules: ModuleFilter,

        /// Include specified functions in the output.
        #[clap(long, value_parser, default_value = "all")]
        functions: FunctionFilter,

        /// Specify which mutants to print.
        #[clap(long, default_value = "alive")]
        mutants: MutantFilter,
    },
}

/// Display the report in a more readable format.
#[derive(Parser)]
pub struct DisplayReportOptions {
    /// Report location. The default file is "report.txt" under the same directory.
    #[clap(global = true, long, default_value = "report.txt")]
    pub path_to_report: PathBuf,

    /// Display report subcommands.
    #[clap(subcommand)]
    pub cmds: DisplayReportCmd,
}

impl DisplayReportOptions {
    /// Execute the command.
    pub fn execute(&self) -> Result<()> {
        let path_to_report = &self.path_to_report;

        match &self.cmds {
            DisplayReportCmd::Summary => display_summary(path_to_report),
            DisplayReportCmd::Coverage { modules } => {
                display_coverage_on_screen(path_to_report, modules)
            },
            DisplayReportCmd::Mutants {
                modules,
                functions,
                mutants,
            } => display_mutants_on_screen(path_to_report, modules, functions, mutants),
        }
    }
}

/// Filter for mutants to be included in the output.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum MutantFilter {
    #[default]
    Alive,
    Killed,
    All,
}

impl MutantFilter {
    /// Check whether the filter allows killed mutants.
    fn contains_killed(&self) -> bool {
        *self == Self::All || *self == Self::Killed
    }

    /// Check whether the filter allows alive mutants.
    fn contains_alive(&self) -> bool {
        *self == Self::All || *self == Self::Alive
    }
}

impl FromStr for MutantFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alive" => Ok(MutantFilter::Alive),
            "killed" => Ok(MutantFilter::Killed),
            "all" => Ok(MutantFilter::All),
            _ => Err("Invalid mutant option. Allowed only: ".to_owned()),
        }
    }
}

/// Filter for functions to include in the report.
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

/// Filter for modules to include in the report.
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

impl ModuleFilter {
    fn get_all_files_containing_the_modules(&self, report: &Report) -> BTreeSet<PathBuf> {
        match *self {
            Self::All => report.entries().keys().cloned().collect(),
            Self::Selected(ref modules) => {
                let mut files_to_print = BTreeSet::<PathBuf>::new();

                for module in modules {
                    for (file, mutants) in report.entries() {
                        if mutants.iter().any(|m| &m.get_module_name() == module) {
                            files_to_print.insert(file.clone());
                            break;
                        }
                    }
                }
                files_to_print
            },
        }
    }
}

/// Line stats for mutations.
#[derive(Default, Debug)]
struct MutatedLine {
    /// Number of total mutants.
    total_mutants: u32,

    /// Number of killed mutants.
    killed_mutants: u32,
}

impl From<&MutantStats> for MutatedLine {
    fn from(mutant_stats: &MutantStats) -> Self {
        Self {
            killed_mutants: mutant_stats.killed,
            total_mutants: mutant_stats.tested,
        }
    }
}

/// Line number. The first line is indexed from 1.
type LineNumber = usize;

/// File statistics about the mutated lines.
#[derive(Default, Debug)]
struct FileStats {
    /// Info about mutated lines.
    mutated_lines: BTreeMap<LineNumber, MutatedLine>,
}

impl FileStats {
    fn increment_killed_per_line(&mut self, line_number: LineNumber) {
        let mutated_line = self.mutated_lines.entry(line_number).or_default();
        mutated_line.total_mutants += 1;
        mutated_line.killed_mutants += 1;
    }

    fn increment_total_per_line(&mut self, line_number: LineNumber) {
        let mutated_line = self.mutated_lines.entry(line_number).or_default();
        mutated_line.total_mutants += 1;
    }
}

/// Displays a friendly readable report for given modules.
pub fn display_coverage_on_screen(
    path_to_report: impl AsRef<Path>,
    modules: &ModuleFilter,
) -> Result<()> {
    let report = Report::load_from_json_file(path_to_report.as_ref())?;
    let files_to_print = modules.get_all_files_containing_the_modules(&report);

    if files_to_print.is_empty() {
        println!("No matching files found.");
        return Ok(());
    };

    println!("The legend is shown below in the table format");
    display_nice_legend_info();
    println!(); // One empty line before the actual result.

    for file in files_to_print {
        let file_stats = calculate_file_stats(&file, &report)?;

        // Get the absolute file path.
        let abs_file_path = report.get_package_dir().to_path_buf().join(&file);
        let source_code = std::fs::read_to_string(&abs_file_path)?;

        display_nice_file_report(&file, source_code, file_stats)?;
    }

    Ok(())
}

fn get_formatted_table() -> Table {
    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('│')
        .separator(LinePosition::Title, LineSeparator::new('=', '┼', '─', '─'))
        .separator(LinePosition::Top, LineSeparator::new('=', '┬', '─', '─'))
        .indent(0)
        .padding(1, 1)
        .build();
    table.set_format(format);
    table
}

fn display_nice_legend_info() {
    let mut table = get_formatted_table();

    let title = Cell::new_align("Source code file path", Alignment::LEFT).with_style(Attr::Bold);
    let helper_table_cell = Cell::new("mutants killed / mutants in total");
    table.set_titles(Row::new(vec![helper_table_cell, title]));

    let mut add_row = |left_text, right_text, color: Option<Attr>| {
        let mut left_cell = Cell::new_align(left_text, Alignment::RIGHT);
        let mut right_cell = Cell::new(right_text);
        if let Some(color) = color {
            left_cell.style(color);
            right_cell.style(color);
        }
        table.add_row(Row::new(vec![left_cell, right_cell]));
    };

    add_row("<examples below>", "<Line>", COLOR_NONE);
    add_row("", "", COLOR_NONE);
    add_row("", "Line without any mutants", COLOR_NONE);
    add_row("6/8", "Some mutants killed on this line", COLOR_WARN);
    add_row("", "Another line without any mutants", COLOR_NONE);
    add_row("10/10", "All mutants killed on this line", COLOR_HAPPY);
    add_row("0/4", "No mutants killed on this line", COLOR_CRITICAL);
    add_row("", "One final line without mutants", COLOR_NONE);

    table.printstd();
}

fn display_nice_file_report(file: &Path, source_code: String, stats: FileStats) -> Result<()> {
    let mut table = get_formatted_table();

    let title = Cell::new_align(file.to_str().expect("invalid path"), Alignment::LEFT)
        .with_style(Attr::Bold);
    let helper_table_cell = Cell::new("K/T");
    table.set_titles(Row::new(vec![helper_table_cell, title]));

    // Line numbers are indexed from 1, not from 0.
    for (line_no, line) in (1..).zip(source_code.lines()) {
        let (mut stat_cell, line_color) = if let Some(m) = stats.mutated_lines.get(&line_no) {
            let style_color = match m.killed_mutants {
                0 => COLOR_CRITICAL,
                killed if killed == m.total_mutants => COLOR_HAPPY,
                _ => COLOR_WARN,
            };

            let text = format!("{}/{}", m.killed_mutants, m.total_mutants);
            (Cell::new_align(&text, Alignment::RIGHT), style_color)
        } else {
            (Cell::new(""), COLOR_NONE)
        };

        let mut line_cell = Cell::new(line);
        if let Some(color) = line_color {
            line_cell.style(color);
            stat_cell.style(color);
        }

        table.add_row(Row::new(vec![stat_cell, line_cell]));
    }

    table.printstd();
    Ok(())
}

fn calculate_file_stats(file: &Path, report: &Report) -> Result<FileStats> {
    let mut file_stats = FileStats::default();

    let Some(mutants) = report.entries().get(&file.to_path_buf()) else {
        return Ok(file_stats);
    };

    for mutant in mutants {
        for patch_str in &mutant.mutants_alive_diffs {
            let mutated_line_no = find_mutated_line_number(patch_str)?;
            file_stats.increment_total_per_line(mutated_line_no);
        }
        for patch_str in &mutant.mutants_killed_diff {
            let mutated_line_no = find_mutated_line_number(patch_str)?;
            file_stats.increment_killed_per_line(mutated_line_no);
        }
    }

    Ok(file_stats)
}

fn find_mutated_line_number(file_diff: &str) -> Result<usize> {
    let patch = diffy::Patch::from_str(file_diff)?;
    let hunk = patch
        .hunks()
        .first()
        .context("invalid diff in the report")?;

    let mut current_line_no = hunk.old_range().start();
    let mut lines = hunk.lines().iter();

    // Loop until Line::Deleted or Line::Insert.
    while let Some(Line::Context(_)) = lines.next() {
        current_line_no += 1;
    }

    Ok(current_line_no)
}

/// Displays mutants in a readable format.
pub fn display_mutants_on_screen(
    path_to_report: impl AsRef<Path>,
    modules: &ModuleFilter,
    functions: &FunctionFilter,
    mutant_filter: &MutantFilter,
) -> Result<()> {
    let report = Report::load_from_json_file(path_to_report.as_ref())?;
    let files_to_print = modules.get_all_files_containing_the_modules(&report);
    let Report { mut files, .. } = report;

    if files_to_print.is_empty() {
        println!("No matching files found.");
        return Ok(());
    };

    let mut all_mutant_stats = Vec::<MutantStats>::new();
    for file in files_to_print {
        if let Some(mut file_mutant_stats) = files.remove(&file) {
            if let FunctionFilter::Selected(filtered_funcs) = functions {
                file_mutant_stats.retain(|m| {
                    let (_, func) = m
                        .module_func
                        .split_once("::")
                        .expect("invalid function signature in the report file");
                    filtered_funcs.contains(&func.to_owned())
                });
            }
            all_mutant_stats.extend(file_mutant_stats);
        }
    }

    if all_mutant_stats.is_empty() {
        println!("No matching functions found.");
        return Ok(());
    };

    let f = PatchFormatter::new().with_color();
    for mutant in all_mutant_stats {
        if mutant_filter.contains_alive() {
            for diff in mutant.mutants_alive_diffs {
                println!("----------------------------------------------------------------------------------------------------");
                println!("{}: Alive mutant", mutant.module_func);
                let patch = Patch::from_str(&diff).expect("invalid patch");
                println!("{}", f.fmt_patch(&patch));
            }
        }

        if mutant_filter.contains_killed() {
            for diff in mutant.mutants_killed_diff {
                println!("----------------------------------------------------------------------------------------------------");
                println!("{}: Killed mutant", mutant.module_func);
                let patch = Patch::from_str(&diff).expect("invalid patch");
                println!("{}", f.fmt_patch(&patch));
            }
        }

        println!(); // Add one empty line
    }

    Ok(())
}

/// Summarize the report.
pub fn display_summary(path_to_report: impl AsRef<Path>) -> Result<()> {
    let report = Report::load_from_json_file(path_to_report.as_ref())?;
    report.print_table();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn reading_report_from_file_works() {
        let package_dir = tempfile::tempdir().unwrap().into_path();

        let mut report = Report::new(package_dir.clone());
        let path1 = package_dir.join("src_file1");
        let path2 = package_dir.join("src_file2");
        let module_name = "module";
        report.increment_mutants_tested(&path1, module_name);
        report.increment_mutants_tested(&path2, module_name);

        let report_path = package_dir.join("report.txt");
        report
            .save_to_json_file(&report_path)
            .expect("failed to save the file to a disk");

        // Files also need to exist.
        fs::File::create(path1).unwrap();
        fs::File::create(path2).unwrap();

        let modules = ModuleFilter::All;
        let ret = display_coverage_on_screen(&report_path, &modules);
        assert!(ret.is_ok());

        let functions = FunctionFilter::All;
        let mutant_filter = MutantFilter::All;
        let ret = display_mutants_on_screen(&report_path, &modules, &functions, &mutant_filter);
        assert!(ret.is_ok());

        let ret = display_summary(report_path);
        assert!(ret.is_ok());
    }

    #[test]
    fn report_file_not_found() {
        let path = PathBuf::from("/path/to/non/existing/file");
        let modules = ModuleFilter::All;
        let ret = display_coverage_on_screen(&path, &modules);
        assert!(ret.is_err());

        let functions = FunctionFilter::All;
        let mutant_filter = MutantFilter::Alive;
        let ret = display_mutants_on_screen(&path, &modules, &functions, &mutant_filter);
        assert!(ret.is_err());

        let ret = display_summary(path);
        assert!(ret.is_err());
    }
}
