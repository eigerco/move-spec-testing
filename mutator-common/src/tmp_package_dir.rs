//! A path setup container for packages under test.
// Copyright © Eiger
// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use fs_extra::dir::CopyOptions;
use log::{info, trace};
use move_package::source_package::{layout::SourcePackageLayout, manifest_parser};
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
    fs_extra::dir::copy(&package_path, &new_package_path, &options)?;

    rewrite_manifest_to_use_abs_paths(&package_path, &new_package_path)?;

    // Since the tool will copy the original package often, remove unnecessary files.
    let remove_item = |item_name: &str| {
        let item_path = new_package_path.join(item_name);
        let result = if item_path.is_dir() {
            fs::remove_dir_all(&item_path)
        } else {
            fs::remove_file(&item_path)
        };
        if result.is_ok() {
            trace!("removed {}", item_path.display());
        }
    };
    [
        "build",
        "doc",
        ".trace",
        "output.bpl",
        "doc_template",
        "report.txt",
    ]
    .iter()
    .for_each(|&dir| remove_item(dir));

    Ok((outdir, new_package_path))
}

/// Helper method to strip the temp dir prefix and keep only the `sources/xxx.move` path.
pub fn strip_path_prefix<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let original_file = path.as_ref().to_string_lossy();

    // This should always work.
    let sources_dir_idx = original_file
        .find(ORIGINAL_PACKAGE_PATH).ok_or(anyhow::Error::msg("original package path not found"))?
        + ORIGINAL_PACKAGE_PATH.len() // skip package path.
        + 1; // skip the `/` character before 'sources'.

    Ok(PathBuf::from(&original_file[sources_dir_idx..]))
}

/// Rewrite the manifest file to use absolute paths.
///
/// # Arguments
///
/// * `root` - the path to the package root.
/// * `tempdir` - the path to the temporary directory.
///
/// # Errors
///
/// * If any error occurs during the rewrite, the appropriate error is returned using anyhow.
///
/// # Panics
///
/// This function panics if dependency paths contain no Unicode characters.
///
/// # Returns
///
/// * `Result<(), anyhow::Error>` - Ok if the rewrite is successful, or an error if any error occurs.
fn rewrite_manifest_to_use_abs_paths(root: &Path, tempdir: &Path) -> Result<(), anyhow::Error> {
    let mut manifest_string = fs::read_to_string(root.join(SourcePackageLayout::Manifest.path()))?;
    let manifest = manifest_parser::parse_move_manifest_string(manifest_string.clone())?;
    let manifest = manifest_parser::parse_source_manifest(manifest)?;
    let curdir = std::env::current_dir()?;

    // We need to switch to package dir as paths in manifest are relative to package dir.
    std::env::set_current_dir(root)?;

    manifest
        .dependencies
        .values()
        .chain(manifest.dev_dependencies.values())
        .for_each(|dep| {
            let dep_canon = dep.local.canonicalize();
            if let Ok(dep_canon) = dep_canon {
                manifest_string = manifest_string
                    .replace(dep.local.to_str().unwrap(), dep_canon.to_str().unwrap());
            }
        });

    // Switch back to the original dir.
    std::env::set_current_dir(curdir)?;
    fs::write(
        tempdir.join(SourcePackageLayout::Manifest.path()),
        manifest_string,
    )?;
    Ok(())
}
