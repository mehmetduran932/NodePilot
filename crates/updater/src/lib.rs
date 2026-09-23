//! Application update checker and self-updater.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use nodepilot_core::{NodePilotError, NodePilotPaths};

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/mehmetduran932/NodePilot/releases/latest";

/// Binaries shipped in each release archive.
const RELEASE_BINARIES: &[&str] = &["nodepilot", "nodepilot-shim"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    /// Direct download URL of this platform's release archive, if the release has one.
    pub asset_url: Option<String>,
    pub package_manager_command: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: String,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Release archive name for the platform this binary was built for.
pub fn platform_asset_name() -> Option<&'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some("nodepilot-windows-x64.zip")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("nodepilot-macos-arm64.tar.gz")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some("nodepilot-macos-x64.tar.gz")
    } else {
        None
    }
}

fn http_client() -> Result<reqwest::Client, NodePilotError> {
    reqwest::Client::builder()
        .user_agent("NodePilot-Updater")
        .build()
        .map_err(|e| NodePilotError::Execution(e.to_string()))
}

/// Checks GitHub releases API for newer NodePilot version.
pub async fn check_for_updates() -> Result<UpdateCheckResult, NodePilotError> {
    let client = http_client()?;

    let res = client
        .get(GITHUB_RELEASES_API)
        .send()
        .await;

    let (latest_ver, notes, url, asset_url) = match res {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(gh) = resp.json::<GitHubRelease>().await {
                let clean = gh.tag_name.trim_start_matches('v').to_string();
                let asset_url = platform_asset_name().and_then(|name| {
                    gh.assets
                        .iter()
                        .find(|a| a.name == name)
                        .map(|a| a.browser_download_url.clone())
                });
                (clean, gh.body, Some(gh.html_url), asset_url)
            } else {
                (CURRENT_VERSION.to_string(), None, None, None)
            }
        }
        _ => (CURRENT_VERSION.to_string(), None, None, None),
    };

    let update_available = if let (Ok(curr), Ok(latest)) = (
        semver::Version::parse(CURRENT_VERSION),
        semver::Version::parse(&latest_ver),
    ) {
        latest > curr
    } else {
        false
    };

    Ok(UpdateCheckResult {
        update_available,
        current_version: CURRENT_VERSION.to_string(),
        latest_version: latest_ver,
        release_notes: notes,
        download_url: url,
        asset_url,
        package_manager_command: "nodepilot update".to_string(),
    })
}

/// Downloads the release archive for this platform and replaces the NodePilot binaries in `paths.bin_dir`.
pub async fn apply_update(paths: &NodePilotPaths, check: &UpdateCheckResult) -> Result<(), NodePilotError> {
    let asset_url = check.asset_url.as_deref().ok_or_else(|| {
        NodePilotError::InvalidOperation(format!(
            "Release v{} has no archive for this platform",
            check.latest_version
        ))
    })?;
    let archive_name = platform_asset_name().unwrap_or("nodepilot-update.tar.gz");

    let staging = paths.cache_dir.join(format!("update-{}", check.latest_version));
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;

    let result = download_and_install(paths, asset_url, &staging, archive_name).await;
    let _ = std::fs::remove_dir_all(&staging);
    result
}

async fn download_and_install(
    paths: &NodePilotPaths,
    asset_url: &str,
    staging: &Path,
    archive_name: &str,
) -> Result<(), NodePilotError> {
    let resp = http_client()?
        .get(asset_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| NodePilotError::ReleaseFetch(format!("Download failed: {}", e)))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| NodePilotError::ReleaseFetch(format!("Download failed: {}", e)))?;

    let archive_path = staging.join(archive_name);
    std::fs::write(&archive_path, &bytes)?;
    let extract_dir = staging.join("extracted");
    std::fs::create_dir_all(&extract_dir)?;
    nodepilot_node_runtime::extract_archive(&archive_path, &extract_dir)?;

    // Verify every binary is present before touching the installed ones
    let mut replacements = Vec::new();
    for bin in RELEASE_BINARIES {
        let file_name = if cfg!(windows) { format!("{}.exe", bin) } else { bin.to_string() };
        let src = find_file(&extract_dir, &file_name).ok_or_else(|| {
            NodePilotError::Extraction(format!("{} not found in release archive", file_name))
        })?;
        replacements.push((src, paths.bin_dir.join(&file_name)));
    }

    std::fs::create_dir_all(&paths.bin_dir)?;
    for (src, dest) in replacements {
        replace_binary(&src, &dest)?;
    }
    Ok(())
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let direct = dir.join(name);
    if direct.is_file() {
        return Some(direct);
    }
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .find_map(|sub| Some(sub.join(name)).filter(|p| p.is_file()))
}

/// Replaces `dest` with `src`, also when `dest` is the currently running executable.
fn replace_binary(src: &Path, dest: &Path) -> Result<(), NodePilotError> {
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        // Copy next to the target, then atomically rename over it (safe for a running binary)
        let tmp = dest.with_extension("new");
        std::fs::copy(src, &tmp)?;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
        std::fs::rename(&tmp, dest)?;
    }

    #[cfg(target_os = "windows")]
    {
        // A running .exe cannot be overwritten but can be renamed out of the way
        if dest.exists() {
            let old = dest.with_extension("exe.old");
            let _ = std::fs::remove_file(&old);
            std::fs::rename(dest, &old)?;
        }
        std::fs::copy(src, dest)?;
    }

    Ok(())
}

/// Removes `*.exe.old` leftovers from a previous Windows self-update.
pub fn cleanup_previous_update(paths: &NodePilotPaths) {
    if cfg!(windows) {
        for bin in RELEASE_BINARIES {
            let _ = std::fs::remove_file(paths.bin_dir.join(format!("{}.exe.old", bin)));
        }
    }
}
