//! Tauri 2 desktop backend for NodePilot.

use std::path::PathBuf;
use tauri::command;
use nodepilot_core::{
    ensure_gitignore_entry, inspect_environment, resolve_project_node,
    AppState, EnvironmentReport, NodePilotPaths, ProjectRecord, Settings,
    get_integration_status, enable_integration, disable_integration, IntegrationStatus,
    LOCAL_METADATA_FILENAME,
};
use nodepilot_node_runtime::{
    download_and_verify_archive, fetch_releases, install_node_archive,
    is_version_installed, list_installed_versions, resolve_version_alias,
    uninstall_node_version, InstalledVersion, NodeRelease,
};
use nodepilot_scanner::scan_workspace;
use nodepilot_compatibility::{
    calculate_auto_assign_diff, evaluate_compatibility, AutoAssignProposal,
};
use nodepilot_updater::{check_for_updates, UpdateCheckResult};

// -------------------------------- State & Workspaces --------------------------------

#[command]
fn get_state() -> Result<AppState, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let state = AppState::load(&paths.state_file()).map_err(|e| e.to_string())?;
    Ok(state)
}

#[command]
fn get_settings() -> Result<Settings, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let settings = Settings::load(&paths.settings_file()).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[command]
fn update_settings(settings: Settings) -> Result<(), String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    settings.save(&paths.settings_file()).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
fn add_workspace(path: String) -> Result<AppState, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let mut state = AppState::load(&paths.state_file()).map_err(|e| e.to_string())?;
    let pb = PathBuf::from(&path);
    if state.add_workspace(pb) {
        state.save(&paths.state_file()).map_err(|e| e.to_string())?;
    }
    Ok(state)
}

#[command]
fn remove_workspace(path: String) -> Result<AppState, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let mut state = AppState::load(&paths.state_file()).map_err(|e| e.to_string())?;
    let pb = PathBuf::from(&path);
    if state.remove_workspace(&pb) {
        state.save(&paths.state_file()).map_err(|e| e.to_string())?;
    }
    Ok(state)
}

#[command]
async fn scan_workspaces() -> Result<Vec<ProjectRecord>, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
    let mut state = AppState::load(&paths.state_file()).map_err(|e| e.to_string())?;

    let mut all_projects = Vec::new();
    for ws in &state.workspaces {
        if ws.is_dir() {
            if let Ok(projects) = scan_workspace(ws, settings.global_default_version.as_deref(), 8) {
                all_projects.extend(projects);
            }
        }
    }

    all_projects.sort_by(|a, b| a.name.cmp(&b.name));
    state.projects = all_projects.clone();
    state.last_scan_time = Some(chrono::Utc::now());
    state.save(&paths.state_file()).map_err(|e| e.to_string())?;

    Ok(all_projects)
}

// -------------------------------- Project Assignment --------------------------------

#[command]
fn assign_project_node(project_path: String, version: String) -> Result<ProjectRecord, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let p = PathBuf::from(&project_path);
    if !p.is_dir() {
        return Err(format!("Project directory does not exist: {}", project_path));
    }

    let clean = version.trim().trim_start_matches('v');

    // 1. Write .nodepilot.local
    let local_file = p.join(LOCAL_METADATA_FILENAME);
    std::fs::write(&local_file, format!("{}\n", clean)).map_err(|e| e.to_string())?;

    // 2. Safely add to .gitignore
    ensure_gitignore_entry(&p).map_err(|e| e.to_string())?;

    // 3. Update project record in state
    let mut state = AppState::load(&paths.state_file()).map_err(|e| e.to_string())?;
    let mut updated_record = None;

    for proj in &mut state.projects {
        if proj.path == p {
            proj.assigned_node = Some(clean.to_string());
            proj.config_source = nodepilot_core::ConfigSourceType::LocalOverride;
            proj.config_source_path = Some(local_file.clone());
            proj.is_inherited = false;

            let compat = evaluate_compatibility(
                &proj.framework,
                proj.framework_version.as_deref(),
                Some(clean),
                proj.node_requirements.as_deref(),
            );
            proj.compatibility_status = compat.status;
            proj.compatibility_message = Some(compat.message);
            proj.recommended_node = compat.recommended_node;

            updated_record = Some(proj.clone());
            break;
        }
    }

    state.save(&paths.state_file()).map_err(|e| e.to_string())?;

    updated_record.ok_or_else(|| "Project updated on disk, reload dashboard to refresh state".to_string())
}

#[command]
fn bulk_assign_node(project_paths: Vec<String>, version: String) -> Result<usize, String> {
    let mut count = 0;
    for path in project_paths {
        if assign_project_node(path, version.clone()).is_ok() {
            count += 1;
        }
    }
    Ok(count)
}

#[command]
fn get_auto_assign_diff() -> Result<Vec<AutoAssignProposal>, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let state = AppState::load(&paths.state_file()).map_err(|e| e.to_string())?;
    let diff = calculate_auto_assign_diff(&state.projects);
    Ok(diff)
}

#[command]
fn apply_auto_assign_recommendations() -> Result<usize, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let state = AppState::load(&paths.state_file()).map_err(|e| e.to_string())?;
    let diff = calculate_auto_assign_diff(&state.projects);

    let mut applied = 0;
    for proposal in diff {
        if let Some(proj) = state.projects.iter().find(|p| p.id == proposal.project_id) {
            let path_str = proj.path.to_string_lossy().to_string();
            if assign_project_node(path_str, proposal.proposed_version).is_ok() {
                applied += 1;
            }
        }
    }

    Ok(applied)
}

// -------------------------------- Node Runtimes --------------------------------

#[command]
fn get_installed_node_versions() -> Result<Vec<InstalledVersion>, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let state = AppState::load(&paths.state_file()).unwrap_or_default();
    let installed = list_installed_versions(&paths, &state.projects).map_err(|e| e.to_string())?;
    Ok(installed)
}

#[command]
async fn get_available_node_versions() -> Result<Vec<NodeRelease>, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let releases = fetch_releases(&paths, false).await.map_err(|e| e.to_string())?;
    Ok(releases)
}

#[command]
async fn install_node_version(version: String) -> Result<String, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let releases = fetch_releases(&paths, false).await.map_err(|e| e.to_string())?;
    let resolved = resolve_version_alias(&version, &releases).map_err(|e| e.to_string())?;

    if is_version_installed(&paths, &resolved) {
        return Ok(format!("Node {} is already installed", resolved));
    }

    let archive = download_and_verify_archive(&paths, &resolved, None::<fn(u64, Option<u64>)>)
        .await
        .map_err(|e| e.to_string())?;

    let installed_dir = install_node_archive(&paths, &archive.archive_path, &resolved)
        .map_err(|e| e.to_string())?;

    Ok(format!("Installed Node {} at {}", resolved, installed_dir.display()))
}

#[command]
fn uninstall_node_version_cmd(version: String) -> Result<(), String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let clean = version.trim_start_matches('v');
    uninstall_node_version(&paths, clean).map_err(|e| e.to_string())?;
    Ok(())
}

// -------------------------------- Environment & Coexistence --------------------------------

#[command]
fn get_doctor_report() -> Result<EnvironmentReport, String> {
    let report = inspect_environment();
    Ok(report)
}

#[command]
fn get_shell_integration_status() -> Result<IntegrationStatus, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    Ok(get_integration_status(&paths))
}

#[command]
fn toggle_shell_integration(enable: bool) -> Result<IntegrationStatus, String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    if enable {
        enable_integration(&paths).map_err(|e| e.to_string())?;
    } else {
        disable_integration(&paths).map_err(|e| e.to_string())?;
    }
    Ok(get_integration_status(&paths))
}

// -------------------------------- Actions (Terminal, Folder, Editor) --------------------------------

#[command]
fn open_project_terminal(project_path: String) -> Result<(), String> {
    let paths = NodePilotPaths::default().map_err(|e| e.to_string())?;
    let p = PathBuf::from(&project_path);
    let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
    let resolved = resolve_project_node(&p, settings.global_default_version.as_deref());

    let mut env_path = std::env::var("PATH").unwrap_or_default();
    if let Some(res) = resolved {
        let clean = res.raw_version.trim_start_matches('v');
        if is_version_installed(&paths, clean) {
            let bin_dir = paths.version_bin_dir(clean);
            let sep = if cfg!(windows) { ";" } else { ":" };
            env_path = format!("{}{}{}", bin_dir.to_string_lossy(), sep, env_path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let normalized = nodepilot_core::normalize_path(&p);
        // Try Windows Terminal first (wt -d <path>), then powershell
        let wt_status = std::process::Command::new("wt")
            .arg("-d")
            .arg(&normalized)
            .env("PATH", &env_path)
            .spawn();

        if wt_status.is_err() {
            // Fallback to powershell launched directly in the project directory
            std::process::Command::new("powershell")
                .arg("-NoExit")
                .current_dir(&normalized)
                .env("PATH", &env_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On macOS: open -a Terminal <path>
        std::process::Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(&project_path)
            .env("PATH", &env_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[command]
fn open_project_folder(project_path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let p = PathBuf::from(&project_path);
        let normalized = nodepilot_core::normalize_path(&p);
        std::process::Command::new("explorer")
            .arg(&normalized)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("open")
            .arg(&project_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[command]
fn open_project_editor(project_path: String, editor: Option<String>) -> Result<(), String> {
    let ed = editor.unwrap_or_else(|| "code".into());
    let (cmd_name, cmd_arg) = match ed.as_str() {
        "webstorm" => ("webstorm", project_path),
        _ => ("code", project_path),
    };

    #[cfg(target_os = "windows")]
    {
        let cmd = format!("{}.cmd", cmd_name);
        if std::process::Command::new(&cmd).arg(&cmd_arg).spawn().is_err() {
            // Try standard executable without .cmd
            std::process::Command::new(cmd_name)
                .arg(&cmd_arg)
                .spawn()
                .map_err(|e| format!("Could not launch {}: {}", cmd_name, e))?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new(cmd_name)
            .arg(&cmd_arg)
            .spawn()
            .map_err(|e| format!("Could not launch {}: {}", cmd_name, e))?;
    }

    Ok(())
}

#[command]
async fn check_app_updates() -> Result<UpdateCheckResult, String> {
    check_for_updates().await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let paths = NodePilotPaths::default().expect("Failed to initialize paths");
    let _ = paths.ensure_dirs();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_settings,
            update_settings,
            add_workspace,
            remove_workspace,
            scan_workspaces,
            assign_project_node,
            bulk_assign_node,
            get_auto_assign_diff,
            apply_auto_assign_recommendations,
            get_installed_node_versions,
            get_available_node_versions,
            install_node_version,
            uninstall_node_version_cmd,
            get_doctor_report,
            get_shell_integration_status,
            toggle_shell_integration,
            open_project_terminal,
            open_project_folder,
            open_project_editor,
            check_app_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running nodepilot desktop application");
}
