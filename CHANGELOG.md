# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-09-24

### Added
- `nodepilot update` now updates NodePilot itself: it downloads this platform's release archive, replaces `nodepilot` / `nodepilot-shim` in place (atomic rename on macOS, rename-aside on Windows), and refreshes the tool shims. Use `--check` to only check and `--force` to reinstall/repair.
- README: "Updating" and "Good to Know" sections (routing rules, supported version specs, nvm coexistence, Windows System-vs-User PATH order, macOS shell integration, troubleshooting).

### Fixed
- The update command no longer suggests non-existent `brew` / `winget` packages.

## [0.1.1] - 2026-09-24

### Fixed
- Tool shims (`node`, `npm`, `npx`, `ng`, `yarn`, `pnpm`, `tsc`, `vite`, ...) are now actually created by `nodepilot integration enable`, `nodepilot setup` and the desktop app, so tools like `ng build` route through NodePilot.
- Version specs such as `20`, `v20.19`, `lts/*`, `lts/iron`, `>=18.19` and `^20 || ^22` resolve to the best installed runtime (previously only exact versions worked, causing repeated failed auto-installs).
- nvm / system Node coexistence: directories without an assigned version, and loose `engines` ranges not satisfied locally, pass through to the next `node` on PATH instead of failing.
- Tools installed only elsewhere on PATH (e.g. an nvm global `ng`) run with the project's Node runtime instead of erroring.
- Nested tool probes (e.g. `pnpm --version` during `ng serve`) no longer trip the recursion guard or trigger global installs; the guard is now a depth limit.
- Installed versions are ordered numerically (22.10.0 > 22.9.0 > 9.0.0).
- macOS/Linux: `integration enable` now sources `~/.nodepilot/nodepilot.env` from `~/.zshrc` (and existing bash rc files); `disable` removes it.
- Shim tool name is taken from `argv[0]`, fixing symlinked shims on Linux.
- `install.sh` no longer starts with a UTF-8 BOM; update checks point at this repository.
- Release workflow builds the macOS x64 binary (the retired `macos-13` runner blocked the v0.1.0 release).

## [0.1.0] - 2026-09-04

### Added
- **Core Architecture**:
  - Isolated Node.js runtime manager with SHA-256 checksum verification and atomic extraction.
  - Multi-level configuration resolution engine (`.nodepilot.local`, `.nvmrc`, `.node-version`, `volta.node`, `engines.node`, `.tool-versions`, `mise.toml`, parent directory inheritance, global defaults).
  - Idempotent `.gitignore` updater with CRLF and LF newline style preservation.
  - Read-only third-party manager inspector (`nvm`, `fnm`, `Volta`, `mise`, `asdf`, system Node) ensuring zero conflict.
  - Reversible shell and PATH integration manager with rollback backup.
- **Routing & Shim**:
  - Sub-millisecond routing layer for `node`, `npm`, `npx`, and `corepack` executables.
  - Loop protection (`NODEPILOT_SHIM_ACTIVE`).
  - Active runtime bin directory prepending for isolated global package and subprocess execution.
- **Scanner**:
  - Passive recursive project scanner with heavy directory pruning (`node_modules`, `.git`, `dist`, `build`, `target`, `.next`, etc.).
  - Framework detection for Angular, React, Next.js, Vue, Nuxt, Svelte, SvelteKit, NestJS, Express, Vite, Nx.
  - Package manager detection from lockfiles (`pnpm-lock.yaml`, `yarn.lock`, `package-lock.json`, `bun.lockb`).
  - Monorepo detection for pnpm workspaces, Nx, and npm/yarn workspaces.
- **Compatibility Engine**:
  - Static matrix for Angular and Next.js Node version requirements.
  - Automatic version recommendation calculation and bulk auto-assign preview diff.
- **CLI (`nodepilot`)**:
  - Human-readable commands: `version`, `install`, `uninstall`, `list`, `current`, `assign`, `unassign`, `scan`, `projects`, `workspaces`, `doctor`, `integration`, `update`, `logs`.
  - Interactive selection prompt via `dialoguer` for `nodepilot assign`.
- **Desktop Application (Tauri 2 + React + TypeScript)**:
  - Projects dashboard with search, framework filter, compatibility filter, sorting, bulk actions, and auto-assign diff modal.
  - Project details drawer with one-click terminal, code editor, folder launch, and version assignment.
  - Installed Node runtimes manager with disk usage breakdown and deletion warnings for active versions.
  - Workspace root directory manager.
  - Doctor diagnostics screen.
  - 5-step initial onboarding wizard.
  - Dark, light, and system theme switcher.
