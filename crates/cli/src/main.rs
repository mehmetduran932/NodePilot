//! NodePilot CLI Application.

use clap::{Parser, Subcommand};
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use nodepilot_core::{
    ensure_gitignore_entry, inspect_environment, resolve_project_node,
    AppState, NodePilotPaths, Settings,
    get_integration_status, enable_integration, disable_integration,
    LOCAL_METADATA_FILENAME,
};
use nodepilot_node_runtime::{
    download_and_verify_archive, fetch_releases, install_node_archive,
    is_version_installed, list_installed_versions, resolve_version_alias,
    uninstall_node_version,
};
use nodepilot_scanner::scan_workspace;
use nodepilot_updater::check_for_updates;

static CHECKMARK: Emoji<'_, '_> = Emoji("✔ ", "[v] ");
static CROSS: Emoji<'_, '_> = Emoji("✖ ", "[x] ");
static WARN: Emoji<'_, '_> = Emoji("⚠ ", "[!] ");
static PACKAGE: Emoji<'_, '_> = Emoji("📦 ", "");

#[derive(Parser)]
#[command(name = "nodepilot", author, version, about = "Cross-platform project-aware Node.js version manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display NodePilot version information
    Version,

    /// Install a Node.js runtime (e.g. 22, 22.18.0, lts, latest)
    Install {
        /// Version number, major, or alias
        version: String,
    },

    /// Uninstall an installed Node.js runtime
    Uninstall {
        /// Version number to remove
        version: String,
    },

    /// List all installed Node.js versions
    List,

    /// Display the resolved Node version for the current directory
    Current,

    /// Assign a Node.js version to the current project (.nodepilot.local)
    Assign {
        /// Version to assign (if omitted, interactive menu will open)
        version: Option<String>,
    },

    /// Remove local NodePilot assignment from current project
    Unassign,

    /// Scan a directory or workspace for Node.js projects
    Scan {
        /// Path to scan (defaults to current directory)
        path: Option<PathBuf>,
    },

    /// List all cached projects across configured workspaces
    Projects,

    /// Manage registered workspaces
    Workspaces {
        #[command(subcommand)]
        action: Option<WorkspaceCommands>,
    },

    /// Run diagnostic environment and coexistence checks
    Doctor,

    /// Manage shell and PATH integration
    Integration {
        #[command(subcommand)]
        action: Option<IntegrationCommands>,
    },

    /// Check for NodePilot application updates
    Update,

    /// Open or display application logs directory
    Logs,
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// Add a directory to workspaces
    Add { path: PathBuf },
    /// Remove a directory from workspaces
    Remove { path: PathBuf },
    /// List all configured workspaces
    List,
}

#[derive(Subcommand)]
enum IntegrationCommands {
    /// Show current shell integration status
    Status,
    /// Enable NodePilot in user PATH
    Enable,
    /// Disable NodePilot from user PATH
    Disable,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let paths = NodePilotPaths::default()?;
    paths.ensure_dirs()?;

    match cli.command {
        Commands::Version => {
            println!("NodePilot v{}", env!("CARGO_PKG_VERSION"));
            println!("Philosophy: Stop managing Node versions globally. Let each project define its own Node environment.");
        }

        Commands::Install { version } => {
            handle_install(&paths, &version).await?;
        }

        Commands::Uninstall { version } => {
            let clean = version.trim_start_matches('v');
            if !is_version_installed(&paths, clean) {
                println!("{} Node {} is not currently installed.", WARN, clean);
                return Ok(());
            }

            uninstall_node_version(&paths, clean)?;
            println!("{} Node {} uninstalled successfully.", CHECKMARK, clean);
        }

        Commands::List => {
            let state = AppState::load(&paths.state_file()).unwrap_or_default();
            let installed = list_installed_versions(&paths, &state.projects)?;

            println!("\n{}", style("Installed Node.js Runtimes").bold());
            println!("Location: {}\n", paths.versions_dir.display());

            if installed.is_empty() {
                println!("  No Node versions installed yet.");
                println!("  Run: {} to install one.\n", style("nodepilot install lts").cyan());
            } else {
                for item in installed {
                    let mb = item.size_bytes as f64 / 1_048_576.0;
                    let usage_str = if item.projects_using_count > 0 {
                        format!(" (used by {} project{})", item.projects_using_count, if item.projects_using_count == 1 { "" } else { "s" })
                    } else {
                        "".to_string()
                    };
                    println!("  {} v{} ({:.1} MB){}", CHECKMARK, item.version, mb, style(usage_str).dim());
                }
                println!();
            }
        }

        Commands::Current => {
            let cwd = std::env::current_dir()?;
            let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
            let resolved = resolve_project_node(&cwd, settings.global_default_version.as_deref());

            match resolved {
                Some(res) => {
                    let clean = res.raw_version.trim_start_matches('v');
                    let installed_status = if is_version_installed(&paths, clean) {
                        style("installed").green()
                    } else {
                        style("not installed - run 'nodepilot install <version>'").red()
                    };

                    println!("\n{}", style("Current Node Configuration").bold());
                    println!("  Directory: {}", cwd.display());
                    println!("  Node:      v{} ({})", res.raw_version, installed_status);
                    println!("  Source:    {:?}", res.source_type);
                    println!("  File:      {}", res.source_path.display());
                    if res.is_inherited {
                        println!("  Status:    {} (inherited from parent)", style("Inherited").yellow());
                    }
                    println!();
                }
                None => {
                    println!("\nNo Node.js version defined for {}", cwd.display());
                    println!("Run {} to assign a version.\n", style("nodepilot assign <version>").cyan());
                }
            }
        }

        Commands::Assign { version } => {
            handle_assign(&paths, version).await?;
        }

        Commands::Unassign => {
            let cwd = std::env::current_dir()?;
            let local_file = cwd.join(LOCAL_METADATA_FILENAME);
            if local_file.is_file() {
                std::fs::remove_file(&local_file)?;
                println!("{} Removed local assignment (.nodepilot.local) from {}", CHECKMARK, cwd.display());
            } else {
                println!("{} No .nodepilot.local file found in {}", WARN, cwd.display());
            }
        }

        Commands::Scan { path } => {
            let target = path.unwrap_or(std::env::current_dir()?);
            println!("{} Scanning for Node.js projects in {}...", PACKAGE, target.display());

            let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
            let projects = scan_workspace(&target, settings.global_default_version.as_deref(), 8)?;

            println!("\nDiscovered {} projects:\n", projects.len());
            for p in &projects {
                let node_display = p.assigned_node.as_deref().unwrap_or("None");
                let fw_ver = p.framework_version.as_deref().unwrap_or("");
                println!(
                    "  {: <20} {: <15} {: <10} [{:?}]",
                    p.name,
                    format!("{} {}", p.framework, fw_ver),
                    node_display,
                    p.compatibility_status
                );
            }
            println!();
        }

        Commands::Projects => {
            let state = AppState::load(&paths.state_file()).unwrap_or_default();
            println!("\n{} Cached Projects ({} total):\n", PACKAGE, state.projects.len());

            for p in &state.projects {
                let node_display = p.assigned_node.as_deref().unwrap_or("None");
                let fw_ver = p.framework_version.as_deref().unwrap_or("");
                println!(
                    "  {: <22} {: <16} Node {: <10} {}",
                    style(&p.name).bold(),
                    format!("{} {}", p.framework, fw_ver),
                    node_display,
                    style(p.path.display().to_string()).dim()
                );
            }
            println!();
        }

        Commands::Workspaces { action } => {
            let mut state = AppState::load(&paths.state_file()).unwrap_or_default();
            match action.unwrap_or(WorkspaceCommands::List) {
                WorkspaceCommands::Add { path } => {
                    let canonical = path.canonicalize().unwrap_or(path);
                    if state.add_workspace(canonical.clone()) {
                        state.save(&paths.state_file())?;
                        println!("{} Added workspace: {}", CHECKMARK, canonical.display());
                    } else {
                        println!("{} Workspace already registered: {}", WARN, canonical.display());
                    }
                }
                WorkspaceCommands::Remove { path } => {
                    if state.remove_workspace(&path) {
                        state.save(&paths.state_file())?;
                        println!("{} Removed workspace: {}", CHECKMARK, path.display());
                    } else {
                        println!("{} Workspace not found: {}", CROSS, path.display());
                    }
                }
                WorkspaceCommands::List => {
                    println!("\nRegistered Workspaces ({}):\n", state.workspaces.len());
                    for w in &state.workspaces {
                        println!("  {}", w.display());
                    }
                    println!();
                }
            }
        }

        Commands::Doctor => {
            run_doctor(&paths)?;
        }

        Commands::Integration { action } => {
            match action.unwrap_or(IntegrationCommands::Status) {
                IntegrationCommands::Status => {
                    let status = get_integration_status(&paths);
                    println!("\n{}", style("NodePilot Shell Integration Status").bold());
                    println!("  Enabled in User PATH:  {}", if status.is_enabled { style("YES").green() } else { style("NO (Safe Mode)").yellow() });
                    println!("  NodePilot Bin Path:    {}", status.bin_dir.display());
                    println!("  Active in Current Env: {}", if status.in_current_process_path { "YES" } else { "NO" });
                    println!("  Backup Available:      {}", if status.backup_available { "YES" } else { "NO" });
                    println!();
                }
                IntegrationCommands::Enable => {
                    enable_integration(&paths)?;
                    println!("{} Shell integration enabled.", CHECKMARK);
                    println!("  NodePilot bin directory added to your user PATH.");
                    println!("  Open a new terminal to start using project-aware 'node' and 'npm'.");
                }
                IntegrationCommands::Disable => {
                    disable_integration(&paths)?;
                    println!("{} Shell integration disabled.", CHECKMARK);
                    println!("  NodePilot removed from your user PATH.");
                }
            }
        }

        Commands::Update => {
            println!("Checking for NodePilot updates...");
            let update = check_for_updates().await?;
            if update.update_available {
                println!("\n{} NodePilot {} is available! (Current: {})", CHECKMARK, update.latest_version, update.current_version);
                println!("To update run:\n  {}", style(&update.package_manager_command).cyan());
            } else {
                println!("{} You are on the latest version of NodePilot (v{}).", CHECKMARK, update.current_version);
            }
        }

        Commands::Logs => {
            println!("NodePilot logs directory:\n  {}", paths.logs_dir.display());
        }
    }

    Ok(())
}

async fn handle_install(paths: &NodePilotPaths, query: &str) -> anyhow::Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Fetching Node.js release index...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let releases = fetch_releases(paths, false).await?;
    let resolved_version = resolve_version_alias(query, &releases)?;
    spinner.finish_and_clear();

    println!("Resolved version: {}", style(&resolved_version).bold());

    if is_version_installed(paths, &resolved_version) {
        println!("{} Node {} is already installed at {}", CHECKMARK, resolved_version, paths.version_dir(&resolved_version).display());
        return Ok(());
    }

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );

    let pb_clone = pb.clone();
    let archive = download_and_verify_archive(paths, &resolved_version, Some(move |current, total| {
        if let Some(tot) = total {
            pb_clone.set_length(tot);
        }
        pb_clone.set_position(current);
    })).await?;

    pb.finish_with_message("Download verified.");

    println!("Extracting and installing atomically...");
    let installed_dir = install_node_archive(paths, &archive.archive_path, &resolved_version)?;

    println!("{} Node {} installed successfully into {}\n", CHECKMARK, resolved_version, installed_dir.display());
    Ok(())
}

async fn handle_assign(paths: &NodePilotPaths, version_opt: Option<String>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let state = AppState::load(&paths.state_file()).unwrap_or_default();
    let installed = list_installed_versions(paths, &state.projects)?;

    let target_version = match version_opt {
        Some(v) => v,
        None => {
            // Interactive version selection
            if installed.is_empty() {
                println!("No Node.js versions are installed yet.");
                println!("Run: {} first.", style("nodepilot install lts").cyan());
                return Ok(());
            }

            let mut options: Vec<String> = installed.iter().map(|i| format!("{} (installed)", i.version)).collect();
            options.push("Download & install another version...".to_string());

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Node.js version for this project:")
                .default(0)
                .items(&options)
                .interact()?;

            if selection == installed.len() {
                println!("Please run: nodepilot install <version> first.");
                return Ok(());
            }

            installed[selection].version.clone()
        }
    };

    let clean = target_version.trim_start_matches('v');

    // Check if installed
    if !is_version_installed(paths, clean) {
        println!("{} Node {} is not installed.", WARN, clean);
        println!("Installing now...");
        handle_install(paths, clean).await?;
    }

    // Write .nodepilot.local
    let local_file = cwd.join(LOCAL_METADATA_FILENAME);
    std::fs::write(&local_file, format!("{}\n", clean))?;

    // Idempotent .gitignore update
    ensure_gitignore_entry(&cwd)?;

    println!("\n{} Project assigned to Node {}", CHECKMARK, clean);
    println!("  Created {}", LOCAL_METADATA_FILENAME);
    println!("  Updated .gitignore safely");
    println!("  Working directory: {}\n", cwd.display());

    Ok(())
}

fn run_doctor(paths: &NodePilotPaths) -> anyhow::Result<()> {
    println!("\n{}", style("NodePilot Doctor - Environment Health Report").bold());
    println!("--------------------------------------------------");

    let report = inspect_environment();
    let integration = get_integration_status(paths);

    println!("\n{} Existing Node Managers:", style("1.").bold());
    if report.existing_managers.is_empty() {
        println!("   None detected. System is clean for NodePilot integration.");
    } else {
        for mgr in &report.existing_managers {
            println!("   {} {} ({})", WARN, mgr.name, mgr.description);
            if let Some(ref p) = mgr.path {
                println!("     Path: {}", p.display());
            }
        }
        println!("   {} NodePilot Safe Coexistence Mode is recommended.", CHECKMARK);
    }

    println!("\n{} System Node Binary:", style("2.").bold());
    if let Some(ref p) = report.system_node_path {
        println!("   Path:    {}", p.display());
        if let Some(ref v) = report.system_node_version {
            println!("   Version: {}", v);
        }
    } else {
        println!("   No system node binary found in PATH.");
    }

    println!("\n{} NodePilot Status:", style("3.").bold());
    println!("   Root Directory:    {}", paths.root.display());
    println!("   Versions Directory:{}", paths.versions_dir.display());
    println!("   Bin / Shims:       {}", paths.bin_dir.display());
    println!("   Shell Integration: {}", if integration.is_enabled { style("ENABLED").green() } else { style("DISABLED (Safe Coexistence)").yellow() });

    println!("\n{} Coexistence Guarantee:", style("4.").bold());
    println!("   NodePilot never deletes, rewrites, or interferes with other Node managers.");
    println!("   All runtimes are strictly isolated inside NodePilot's directory.\n");

    Ok(())
}
