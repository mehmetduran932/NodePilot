# NodePilot Architecture Specification

> **Philosophy**: Stop managing Node versions globally. Let each project define its own Node environment.

This document details the architectural design of **NodePilot**, a cross-platform, project-aware Node.js runtime manager and developer dashboard for Windows and macOS.

---

## 1. System Overview

NodePilot decouples Node.js version management from global shell states. Instead of switching system-wide active versions via `nvm use` or global symlinks, NodePilot executes Node.js binaries tailored to the specific directory and project hierarchy of each process invocation.

```
+-------------------------------------------------------------------------+
|                              NodePilot UX                               |
|   +---------------------------------+  +----------------------------+   |
|   | Desktop GUI (Tauri 2 + React)   |  | CLI (nodepilot binary)     |   |
|   +---------------------------------+  +----------------------------+   |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                           Core Services & Engine                        |
|  +------------------------+  +-------------------+  +----------------+  |
|  | nodepilot-core         |  | nodepilot-scanner |  | nodepilot-     |  |
|  | (Config, State, Paths) |  | (Project & Mono)  |  | compatibility  |  |
|  +------------------------+  +-------------------+  +----------------+  |
|  +------------------------+  +-------------------+  +----------------+  |
|  | nodepilot-node-runtime |  | nodepilot-updater |  | nodepilot-shim |  |
|  | (Fetch, Verify, Unpack)|  | (Self-update)     |  | (Fast routing) |  |
|  +------------------------+  +-------------------+  +----------------+  |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                           Operating System                              |
| Windows 10/11 (x64, arm64)                macOS (Apple Silicon, Intel)  |
| %LOCALAPPDATA%\NodePilot                  ~/.nodepilot                  |
+-------------------------------------------------------------------------+
```

---

## 2. Workspace & Crate Structure

The repository is structured as a high-cohesion Cargo workspace with clean library boundaries:

```
NodePilot/
├── Cargo.toml                  # Root Cargo workspace manifest
├── crates/
│   ├── core/                   # Shared types, paths, errors, configuration, and state storage
│   ├── scanner/                # Fast, passive project & monorepo detection, framework parsing
│   ├── node-runtime/           # Node.js release index fetching, download, sha256 verify, atomic extract
│   ├── compatibility/          # Framework-to-Node matrix rules & recommendation engine
│   ├── shim/                   # Ultra-fast directory-walking routing binary for node, npm, npx, corepack
│   ├── updater/                # Self-update checks, binary verification, package manager diagnostics
│   └── cli/                    # Comprehensive CLI executable (`nodepilot`)
├── apps/
│   └── desktop/                # Tauri 2 desktop app (Rust backend + React/TypeScript frontend)
├── docs/                       # Specifications, user guides, compatibility matrices
├── scripts/                    # Build, packaging, and release automation scripts
├── tests/                      # Integration and coexistence test suites
└── README.md
```

### Dependency Graph

* `nodepilot-core`: Base crate depended upon by all other crates. No circular dependencies.
* `nodepilot-node-runtime`: Depends on `nodepilot-core`. Handles HTTP/TLS downloads, tar.gz/zip extraction, and local version registry.
* `nodepilot-scanner`: Depends on `nodepilot-core`. Passive JSON and file parser without shell script execution.
* `nodepilot-compatibility`: Depends on `nodepilot-core`. Contains static matrix rules for Angular, Next.js, React, Nuxt, etc.
* `nodepilot-shim`: Minimal dependencies (only `nodepilot-core` or pure lightweight resolution) to ensure sub-5ms process startup.
* `nodepilot-cli`: Depends on all engines to provide terminal commands.
* `apps/desktop`: Tauri 2 application invoking core crates directly.

---

## 3. Node Runtime Storage Layout

NodePilot isolates its runtimes completely from other tools (such as nvm, fnm, Volta, mise, Homebrew, or system Node).

### Windows (`%LOCALAPPDATA%\NodePilot`)
```
C:\Users\<User>\AppData\Local\NodePilot\
├── versions\
│   ├── 16.20.2\
│   │   ├── node.exe
│   │   ├── npm, npm.cmd
│   │   ├── npx, npx.cmd
│   │   └── node_modules\
│   ├── 20.19.5\
│   └── 22.18.0\
├── bin\                     # NodePilot shims (node.exe, npm.cmd, npx.cmd, etc.)
├── cache\                   # Downloaded tarballs/zips and release indexes
├── config\                  # Global settings (settings.json)
├── state\                   # Local SQLite/JSON state (workspaces, scanned project cache)
└── logs\                    # Structured, sanitized log files
```

### macOS (`~/.nodepilot`)
```
/Users/<user>/.nodepilot/
├── versions/
│   ├── 16.20.2/
│   │   ├── bin/node, npm, npx, corepack
│   │   ├── lib/node_modules/
│   │   └── include/
│   ├── 20.19.5/
│   └── 22.18.0/
├── bin/                     # NodePilot shims (node, npm, npx, corepack)
├── cache/                   # Download cache & SHASUMS256.txt
├── config/                  # Global settings
├── state/                   # Local database/state
└── logs/                    # Logs
```

---

## 4. Configuration Resolution Order

When executing `node`, `npm`, `npx`, or `corepack`, NodePilot resolves the required Node version by walking upward from the current working directory (`CWD`) towards the filesystem root.

Resolution priority at each directory level:

1. **`.nodepilot.local`** (Local developer override for NodePilot)
2. **`.nvmrc`** (Standard NVM config file)
3. **`.node-version`** (Standard across nodenv, asdf, fnm)
4. **`package.json` -> `volta.node`** (Volta configuration key)
5. **`package.json` -> `engines.node`** (Semver constraint specification)
6. **`.tool-versions`** (asdf / mise configuration)
7. **`mise.toml`** (mise configuration)
8. **Parent directory check**: Repeat steps 1–7 up to the filesystem root.
9. **Global NodePilot default version**: Configured in NodePilot settings.
10. **Fallback**: If no version is resolved, prompt or pass through to system node if configured.

### Local Assignment & `.gitignore` Automation

When the user assigns a Node.js version via GUI or CLI:
* File written: `.nodepilot.local` containing the exact version string (e.g. `22.18.0`).
* Idempotent `.gitignore` update:
  * Reads `.gitignore` preserving line ending format (`\r\n` on Windows, `\n` on POSIX).
  * Checks if `.nodepilot.local` is already present.
  * Appends `.nodepilot.local` if absent.
  * Creates `.gitignore` if none exists.

---

## 5. Shim & Process Routing Strategy

### Fast Shim Execution

Running `node` must introduce near-zero perceptible latency.

1. **Invocation**: User types `node app.js` or an IDE triggers `npm test`.
2. **Shim Entry Point**: The NodePilot shim executable in `%LOCALAPPDATA%\NodePilot\bin` or `~/.nodepilot/bin` is invoked.
3. **Loop Protection**:
   * Inspects `NODEPILOT_SHIM_ACTIVE`. If already set, aborts recursion and invokes the underlying binary or errors cleanly.
   * Sets `NODEPILOT_SHIM_ACTIVE=1`.
4. **Hierarchy Walk**:
   * Reads `std::env::current_dir()`.
   * Ascends parent directories looking for version configuration markers.
5. **Runtime Lookup**:
   * Checks `%LOCALAPPDATA%\NodePilot\versions\<resolved_version>` (or macOS equivalent).
   * Verifies the target executable exists (`node.exe` or `bin/node`).
6. **Execution**:
   * Windows: Uses `std::process::Command` to invoke target `node.exe` or `npm.cmd` passing all arguments, stdio inheritance, and updated `PATH` prepend so subprocesses find the matching tools.
   * Unix: Uses `execvp` via `std::os::unix::process::CommandExt::exec` replacing the shim process directly with zero memory overhead.

### Subprocess PATH Injection
Before delegating to the target Node executable, NodePilot prepends the target runtime's directory to the process `PATH`. This ensures child processes spawned by npm or build scripts automatically resolve `node`, `npm`, and runtime-installed global tools from the same isolated Node version.

---

## 6. Passive Project Scanner & Monorepos

### Scanner Rules
* **Passive Only**: Never runs scripts, package hooks (`postinstall`), or child processes.
* **Excluded Paths**: `node_modules`, `.git`, `dist`, `build`, `target`, `coverage`, `.next`, `.angular`, `.cache`, `out`, `vendor`.
* **Marker Inspection**: Reads `package.json`, `angular.json`, `nx.json`, `next.config.*`, `vite.config.*`, `pnpm-workspace.yaml`, lockfiles.
* **Framework Detection**:
  * **Angular**: Detects `@angular/core` or `@angular/cli` in `dependencies`/`devDependencies`.
  * **React**: Detects `react` (and distinguishes Next.js, Remix, Vite if present).
  * **Next.js**: Detects `next`.
  * **Vue / Nuxt**: Detects `vue`, `nuxt`.
  * **Svelte / SvelteKit**: Detects `svelte`, `@sveltejs/kit`.
  * **NestJS**: Detects `@nestjs/core`.
  * **Express**: Detects `express`.
  * **Vite**: Detects `vite`.
  * **Generic Node**: Has `package.json` with main/scripts.
* **Monorepo Awareness**:
  * Identifies root workspaces (`pnpm-workspace.yaml`, `lerna.json`, `nx.json`, or `package.json` `workspaces`).
  * Groups packages logically under the workspace parent while preserving per-package override capability.

---

## 7. Framework & Node Compatibility Engine

NodePilot includes built-in compatibility knowledge:

* **Angular**:
  * Angular 11: Node 10.13 - 12.x / 14.x
  * Angular 12: Node 12.14+ / 14.15+
  * Angular 13: Node 12.20+ / 14.15+ / 16.10+
  * Angular 14: Node 14.15+ / 16.10+
  * Angular 15: Node 14.20+ / 16.13+ / 18.10+
  * Angular 16: Node 16.14+ / 18.10+
  * Angular 17: Node 18.13+ / 20.9+
  * Angular 18: Node 18.19+ / 20.9+ / 22.0+
  * Angular 19: Node 18.19+ / 20.11+ / 22.0+
* **Next.js**:
  * Next 12: Node 12.22.0+
  * Next 13: Node 14.18.2+ / 16.8.0+
  * Next 14: Node 18.17.0+
  * Next 15: Node 18.18.0+
* **Engine Semver Validation**: Checks resolved Node versions against `engines.node` rules.

The engine provides:
* **Status**: `Compatible`, `Incompatible`, or `Unknown`.
* **Recommendations**: Suggests the optimal LTS version for the project.
* **Diff Preview**: When running bulk auto-assign, generates an exact diff before modifying `.nodepilot.local`.

---

## 8. Coexistence Strategy & Shell Integration

### Read-Only Existing Manager Detection
NodePilot inspects the system non-destructively:
* Checks for `NVM_HOME`, `NVM_DIR`, `FNM_DIR`, `VOLTA_HOME`, `.asdf`, `.mise`.
* Checks for `fnm.exe`, `nvm.exe`, `volta.exe`, `mise`, `brew`.
* Checks system PATH entries.
* **Rule**: Never modifies existing manager files, registry keys, or environment variables without explicit user confirmation.

### Safe Coexistence Mode (Default)
In Safe Coexistence Mode:
* NodePilot does not touch system PATH.
* The Desktop GUI opens terminals or spawns project tasks with isolated environment variables (`PATH` prefixed with the project's assigned Node runtime).
* Existing global Node workflows remain 100% untouched.

### System Integration Mode (User-Enabled)
When explicitly toggled on by the user:
* Adds `%LOCALAPPDATA%\NodePilot\bin` or `~/.nodepilot/bin` to the user's PATH.
* Can be enabled, disabled, or verified at any time via:
  `nodepilot integration enable`
  `nodepilot integration disable`
  `nodepilot integration status`
* Fully reversible: NodePilot saves the previous PATH state in `config/integration.json` before modifications.

---

## 9. Security Model

* **HTTPS Enforcement**: All downloads from `https://nodejs.org/dist/...` must use TLS.
* **SHA256 Integrity Verification**: Downloads the official `SHASUMS256.txt`, calculates local archive SHA256, and verifies match before unpacking.
* **Safe Extraction**: Prevents directory traversal (`ZipSlip` / tar slip vulnerability) by validating every file path against the target extraction root.
* **Atomic Installation**: Unpacks into a temporary directory `versions/.tmp_<version>_<timestamp>` and renames to `versions/<version>` only upon complete success. Failed installations are immediately wiped.
* **No Code Execution**: Scanner performs passive JSON parsing only.

---

## 10. Desktop GUI & CLI Integration

Both GUI and CLI interface directly with the Rust core crates:
* **Tauri 2 IPC**: Exposes strongly-typed Rust commands:
  * `get_workspaces`, `add_workspace`, `remove_workspace`
  * `scan_workspace`, `get_projects`, `assign_node_version`, `bulk_assign`
  * `get_installed_node_versions`, `get_available_node_versions`, `install_node_version`, `uninstall_node_version`
  * `get_system_status`, `check_compatibility`, `open_terminal`, `open_folder`, `open_editor`
  * `get_settings`, `update_settings`, `toggle_integration`
* **CLI Commands**:
  * `nodepilot version`
  * `nodepilot install <version|lts|latest>`
  * `nodepilot uninstall <version>`
  * `nodepilot list`
  * `nodepilot current`
  * `nodepilot assign [version]`
  * `nodepilot unassign`
  * `nodepilot scan [path]`
  * `nodepilot projects`
  * `nodepilot workspaces [add|remove|list]`
  * `nodepilot doctor`
  * `nodepilot integration [status|enable|disable]`
  * `nodepilot update`
  * `nodepilot logs`

---

## 11. State & Cache Storage

* Configuration file: `config/settings.json`
* Workspaces & project metadata cache: `state/projects.json` (lightweight, portable, fast JSON with atomic file writes).
* Local project metadata: `.nodepilot.local` inside the project root directory.

This architecture ensures high performance, zero global state collision, rock-solid security, and seamless developer workflows.
