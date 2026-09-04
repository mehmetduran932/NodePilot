# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
