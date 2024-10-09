use crate::compiler::compile_package;
use anyhow::Error;
use codespan::Span;
use move_command_line_common::files::FileHash;
use move_compiler::compiled_unit::{CompiledUnit, NamedCompiledModule};
use move_coverage::{
    coverage_map::CoverageMap,
    source_coverage::{FunctionSourceCoverage, SourceCoverageBuilder},
};
use move_model::model::Loc;
use move_package::BuildConfig;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

/// Associated function string.
type AssociatedFuncStr = String;

/// Contains all uncovered spans in the project.
#[derive(Debug, Default)]
pub(crate) struct Coverage {
    /// List of all uncovered spans for all functions for all modules.
    all_uncovered_spans: BTreeMap<AssociatedFuncStr, UncoveredSpans>,
}

impl Coverage {
    /// Compute coverage for the project.
    pub(crate) fn compute_coverage(
        &mut self,
        build_config: &BuildConfig,
        package_path: &Path,
    ) -> anyhow::Result<()> {
        info!("computing coverage");

        let coverage_map = CoverageMap::from_binary_file(package_path.join(".coverage_map.mvcov"))
            .map_err(|e| Error::msg(format!("failed to retrieve the coverage map: {e}")))?;

        let mut coverage_config = build_config.clone();
        coverage_config.test_mode = false;
        let package = compile_package(coverage_config, package_path)?;

        // We might fetch the same sources multiple times per module, so let's store only one instance.
        let mut sources = HashMap::<&PathBuf, String>::new();

        let mut modules_and_sources = Vec::<_>::new();
        for unit in package.root_modules() {
            if let CompiledUnit::Module(NamedCompiledModule {
                module, source_map, ..
            }) = &unit.unit
            {
                let src_path = &unit.source_path;

                // Both below cases are very unlikely since this happens just after we compile the package:
                let file_contents =
                    sources
                        .entry(src_path)
                        .or_insert(fs::read_to_string(src_path).map_err(|e| {
                            Error::msg(format!(
                                "source code removed during the tool execution: {e}"
                            ))
                        })?);
                if !source_map.check(file_contents) {
                    anyhow::bail!("source code changed during the execution");
                }

                modules_and_sources.push((module, source_map, src_path));
            }
        }

        let mut all_uncovered_spans = BTreeMap::new();
        modules_and_sources
            .into_iter()
            .map(|(module, source_map, src_path)| {
                // This `new` function in `SourceCoverageBuilder` calculates uncovered locations.
                let scb = SourceCoverageBuilder::new(module, &coverage_map, source_map);

                (module, scb.uncovered_locations, src_path)
            })
            .for_each(|(module, uncovered_locations, src_path)| {
                for (identifer, func_source_coverage) in uncovered_locations {
                    let first_location = func_source_coverage.uncovered_locations.first();

                    // If not empty, get the uncovered spans for the function.
                    let uncovered_spans = if let Some(loc) = first_location {
                        // Can't fail since we already inserted this value earlier above.
                        let file_contents = sources.get(&src_path).unwrap();

                        let file_hash = loc.file_hash();
                        UncoveredSpans::new(func_source_coverage, file_hash, file_contents)
                    } else {
                        continue;
                    };

                    let module = module.self_name();
                    let function = identifer.into_string();
                    let associated_fn_name = format!("{module}::{function}");
                    all_uncovered_spans.insert(associated_fn_name, uncovered_spans);
                }
            });

        trace!("all uncovered spans: {all_uncovered_spans:?}");
        self.all_uncovered_spans = all_uncovered_spans;
        Ok(())
    }

    /// Check if the location is covered by the unit test.
    pub(crate) fn check_location(&self, associated_fn_name: String, loc: &Loc) -> bool {
        let span = loc.span();

        let Some(UncoveredSpans(spans)) = self.all_uncovered_spans.get(&associated_fn_name) else {
            trace!("location has coverage since {associated_fn_name} has full coverage");
            return true;
        };

        for uncovered_span in spans {
            // Skip all early spans.
            if span.start() >= uncovered_span.end() {
                continue;
            }

            // Check if the span starts earlier than the uncovered span.
            if uncovered_span.start() > span.start() {
                break;
            }

            // If the span goes beyond the uncovered span, we are done here.
            if uncovered_span.end() < span.end() {
                break;
            }

            trace!("{associated_fn_name} has no coverage for the given location");
            return false;
        }

        trace!("{associated_fn_name} has coverage for the given location");
        true
    }
}

#[derive(Debug)]
struct UncoveredSpans(Vec<Span>);

impl UncoveredSpans {
    /// Create a new [`UncoveredSpans`].
    fn new(func_src_cov: FunctionSourceCoverage, file_hash: FileHash, file_contents: &str) -> Self {
        let merged_spans = merge_spans(file_hash, func_src_cov);
        let optimized_spans = merge_spans_after_removing_whitespaces(merged_spans, file_contents);
        Self(optimized_spans)
    }
}

// There is a similar function in aptos-core with the same name, but that one doesn't merge
// spans that are next to each other (e.g. spans (0..5] and (5..10]) - but that is not
// noticeable in their usage for the coverage, so nobody noticed this. We can make an update
// there maybe and then use a more efficient `merge_spans` function from aptos-core repo.
/// Efficiently merge spans.
fn merge_spans(file_hash: FileHash, cov: FunctionSourceCoverage) -> Vec<Span> {
    cov.uncovered_locations
        .iter()
        .filter(|loc| loc.file_hash() == file_hash)
        .map(|loc| Span::new(loc.start(), loc.end()))
        .collect::<Vec<_>>()
        .windows(2)
        .fold(vec![], |mut acc, spans| {
            let [curr, next] = spans else { return acc };
            if curr.end() >= next.start() {
                acc.push(curr.merge(*next));
            } else {
                acc.push(*curr);
            }
            acc
        })
}

/// Remove all whitespaces between spans and merge spans again.
fn merge_spans_after_removing_whitespaces(mut spans: Vec<Span>, source_code: &str) -> Vec<Span> {
    let mut new_spans = Vec::with_capacity(spans.len());
    let mut curr = spans.remove(0);

    'span_loop: for span in spans {
        let mut curr_end_index = curr.end().to_usize();

        let src_chars = source_code[curr_end_index..].chars();
        for next_char in src_chars {
            if next_char != ' ' {
                break;
            }

            // We can safely assume the ' ' char will always have the size of one.
            curr_end_index += 1;

            // If have whitespaces between these two uncovered spans, let's merge those.
            if curr_end_index == span.start().to_usize() {
                curr = curr.merge(span);
                continue 'span_loop;
            }
        }

        new_spans.push(curr);
        curr = span;
    }

    new_spans.push(curr);
    new_spans
}
