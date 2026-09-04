# NodePilot 🧭

[![Build and Test](https://github.com/mehmetduran932/NodePilot/actions/workflows/build.yml/badge.svg)](https://github.com/mehmetduran932/NodePilot/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS-informational.svg)]()
[![Security Policy](https://img.shields.io/badge/Security-Policy-brightgreen.svg)](SECURITY.md)

> **Stop managing Node versions globally. Let each project define its own Node environment.**

NodePilot is a cross-platform (Windows 10/11, macOS Apple Silicon & Intel), project-aware Node.js runtime manager and desktop developer dashboard.

Instead of switching global Node environments via `nvm use` or global symlinks, NodePilot executes Node.js binaries tailored to the specific directory and project hierarchy of each process invocation.

---

## 🌟 Why NodePilot?

Traditional Node version managers require you to switch global state or remember shell hooks:

```text
Traditional Workflow:
1. Terminal opens -> Global Node 22 active
2. cd into legacy-angular (requires Node 16)
3. Run npm start -> CRASH (Incompatible engine)
4. Remember to run: nvm use 16
5. Switch to terminal B (new-project) -> Now running Node 16 by mistake!
```

**NodePilot Workflow**:

```text
C:\dev\legacy-angular    --> .nodepilot.local = 16.20.2
C:\dev\new-angular       --> .nodepilot.local = 22.18.0
```

* In Terminal A (`cd C:\dev\legacy-angular`): `node -v` outputs `v16.20.2`.
* At the exact same time in Terminal B (`cd C:\dev\new-angular`): `node -v` outputs `v22.18.0`.
* **Zero manual switching.**
* **Zero collisions between simultaneous terminals.**
* **Existing version managers (nvm, fnm, Volta, mise) are preserved and never overwritten.**

---

## 📊 Objective Comparison

| Feature | NodePilot | nvm / nvm-windows | fnm | Volta |
| :--- | :--- | :--- | :--- | :--- |
| **Philosophy** | Project-first execution | Global / Shell switching | Fast shell switching | Tool pinning |
| **Simultaneous Terminals** | ✅ Native per-process routing | ❌ Global switch alters state | ⚠️ Requires shell hooks per shell | ✅ Shim-based |
| **Desktop GUI** | ✅ Full Tauri 2 Dashboard | ❌ CLI only | ❌ CLI only | ❌ CLI only |
| **Automated Project Discovery** | ✅ Recursive workspace scanner | ❌ None | ❌ None | ❌ None |
| **Framework Compatibility Engine**| ✅ Built-in matrix (Angular, Next, etc.) | ❌ None | ❌ None | ❌ None |
| **Coexistence Mode** | ✅ Safe read-only detection | ❌ Takes over PATH | ❌ Takes over PATH | ❌ Takes over PATH |
| **Parent Directory Inheritance** | ✅ Monorepo & folder tree | ⚠️ Partial | ⚠️ Partial | ⚠️ Partial |

---

## 🚀 Key Features

* **Desktop Application**: Modern, high-performance interface built with **Tauri 2**, **React**, and **TypeScript**.
* **Unified Core in Rust**: Fast, passive project scanning, SHA-256 verified runtime downloads, and sub-millisecond process routing.
* **Smart Configuration Hierarchy**:
  1. `.nodepilot.local` (Local developer override)
  2. `.nvmrc`
  3. `.node-version`
  4. `package.json` -> `volta.node`
  5. `package.json` -> `engines.node`
  6. `.tool-versions`
  7. `mise.toml`
  8. Parent directory inheritance
  9. Global NodePilot default
* **Idempotent `.gitignore` Automation**: Automatically and safely records `.nodepilot.local` in `.gitignore`, preserving existing line endings (`\r\n` or `\n`).
* **Passive Project Scanner**: Recursively inspects development directories (e.g. `C:\dev`), detects frameworks (Angular, React, Next.js, Vue, Nuxt, Svelte, SvelteKit, NestJS, Express, Vite, Nx), and extracts package managers (npm, pnpm, yarn, bun) without ever executing untrusted project code.
* **Compatibility Advisor**: Identifies framework-to-Node version mismatches and generates preview diffs before applying recommendations.
* **Safe Coexistence Mode**: Never modifies or deletes existing `nvm`, `nvm-windows`, `fnm`, `Volta`, or `mise` installations.

---

## 💻 CLI Usage

The `nodepilot` CLI provides intuitive, human-readable commands:

```bash
# Interactive setup wizard (Asks about shell integration, workspaces & optional GUI)
nodepilot setup

# Display version and philosophy
nodepilot version

# Install a Node.js runtime (resolves latest patch or LTS)
nodepilot install 22
nodepilot install 20.19.5
nodepilot install lts
nodepilot install latest

# List installed runtimes and project usage counts
nodepilot list

# Show active Node version for current directory
nodepilot current

# Assign Node version to current project (creates .nodepilot.local & updates .gitignore)
nodepilot assign 22.18.0

# Interactive assignment prompt (lists installed runtimes)
nodepilot assign

# Remove local assignment
nodepilot unassign

# Scan directory for Node.js projects passively
nodepilot scan C:\dev

# List all discovered projects
nodepilot projects

# Run environment health check and coexistence diagnostics
nodepilot doctor

# Manage shell and user PATH integration
nodepilot integration status
nodepilot integration enable
nodepilot integration disable

# Visual GUI Lifecycle Management (Optional, can be added or removed anytime without re-installing)
nodepilot gui status     # Check if visual interface is enabled
nodepilot gui install    # Enable and configure visual desktop/browser interface
nodepilot gui open       # Launch visual GUI dashboard
nodepilot gui remove     # Disable GUI component and return to pure terminal mode

# Check for application updates
nodepilot update
```

---

## 🖥️ Desktop Application

Run the desktop application:

```bash
cd apps/desktop
npm run build
cargo run -p nodepilot-desktop
```

The desktop dashboard provides:
* **Projects**: Search, filter by framework or compatibility status, select multiple projects for bulk assignment, and auto-assign recommended versions with preview diff.
* **Node.js**: View installed runtimes, disk space usage, projects using each runtime, and install official LTS releases with 1 click.
* **Workspaces**: Register root development directories (e.g. `C:\dev` or `/Users/name/dev`) and trigger automatic discovery.
* **Doctor**: Non-destructive environment scanner detailing detected third-party managers and shell integration status.
* **Settings**: Manage terminal preferences (Windows Terminal, PowerShell, CMD, Terminal.app), preferred code editor (VS Code, WebStorm), appearance (Dark, Light, System), and privacy.

---

## 🔒 Security & Privacy

* **SHA-256 Validation**: Every runtime archive downloaded from `nodejs.org` is checked against official cryptographic checksums before extraction.
* **Safe Archive Extraction**: Prevents directory traversal (`ZipSlip`) during extraction.
* **Passive Project Scanning**: Scanning only parses text files; it never runs `npm install`, lifecycle scripts, or project executables.
* **Strict Privacy Default**: Zero tracking, zero hidden telemetry.

---

## 🛠️ Building From Source

Prerequisites:
* **Rust**: 1.80+ (`rustup default stable`)
* **Node.js**: 18+ (for frontend Vite build)
* **OS**: Windows 10/11 or macOS (Intel / Apple Silicon)

```bash
# Clone the repository
git clone https://github.com/mehmetduran932/NodePilot.git
cd NodePilot

# Run unit and integration tests
cargo test --workspace --exclude nodepilot-desktop

# Build CLI and shim binaries
cargo build --release -p nodepilot -p nodepilot-shim

# Build Desktop GUI
cd apps/desktop
npm install
npm run build
cd ../..
cargo build --release -p nodepilot-desktop
```

---

## 📄 License

NodePilot is licensed under the [MIT License](LICENSE).
