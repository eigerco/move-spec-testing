// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use log::info;
use std::time::{Duration, Instant};

/// A benchmark for a specific operation.
#[derive(Debug, Clone)]
pub struct Benchmark {
    /// Start time of the operation.
    pub start_time: Instant,
    /// Duration of the operation.
    pub elapsed: Duration,
}

impl Default for Benchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl Benchmark {
    /// Creates a new benchmark.
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            elapsed: Duration::new(0, 0),
        }
    }

    /// Starts the benchmark.
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }

    /// Stops the benchmark.
    pub fn stop(&mut self) {
        self.elapsed = self.start_time.elapsed();
    }
}

/// A collection of benchmarks for the tool.
#[derive(Default)]
pub struct Benchmarks {
    /// Total time for the whole tool to complete.
    pub total_tool_duration: Benchmark,
    /// Benchmark for the test execution on all mutants.
    pub executing_original_package: Benchmark,
    /// Benchmark for the mutator.
    pub mutator: Benchmark,
    /// Benchmark for the test execution on all mutants.
    pub executing_tests_on_mutants: Benchmark,
    /// Benchmark for the each mutant.
    pub mutant_results: Vec<Benchmark>,
}

impl Benchmarks {
    /// Creates a new collection of benchmarks.
    pub fn new() -> Self {
        Self {
            total_tool_duration: Benchmark::new(),
            executing_original_package: Benchmark::new(),
            mutator: Benchmark::new(),
            executing_tests_on_mutants: Benchmark::new(),
            mutant_results: Vec::new(),
        }
    }

    /// Displays the benchmarks with the `RUST_LOG` info level.
    pub fn display(&self) {
        info!(
            "Test execution time on the original package is {} msecs",
            self.executing_original_package.elapsed.as_millis()
        );
        info!(
            "Generating mutants took {} msecs",
            self.mutator.elapsed.as_millis()
        );
        info!(
            "Executing the tool on all mutants took {} msecs",
            self.executing_tests_on_mutants.elapsed.as_millis()
        );
        if !self.mutant_results.is_empty() {
            info!(
                "Min execution time for a mutant: {} msecs",
                self.mutant_results
                    .iter()
                    .map(|f| f.elapsed.as_millis())
                    .min()
                    .unwrap()
            );
            info!(
                "Max execution time for a mutant: {} msecs",
                self.mutant_results
                    .iter()
                    .map(|f| f.elapsed.as_millis())
                    .max()
                    .unwrap()
            );
            info!(
                "Average execution time for each mutant: {:.2} msecs",
                self.executing_tests_on_mutants.elapsed.as_millis() as f64
                    / self.mutant_results.len() as f64
            );
        }
        info!(
            "Total tool execution time is {} msecs",
            self.total_tool_duration.elapsed.as_millis()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    fn benchmark_records_correct_elapsed_time() {
        let mut benchmark = Benchmark::new();
        benchmark.start();
        thread::sleep(Duration::from_millis(100));
        benchmark.stop();
        assert!(benchmark.elapsed >= Duration::from_millis(100));
    }

    #[test]
    fn benchmark_does_not_record_time_before_start() {
        let mut benchmark = Benchmark::new();
        thread::sleep(Duration::from_millis(100));
        benchmark.start();
        thread::sleep(Duration::from_millis(100));
        benchmark.stop();
        assert!(benchmark.elapsed < Duration::from_millis(200));
    }

    #[test]
    fn benchmark_does_not_record_time_after_stop() {
        let mut benchmark = Benchmark::new();
        benchmark.start();
        thread::sleep(Duration::from_millis(100));
        benchmark.stop();
        thread::sleep(Duration::from_millis(100));
        assert!(benchmark.elapsed < Duration::from_millis(200));
    }

    #[test]
    fn benchmarks_records_multiple_benchmarks() {
        const TEN: u64 = 10;

        let mut benchmarks = Benchmarks {
            total_tool_duration: Benchmark::new(),
            executing_original_package: Benchmark::new(),
            mutator: Benchmark::new(),
            executing_tests_on_mutants: Benchmark::new(),
            mutant_results: Vec::new(),
        };

        benchmarks.total_tool_duration.start();
        thread::sleep(Duration::from_millis(TEN));
        benchmarks.total_tool_duration.stop();

        benchmarks.executing_original_package.start();
        thread::sleep(Duration::from_millis(TEN));
        benchmarks.executing_original_package.stop();

        benchmarks.mutator.start();
        thread::sleep(Duration::from_millis(TEN));
        benchmarks.mutator.stop();

        benchmarks.executing_tests_on_mutants.start();
        thread::sleep(Duration::from_millis(TEN));
        benchmarks.executing_tests_on_mutants.stop();

        assert!(benchmarks.total_tool_duration.elapsed >= Duration::from_millis(TEN));
        assert!(benchmarks.executing_original_package.elapsed >= Duration::from_millis(TEN));
        assert!(benchmarks.mutator.elapsed >= Duration::from_millis(TEN));
        assert!(benchmarks.executing_tests_on_mutants.elapsed >= Duration::from_millis(TEN));
    }
}
