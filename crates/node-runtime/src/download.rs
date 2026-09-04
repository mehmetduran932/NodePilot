//! Download and cryptographic SHA256 checksum verification for Node.js distribution archives.

use std::path::PathBuf;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use nodepilot_core::{NodePilotError, NodePilotPaths};
use crate::release::get_archive_filename;

const NODE_DIST_BASE: &str = "https://nodejs.org/dist";

pub struct DownloadedArchive {
    pub archive_path: PathBuf,
    pub filename: String,
    pub version: String,
}

/// Fetches the official SHASUMS256.txt file for a specific Node version.
pub async fn fetch_checksums(client: &reqwest::Client, version: &str) -> Result<String, NodePilotError> {
    let url = format!("{}/v{}/SHASUMS256.txt", NODE_DIST_BASE, version);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| NodePilotError::Download {
            url: url.clone(),
            reason: format!("Failed to fetch checksums: {}", e),
        })?;

    if !resp.status().is_success() {
        return Err(NodePilotError::Download {
            url,
            reason: format!("HTTP {}", resp.status()),
        });
    }

    resp.text().await.map_err(|e| NodePilotError::Download {
        url: "SHASUMS256.txt".into(),
        reason: format!("Failed to read checksum body: {}", e),
    })
}

/// Parses the expected SHA256 hex string for a given archive filename from SHASUMS256.txt.
pub fn parse_expected_checksum(checksums_txt: &str, filename: &str) -> Result<String, NodePilotError> {
    for line in checksums_txt.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == filename {
            return Ok(parts[0].to_lowercase());
        }
    }
    Err(NodePilotError::Download {
        url: filename.to_string(),
        reason: format!("Entry for '{}' not found in official SHASUMS256.txt", filename),
    })
}

/// Downloads a Node.js archive, verifies its SHA256 checksum, and saves it in `cache_dir`.
pub async fn download_and_verify_archive<F>(
    paths: &NodePilotPaths,
    clean_version: &str,
    progress_callback: Option<F>,
) -> Result<DownloadedArchive, NodePilotError>
where
    F: Fn(u64, Option<u64>) + Send + Sync + 'static,
{
    let filename = get_archive_filename(clean_version)?;
    let archive_url = format!("{}/v{}/{}", NODE_DIST_BASE, clean_version, filename);
    let target_cache_path = paths.cache_dir.join(&filename);

    let client = reqwest::Client::builder()
        .user_agent("NodePilot-Agent")
        .build()
        .map_err(|e| NodePilotError::Download {
            url: archive_url.clone(),
            reason: e.to_string(),
        })?;

    // 1. Fetch official checksum
    let checksum_txt = fetch_checksums(&client, clean_version).await?;
    let expected_hash = parse_expected_checksum(&checksum_txt, &filename)?;

    // If cached archive already exists and matches hash, reuse it
    if target_cache_path.is_file() {
        if let Ok(bytes) = std::fs::read(&target_cache_path) {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let calculated = hex::encode(hasher.finalize());
            if calculated == expected_hash {
                return Ok(DownloadedArchive {
                    archive_path: target_cache_path,
                    filename,
                    version: clean_version.to_string(),
                });
            }
        }
    }

    // 2. Stream download to temporary file
    let temp_cache_path = paths.cache_dir.join(format!("{}.download", filename));
    if let Some(parent) = temp_cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut response = client
        .get(&archive_url)
        .send()
        .await
        .map_err(|e| NodePilotError::Download {
            url: archive_url.clone(),
            reason: e.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(NodePilotError::Download {
            url: archive_url,
            reason: format!("HTTP status {}", response.status()),
        });
    }

    let total_size = response.content_length();
    let mut downloaded: u64 = 0;
    let mut hasher = Sha256::new();
    let mut file = tokio::fs::File::create(&temp_cache_path).await?;

    while let Some(chunk) = response.chunk().await.map_err(|e| NodePilotError::Download {
        url: archive_url.clone(),
        reason: format!("Stream error: {}", e),
    })? {
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if let Some(ref cb) = progress_callback {
            cb(downloaded, total_size);
        }
    }

    file.flush().await?;
    drop(file);

    // 3. Verify SHA256 checksum
    let actual_hash = hex::encode(hasher.finalize());
    if actual_hash != expected_hash {
        let _ = tokio::fs::remove_file(&temp_cache_path).await;
        return Err(NodePilotError::ChecksumMismatch {
            file: filename,
            expected: expected_hash,
            actual: actual_hash,
        });
    }

    // Rename temp file to target cache path atomically
    std::fs::rename(&temp_cache_path, &target_cache_path)?;

    Ok(DownloadedArchive {
        archive_path: target_cache_path,
        filename,
        version: clean_version.to_string(),
    })
}
