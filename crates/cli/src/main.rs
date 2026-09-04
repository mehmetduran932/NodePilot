//! NodePilot CLI Application.

use clap::{Parser, Subcommand};
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
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
use nodepilot_compatibility::calculate_auto_assign_diff;
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

    /// Install a Node.js runtime (if omitted, automatically detects and installs the project's required Node version)
    Install {
        /// Version number, major, or alias (e.g. 22, 20.19.0, lts). If omitted, installs current project requirement.
        version: Option<String>,
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

    /// Initial setup and configuration wizard (optional browser GUI, shell integration, workspaces)
    Setup,

    /// Show recommended Node.js versions for projects (Framework & Matrix Analysis)
    Recommend {
        /// Target path (defaults to current directory or cached projects)
        path: Option<PathBuf>,
    },

    /// Automatically apply recommended Node.js versions to all scanned projects
    AutoAssign {
        /// Target path (defaults to current directory)
        path: Option<PathBuf>,
        /// Apply assignments directly without confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Manage or launch the visual desktop/browser interface
    Gui {
        #[command(subcommand)]
        action: Option<GuiCommands>,
    },

    /// Run an arbitrary command in the active project's Node.js environment
    Run {
        /// Command to execute (e.g. ng, tsc, vite, npm)
        command: String,
        /// Arguments passed to the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum GuiCommands {
    /// Launch the visual desktop interface
    Open,
    /// Install/enable the visual desktop interface component
    Install,
    /// Disable/remove the visual GUI (switches back to pure terminal mode)
    Remove,
    /// Show current visual GUI status
    Status,
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
            handle_install(&paths, version.as_deref()).await?;
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
            println!("\n{} Scanning for Node.js projects in {}...", PACKAGE, style(target.display().to_string()).cyan());

            let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
            let projects = scan_workspace(&target, settings.global_default_version.as_deref(), 8)?;

            let total_count = projects.len();
            let mut compatible_count = 0;
            let mut incompatible_count = 0;
            let mut unassigned_count = 0;

            for p in &projects {
                if p.assigned_node.is_none() {
                    unassigned_count += 1;
                }
                match p.compatibility_status {
                    nodepilot_core::CompatibilityStatus::Compatible => compatible_count += 1,
                    nodepilot_core::CompatibilityStatus::Incompatible => incompatible_count += 1,
                    nodepilot_core::CompatibilityStatus::Unknown => {},
                }
            }

            println!("\n{}", style("┌─ Discovered Projects ────────────────────────────────────────────────────────────────────────┐").dim());
            println!(
                "  {:<30} {:<20} {:<12} {:<14} {}",
                style("PROJECT NAME").bold().white(),
                style("FRAMEWORK").bold().white(),
                style("ASSIGNED").bold().white(),
                style("RECOMMENDED").bold().white(),
                style("STATUS").bold().white()
            );
            println!("{}", style("├──────────────────────────────────────────────────────────────────────────────────────────────┤").dim());

            for p in &projects {
                let node_display = p.assigned_node.as_deref().unwrap_or("-");
                let rec_display = p.recommended_node.as_deref().map(|r| format!("v{}", r)).unwrap_or_else(|| "-".into());
                let fw_ver = p.framework_version.as_deref().unwrap_or("");
                let fw_str = if fw_ver.is_empty() {
                    p.framework.clone()
                } else {
                    format!("{} {}", p.framework, fw_ver)
                };

                // Truncate cleanly if too long for column
                let proj_name = if p.name.len() > 28 {
                    format!("{}...", &p.name[..25])
                } else {
                    p.name.clone()
                };

                let status_badge = match p.compatibility_status {
                    nodepilot_core::CompatibilityStatus::Compatible => style("✔ Compatible").green(),
                    nodepilot_core::CompatibilityStatus::Incompatible => style("✖ Incompatible").red().bold(),
                    nodepilot_core::CompatibilityStatus::Unknown => style("? Unassigned").dim(),
                };

                let assigned_styled = if node_display == "-" {
                    style("-".to_string()).dim()
                } else {
                    style(format!("v{}", node_display.trim_start_matches('v'))).yellow()
                };

                let rec_styled = if rec_display == "-" {
                    style("-".to_string()).dim()
                } else {
                    style(rec_display).cyan().bold()
                };

                println!(
                    "  {:<30} {:<20} {:<12} {:<14} {}",
                    style(proj_name).bold(),
                    style(fw_str).dim(),
                    assigned_styled,
                    rec_styled,
                    status_badge
                );
            }

            println!("{}", style("└──────────────────────────────────────────────────────────────────────────────────────────────┘").dim());
            println!(
                "Summary: {} total | {} compatible | {} needs update | {} unassigned",
                style(total_count).bold(),
                style(compatible_count).green(),
                style(incompatible_count).red(),
                style(unassigned_count).yellow()
            );

            if incompatible_count > 0 || unassigned_count > 0 {
                println!(
                    "\nNext Action: Run '{}' to preview diff or '{}' to auto-assign all recommended Node versions.\n",
                    style("nodepilot recommend").cyan(),
                    style("nodepilot auto-assign").cyan().bold()
                );
            } else {
                println!();
            }
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
                    let normalized = nodepilot_core::normalize_path(&path);
                    if state.add_workspace(normalized.clone()) {
                        state.save(&paths.state_file())?;
                        println!("{} Added workspace: {}", CHECKMARK, normalized.display());
                    } else {
                        println!("{} Workspace already registered: {}", WARN, normalized.display());
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

        Commands::Setup => {
            run_setup_wizard(&paths).await?;
        }

        Commands::Recommend { path } => {
            handle_recommend(&paths, path)?;
        }

        Commands::AutoAssign { path, yes } => {
            handle_auto_assign(&paths, path, yes).await?;
        }

        Commands::Gui { action } => {
            handle_gui_action(&paths, action).await?;
        }

        Commands::Run { command, args } => {
            let shim_binary = paths.bin_dir.join(if cfg!(windows) { "nodepilot-shim.exe" } else { "nodepilot-shim" });
            if !shim_binary.is_file() {
                eprintln!("{} NodePilot shim binary not found in {}. Please run 'nodepilot setup'.", WARN, paths.bin_dir.display());
                std::process::exit(1);
            }
            let mut cmd = std::process::Command::new(&shim_binary);
            cmd.arg(&command).args(&args);
            let status = cmd.status()?;
            std::process::exit(status.code().unwrap_or(0));
        }
    }

    Ok(())
}

async fn handle_install(paths: &NodePilotPaths, query_opt: Option<&str>) -> anyhow::Result<()> {
    if let Some(q) = query_opt {
        if q.eq_ignore_ascii_case("gui") || q.eq_ignore_ascii_case("desktop") {
            return handle_gui_action(paths, Some(GuiCommands::Install)).await;
        }
    }

    let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
    let cwd = std::env::current_dir()?;

    let query = match query_opt {
        Some(q) => q.to_string(),
        None => {
            // Auto-detect project requirement
            println!("Detecting Node.js version requirement for {}...", cwd.display());
            match resolve_project_node(&cwd, settings.global_default_version.as_deref()) {
                Some(res) => {
                    println!("{} Detected required Node version: v{} (from {:?})", CHECKMARK, res.raw_version, res.source_type);
                    res.raw_version
                }
                None => {
                    println!("{} No Node.js version requirement found in current project.", WARN);
                    println!("Defaulting to latest active LTS...");
                    "lts".to_string()
                }
            }
        }
    };

    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Fetching Node.js release index...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let releases = fetch_releases(paths, false).await?;
    let resolved_version = resolve_version_alias(&query, &releases)?;
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
        println!("Automatically downloading and configuring environment...");
        handle_install(paths, Some(clean)).await?;
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

async fn run_setup_wizard(paths: &NodePilotPaths) -> anyhow::Result<()> {
    println!("\n{}", style("==================================================").cyan());
    println!("{}", style("    🚀 NodePilot Setup & Environment Wizard").bold().cyan());
    println!("{}\n", style("==================================================").cyan());

    println!("NodePilot's primary interface is right here in your terminal (CLI).");
    println!("Everything — scanning, assignments, version management — can be controlled from your shell.\n");

    let theme = ColorfulTheme::default();
    let mut settings = Settings::load(&paths.settings_file()).unwrap_or_default();
    let mut state = AppState::load(&paths.state_file()).unwrap_or_default();

    // 1. PATH & Shell Integration
    let enable_shell = Confirm::with_theme(&theme)
        .with_prompt("Enable Shell Integration? (Adds NodePilot shims to your user PATH for automatic project routing)")
        .default(true)
        .interact()?;

    if enable_shell {
        enable_integration(paths)?;
        settings.shell_integration_enabled = true;
        println!("{} Shell integration enabled in user PATH.", CHECKMARK);
    } else {
        println!("{} Shell integration skipped. (You can enable it anytime via 'nodepilot integration enable')", WARN);
    }

    // 2. Default Workspace Registration
    let register_cwd = Confirm::with_theme(&theme)
        .with_prompt(format!("Add current directory ({}) as a project workspace?", std::env::current_dir()?.display()))
        .default(true)
        .interact()?;

    if register_cwd {
        let cwd = std::env::current_dir()?;
        let normalized = nodepilot_core::normalize_path(&cwd);
        state.add_workspace(normalized.clone());
        println!("{} Registered workspace: {}", CHECKMARK, normalized.display());

        // Perform initial scan
        println!("{} Scanning for existing Node.js projects...", PACKAGE);
        let found = scan_workspace(&normalized, None, 6)?;
        println!("{} Found {} projects.", CHECKMARK, found.len());
        state.projects = found;
        state.save(&paths.state_file())?;
    }

    // 3. Optional Desktop / Browser GUI Interface
    println!("\n{}", style("Desktop / Browser Interface (Optional)").bold());
    println!("NodePilot can run completely headless via CLI. However, a modern visual dashboard (Desktop App / Browser UI) is also available for visual project health, matrix inspection, and bulk assignments.");

    let want_gui = Confirm::with_theme(&theme)
        .with_prompt("Do you want to enable/install the visual Desktop & Browser interface as well?")
        .default(false)
        .interact()?;

    settings.gui_installed = want_gui;
    settings.ask_gui_launch = want_gui;
    settings.save(&paths.settings_file())?;

    println!("\n{}", style("Setup Completed Successfully!").bold().green());
    println!("--------------------------------------------------");
    println!("• Terminal commands available:");
    println!("  - {}     : List all projects and their assigned Node versions", style("nodepilot projects").cyan());
    println!("  - {}       : Assign current directory to a Node version", style("nodepilot assign").cyan());
    println!("  - {}      : Check resolved Node runtime for current directory", style("nodepilot current").cyan());
    println!("  - {}         : List all downloaded Node runtimes", style("nodepilot list").cyan());
    println!("  - {}       : Environment diagnostics & coexistence report", style("nodepilot doctor").cyan());

    if want_gui {
        println!("\n• Visual GUI is enabled:");
        println!("  - Run {} anytime to open the visual dashboard.", style("nodepilot gui").cyan());

        let launch_now = Confirm::with_theme(&theme)
            .with_prompt("Launch the visual interface now?")
            .default(false)
            .interact()?;

        if launch_now {
            launch_gui(paths)?;
        }
    } else {
        println!("• Visual GUI was skipped (Terminal-only mode active).");
        println!("  (If you change your mind later, simply run '{}'.)", style("nodepilot gui").cyan());
    }

    println!();
    Ok(())
}

async fn handle_gui_action(paths: &NodePilotPaths, action: Option<GuiCommands>) -> anyhow::Result<()> {
    let mut settings = Settings::load(&paths.settings_file()).unwrap_or_default();

    match action.unwrap_or(GuiCommands::Open) {
        GuiCommands::Open => {
            if !settings.gui_installed && !is_gui_binary_present(paths) {
                println!("{} NodePilot Visual GUI component is not currently installed/enabled.", WARN);
                println!("You can install and enable it without reinstalling NodePilot by running:\n  {}", style("nodepilot gui install").cyan());
                return Ok(());
            }
            launch_gui(paths)?;
        }

        GuiCommands::Install => {
            println!("\n{}", style("Adding NodePilot Visual Desktop & Browser Interface...").bold().cyan());
            settings.gui_installed = true;
            settings.ask_gui_launch = true;
            settings.save(&paths.settings_file())?;

            println!("{} Visual GUI configuration enabled in settings.", CHECKMARK);

            // Check if binary is present or provide direct command
            if is_gui_binary_present(paths) {
                println!("{} GUI binary detected and ready to use.", CHECKMARK);
                println!("Run {} to open the dashboard.", style("nodepilot gui").cyan());
            } else {
                println!("{} GUI component enabled.", CHECKMARK);
                println!("To compile or launch the Tauri desktop/browser bundle:");
                println!("  1. cd apps/desktop && npm run build");
                println!("  2. cargo tauri build (or 'cargo tauri dev' to launch immediately)");
                println!("The GUI binary will be accessible directly via {} thereafter.", style("nodepilot gui").cyan());
            }
            println!();
        }

        GuiCommands::Remove => {
            println!("\n{}", style("Removing & Disabling NodePilot Visual GUI Component...").bold().yellow());
            settings.gui_installed = false;
            settings.ask_gui_launch = false;
            settings.save(&paths.settings_file())?;

            #[cfg(target_os = "windows")]
            {
                // Terminate any running instances first so Windows releases the file lock
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/IM", "nodepilot-desktop.exe"])
                    .output();
            }

            // Remove desktop binary from user bin if present
            let desktop_bin_name = if cfg!(windows) { "nodepilot-desktop.exe" } else { "nodepilot-desktop" };
            let user_desktop_bin = paths.bin_dir.join(desktop_bin_name);
            if user_desktop_bin.is_file() {
                if let Err(e) = std::fs::remove_file(&user_desktop_bin) {
                    println!("{} Could not delete {}: {}", WARN, user_desktop_bin.display(), e);
                } else {
                    println!("{} Removed visual GUI executable from disk.", CHECKMARK);
                }
            }

            #[cfg(target_os = "windows")]
            {
                let dll = paths.bin_dir.join("WebView2Loader.dll");
                let _ = std::fs::remove_file(&dll);
            }

            println!("{} NodePilot switched to pure Terminal-first mode.", CHECKMARK);
            println!("All project management, scanning, and runtime controls remain fully operational via CLI.");
            println!("(You can re-add the visual interface at any time with '{}'.)\n", style("nodepilot gui install").cyan());
        }

        GuiCommands::Status => {
            println!("\n{}", style("NodePilot GUI Component Status").bold());
            println!("  Configured Enabled: {}", if settings.gui_installed { style("YES").green() } else { style("NO (Terminal-Only)").yellow() });
            println!("  Binary Available:   {}", if is_gui_binary_present(paths) { style("YES").green() } else { style("NO").dim() });
            println!("  Settings Location:  {}", paths.settings_file().display());
            println!();
        }
    }

    Ok(())
}

fn is_gui_binary_present(paths: &NodePilotPaths) -> bool {
    find_gui_binary(paths).is_some()
}

fn find_gui_binary(paths: &NodePilotPaths) -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| paths.bin_dir.clone());

    let desktop_bin_name = if cfg!(windows) { "nodepilot-desktop.exe" } else { "nodepilot-desktop" };
    let primary = exe_dir.join(desktop_bin_name);
    if primary.is_file() {
        return Some(primary);
    }

    let alt_candidates = vec![
        paths.bin_dir.join(desktop_bin_name),
        exe_dir.join("../nodepilot-desktop").join(desktop_bin_name),
        exe_dir.join("../target/release").join(desktop_bin_name),
        exe_dir.join("../target/debug").join(desktop_bin_name),
    ];

    for candidate in alt_candidates {
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn launch_gui(paths: &NodePilotPaths) -> anyhow::Result<()> {
    if let Some(bin) = find_gui_binary(paths) {
        #[cfg(target_os = "windows")]
        {
            if let Some(parent) = bin.parent() {
                let target_dll = parent.join("WebView2Loader.dll");
                if !target_dll.is_file() {
                    let candidates = [
                        paths.bin_dir.join("WebView2Loader.dll"),
                        PathBuf::from("C:\\dev\\NodePilot\\target\\release\\WebView2Loader.dll"),
                    ];
                    for c in &candidates {
                        if c.is_file() {
                            let _ = std::fs::copy(c, &target_dll);
                            break;
                        }
                    }
                }
            }
        }

        println!("{} Launching NodePilot Visual Interface...", CHECKMARK);
        std::process::Command::new(&bin)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to launch {}: {}", bin.display(), e))?;
        return Ok(());
    }

    println!("{} NodePilot Desktop/Browser binary was not found.", WARN);
    println!("You are running in Terminal-only mode.");
    println!("To launch in development mode:");
    println!("  cd apps/desktop && npm run build");
    println!("  cargo run -p nodepilot-desktop");

    Ok(())
}

fn handle_recommend(paths: &NodePilotPaths, path_opt: Option<PathBuf>) -> anyhow::Result<()> {
    let target = path_opt.unwrap_or(std::env::current_dir()?);
    let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
    let projects = scan_workspace(&target, settings.global_default_version.as_deref(), 8)?;

    println!("\n{}", style("NodePilot Framework Compatibility & Node Recommendations").bold().cyan());
    println!("Target: {}\n", target.display());

    let proposals = calculate_auto_assign_diff(&projects);
    if proposals.is_empty() {
        println!("{} All scanned projects already have compatible or optimal Node versions assigned.", CHECKMARK);
        return Ok(());
    }

    println!("Found {} projects with recommended Node versions:\n", proposals.len());
    for p in &proposals {
        let current_display = p.current_version.as_deref().unwrap_or("None");
        println!(
            "• {}",
            style(&p.project_name).bold()
        );
        println!(
            "  Change: {} -> {}",
            style(current_display).dim(),
            style(format!("Node v{}", p.proposed_version)).green().bold()
        );
        println!("  Reason: {}\n", style(&p.reason).italic());
    }

    println!("To automatically apply all recommendations, run:\n  {}\n", style("nodepilot auto-assign").cyan());
    Ok(())
}

async fn handle_auto_assign(paths: &NodePilotPaths, path_opt: Option<PathBuf>, auto_confirm: bool) -> anyhow::Result<()> {
    let target = path_opt.unwrap_or(std::env::current_dir()?);
    let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
    let projects = scan_workspace(&target, settings.global_default_version.as_deref(), 8)?;
    let proposals = calculate_auto_assign_diff(&projects);

    if proposals.is_empty() {
        println!("{} No Node version updates are required for projects in {}.", CHECKMARK, target.display());
        return Ok(());
    }

    println!("\n{}", style("Automatic Node.js Version Assignment Proposals").bold().cyan());
    println!("--------------------------------------------------");
    for p in &proposals {
        let current_display = p.current_version.as_deref().unwrap_or("None");
        println!(
            "  {: <28} {: <10} -> Node {: <8} ({})",
            style(&p.project_name).bold(),
            current_display,
            style(&p.proposed_version).green().bold(),
            style(&p.reason).dim()
        );
    }
    println!();

    if !auto_confirm {
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Apply recommended Node versions to {} project(s)?", proposals.len()))
            .default(true)
            .interact()?;

        if !confirm {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    let mut updated = 0;
    for proposal in &proposals {
        if let Some(proj) = projects.iter().find(|p| p.id == proposal.project_id) {
            let clean = proposal.proposed_version.trim_start_matches('v');
            let local_file = proj.path.join(LOCAL_METADATA_FILENAME);
            std::fs::write(&local_file, format!("{}\n", clean))?;
            let _ = ensure_gitignore_entry(&proj.path);
            updated += 1;
        }
    }

    println!("\n{} Successfully assigned recommended Node versions to {} project(s).", CHECKMARK, updated);
    println!("Every project's .nodepilot.local and .gitignore have been updated safely.\n");

    Ok(())
}


