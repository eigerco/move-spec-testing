// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{cli::CLIOptions, coverage::Coverage};
use std::path::PathBuf;

/// Mutator configuration for the Move project.
#[derive(Debug, Default)]
pub struct Configuration {
    /// Main project options. It's the same as the CLI options.
    pub project: CLIOptions,
    /// Path to the project.
    pub project_path: Option<PathBuf>,
    /// Coverage report where the optional unit test coverage data is stored.
    pub(crate) coverage: Coverage,
}

impl Configuration {
    /// Creates a new configuration using command line options.
    #[must_use]
    pub fn new(project: CLIOptions, project_path: Option<PathBuf>) -> Self {
        Self {
            project,
            project_path,
            // Coverage is disabled by default.
            coverage: Coverage::default(),
        }
    }
}
