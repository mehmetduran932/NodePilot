//! Atomic archive extraction and Node runtime installation.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use tar::Archive as TarArchive;
use zip::ZipArchive;
use nodepilot_core::{NodePilotError, NodePilotPaths};

/// Installs a verified downloaded Node.js archive into `paths.versions_dir/{version}` atomically.
pub fn install_node_archive(
    paths: &NodePilotPaths,
    archive_path: &Path,
    clean_version: &str,
) -> Result<PathBuf, NodePilotError> {
    let target_version_dir = paths.version_dir(clean_version);

    if target_version_dir.is_dir() {
        // Already installed
        return Ok(target_version_dir);
    }

    // Temporary staging directory
    let temp_staging_dir = paths.versions_dir.join(format!(".tmp_{}_{}", clean_version, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
    std::fs::create_dir_all(&temp_staging_dir)?;

    let extract_res = if archive_path.to_string_lossy().ends_with(".zip") {
        extract_zip(archive_path, &temp_staging_dir)
    } else {
        extract_tar_gz(archive_path, &temp_staging_dir)
    };

    if let Err(e) = extract_res {
        let _ = std::fs::remove_dir_all(&temp_staging_dir);
        return Err(e);
    }

    // Node archives extract into a root subdirectory: e.g. "node-v22.18.0-win-x64"
    // Find that directory and move its contents, or rename it
    let extracted_root = find_extracted_root(&temp_staging_dir)?;

    // Atomic move to final target directory
    if let Err(e) = std::fs::rename(&extracted_root, &target_version_dir) {
        let _ = std::fs::remove_dir_all(&temp_staging_dir);
        return Err(NodePilotError::Extraction(format!(
            "Failed to move installed runtime to {}: {}",
            target_version_dir.display(),
            e
        )));
    }

    // Clean up temporary parent if empty
    let _ = std::fs::remove_dir_all(&temp_staging_dir);

    // Verify binary exists
    let bin_path = paths.node_binary_path(clean_version);
    if !bin_path.is_file() {
        let _ = std::fs::remove_dir_all(&target_version_dir);
        return Err(NodePilotError::Extraction(format!(
            "Installation completed but node binary not found at {}",
            bin_path.display()
        )));
    }

    Ok(target_version_dir)
}

/// Uninstalls an installed Node.js version.
pub fn uninstall_node_version(paths: &NodePilotPaths, version: &str) -> Result<(), NodePilotError> {
    let clean = version.trim_start_matches('v');
    let target = paths.version_dir(clean);
    if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }
    Ok(())
}

fn extract_zip(archive_path: &Path, target_dir: &Path) -> Result<(), NodePilotError> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut zip = ZipArchive::new(reader).map_err(|e| NodePilotError::Extraction(e.to_string()))?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| NodePilotError::Extraction(e.to_string()))?;
        let enclosed_name = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => {
                return Err(NodePilotError::SecurityTraversal(format!(
                    "Malicious relative path detected: {}",
                    file.name()
                )));
            }
        };

        let outpath = target_dir.join(enclosed_name);

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

fn extract_tar_gz(archive_path: &Path, target_dir: &Path) -> Result<(), NodePilotError> {
    let file = File::open(archive_path)?;
    let tar = GzDecoder::new(file);
    let mut archive = TarArchive::new(tar);

    for entry in archive.entries().map_err(|e| NodePilotError::Extraction(e.to_string()))? {
        let mut entry = entry.map_err(|e| NodePilotError::Extraction(e.to_string()))?;
        let path = entry.path().map_err(|e| NodePilotError::Extraction(e.to_string()))?;

        // Security check for path traversal
        for component in path.components() {
            if let std::path::Component::ParentDir = component {
                return Err(NodePilotError::SecurityTraversal(format!(
                    "Tar entry contains parent dir: {}",
                    path.display()
                )));
            }
        }

        entry.unpack_in(target_dir).map_err(|e| NodePilotError::Extraction(e.to_string()))?;
    }

    Ok(())
}

fn find_extracted_root(staging_dir: &Path) -> Result<PathBuf, NodePilotError> {
    let mut entries = std::fs::read_dir(staging_dir)?;
    if let Some(first) = entries.next() {
        let entry = first?;
        if entry.file_type()?.is_dir() {
            return Ok(entry.path());
        }
    }
    Ok(staging_dir.to_path_buf())
}
