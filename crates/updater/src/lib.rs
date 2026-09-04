//! Application update checker and package manager upgrade assistant.

use serde::{Deserialize, Serialize};
use nodepilot_core::NodePilotError;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/nodepilot/nodepilot/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
    pub package_manager_command: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: String,
}

/// Checks GitHub releases API for newer NodePilot version.
pub async fn check_for_updates() -> Result<UpdateCheckResult, NodePilotError> {
    let client = reqwest::Client::builder()
        .user_agent("NodePilot-Updater")
        .build()
        .map_err(|e| NodePilotError::Execution(e.to_string()))?;

    let res = client
        .get(GITHUB_RELEASES_API)
        .send()
        .await;

    let (latest_ver, notes, url) = match res {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(gh) = resp.json::<GitHubRelease>().await {
                let clean = gh.tag_name.trim_start_matches('v').to_string();
                (clean, gh.body, Some(gh.html_url))
            } else {
                (CURRENT_VERSION.to_string(), None, None)
            }
        }
        _ => (CURRENT_VERSION.to_string(), None, None),
    };

    let update_available = if let (Ok(curr), Ok(latest)) = (
        semver::Version::parse(CURRENT_VERSION),
        semver::Version::parse(&latest_ver),
    ) {
        latest > curr
    } else {
        false
    };

    #[cfg(target_os = "windows")]
    let pm_cmd = "winget upgrade NodePilot.NodePilot".to_string();
    #[cfg(not(target_os = "windows"))]
    let pm_cmd = "brew upgrade --cask nodepilot".to_string();

    Ok(UpdateCheckResult {
        update_available,
        current_version: CURRENT_VERSION.to_string(),
        latest_version: latest_ver,
        release_notes: notes,
        download_url: url,
        package_manager_command: pm_cmd,
    })
}
