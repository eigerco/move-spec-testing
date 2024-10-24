//! A path setup container for packages under test.
// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use fs_extra::dir::CopyOptions;
use log::{info, trace};
use move_package::source_package::layout::SourcePackageLayout;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The path in temp dir where the original Move package is cloned.
const ORIGINAL_PACKAGE_PATH: &str = "original_package";

/// Returns the output directory and a recreated package path.
pub fn setup_outdir_and_package_path<P: AsRef<Path>>(
    package_path: P,
) -> Result<(PathBuf, PathBuf)> {
    // Check if the package is correctly structured.
    let package_path = SourcePackageLayout::try_find_root(&package_path.as_ref().canonicalize()?)?;
    info!("Found package path: {package_path:?}");

    let outdir = tempfile::tempdir()?.into_path();
    let new_package_path = outdir.join(ORIGINAL_PACKAGE_PATH);
    fs::create_dir_all(&new_package_path)?;

    let options = CopyOptions::new().content_only(true);
    fs_extra::dir::copy(package_path, &new_package_path, &options)?;

    let remove_dir = |dir_name| {
        let dir = new_package_path.join(dir_name);
        if fs::remove_dir_all(&dir).is_ok() {
            trace!("removing {}", dir.display());
        }
    };
    let remove_file = |dir_name| {
        let dir = new_package_path.join(dir_name);
        if fs::remove_file(&dir).is_ok() {
            trace!("removing {}", dir.display());
        }
    };

    // Since the tool will copy the original package often, remove unnecessary files.
    remove_dir("build");
    remove_dir("doc");
    remove_file(".trace");

    Ok((outdir, new_package_path))
}

/// Helper method to strip the temp dir prefix and keep only the `sources/xxx.move` path.
pub fn strip_path_prefix<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let original_file = path.as_ref().to_string_lossy().into_owned();

    // This should always work.
    let sources_dir_idx = original_file
        .find(ORIGINAL_PACKAGE_PATH).ok_or(anyhow::Error::msg("original package path not found"))?
        + ORIGINAL_PACKAGE_PATH.len() // skip package path.
        + 1; // skip the `/` character before 'sources'.

    Ok(PathBuf::from(&original_file[sources_dir_idx..]))
}
