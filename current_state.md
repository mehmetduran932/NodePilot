# NodePilot — current state

Working note for resuming this project in a new session. Update it whenever a
release ships or a known issue is found or fixed.

Snapshot date: 2026-09-24 · latest release **v0.1.2** (commit `ecc2906` on `master`).

## What this is

A project-aware Node.js version manager for Windows 10/11 and macOS (Apple
Silicon and Intel). Shims named `node`, `npm`, `ng` and so on resolve the
project's version spec per invocation and run the matching runtime, without
any global `nvm use`.

```
crates/core           paths, config/state, resolution (.nodepilot.local, .nvmrc, engines, ...),
                      version_match (spec → installed runtime), shell/PATH integration
crates/shim           nodepilot-shim binary (router) + install_shims / deploy_tool_shims
crates/cli            `nodepilot` CLI
crates/node-runtime   release index, SHA-256 verified download, archive extraction
crates/updater        GitHub release check + self-update
crates/scanner        passive project/framework/package-manager scanner
crates/compatibility  framework ↔ Node matrix, auto-assign diff
apps/desktop          Tauri 2 + React companion GUI (optional)
scripts/              install.sh (macOS), install.ps1 / uninstall.ps1 (Windows)
```

## Releases

| Version | Content |
| --- | --- |
| v0.1.0 | Initial release. The macOS x64 job hung for 24h on the retired `macos-13` runner, so this release has no Intel Mac asset. |
| v0.1.1 | Tool shims are actually created (`install_shims` was never called before). Adds nvm passthrough, version-spec matching (`20`, `lts/*`, `>=18.19`, `^20 \|\| ^22`) and a nested-call depth guard. Adds the `~/.zshrc` integration block. |
| v0.1.2 | `nodepilot update [--check\|--force]` updates NodePilot itself. README gains "Updating" and "Good to Know" sections. |

Every release ships `nodepilot-macos-arm64.tar.gz`, `nodepilot-macos-x64.tar.gz`,
`nodepilot-windows-x64.zip`, `install.sh` and `install.ps1`.

**v0.1.0 and v0.1.1 cannot update themselves.** Users on those versions must re-run the one-line
installer once. From v0.1.2 on, `nodepilot update` is enough.

## How to release

1. Bump `version` in `[workspace.package]` of `Cargo.toml`, `apps/desktop/package.json`
   and `apps/desktop/src-tauri/tauri.conf.json`, then run
   `npm install --package-lock-only` in `apps/desktop`.
2. Add a `## [x.y.z]` section to `CHANGELOG.md`.
3. Open a PR. `build.yml` must be green on both windows-latest and macos-latest, because
   cfg-gated clippy warnings only show up on the other OS.
4. Merge the PR, then run `git tag -a vX.Y.Z -m ... && git push origin vX.Y.Z`. `release.yml` builds
   all three archives: the Intel binary is cross-compiled on `macos-latest`, and the job
   timeout is 60 min.
5. The workflow's auto-generated notes are duplicated. Replace them with the CHANGELOG
   section plus install/upgrade instructions (`gh release edit --notes-file`), and upload
   `scripts/install.sh` and `scripts/install.ps1` to the release.

## Verifying changes

- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --exclude nodepilot-desktop`
- `cd apps/desktop && npm run build`
- For shim and integration work, use a **throwaway HOME**:
  `HOME=<scratch dir> ./target/debug/nodepilot integration enable`. The paths
  honor `$HOME`. Never run `integration enable` against the real HOME, because it
  edits `~/.zshrc`. Fake runtimes are shell scripts under
  `$HOME/.nodepilot/versions/<ver>/bin/{node,npm}`.

## Known issues / remaining work

1. **Windows cmd.exe / IDE terminals.** The System PATH comes before the User PATH. If
   nvm-windows or the Node installer put `C:\Program Files\nodejs` on the System PATH,
   cmd.exe runs that `node` instead of NodePilot's; PowerShell is fixed via its
   profile. This is documented in the README but has no code fix. Options: a doctor
   warning that detects the problem, or an opt-in cmd `AutoRun` prepend.
2. **Windows is verified only by CI.** It compiles, and clippy and tests pass, but nobody
   has run it on a real machine. Untested on Windows: shim routing, the `.exe.old`
   rename in self-update, and shim copies (`node.exe`, ...) that are in use during an
   update. Copy errors there are ignored.
3. **No checksums for NodePilot's own release assets.** Self-update trusts the HTTPS
   download from GitHub. Adding a `SHA256SUMS` asset to `release.yml` and verifying it
   in `crates/updater` would close this.
4. **Unix tool import copies files instead of symlinks.** When `try_import_tool_from_nvm_or_versions`
   copies another version's `bin/<tool>`, it follows the symlink and copies the JS
   entry file, which can break relative requires. This is a rare path.
5. **Shell coverage.** Only zsh and bash rc files are handled; fish and nushell need
   PATH set manually. Linux has no release asset (`install.sh` exits there; use
   `cargo install`).
6. **GitHub Actions warning about Node 20.** `actions/checkout@v4` and
   `softprops/action-gh-release@v2` are forced onto Node 24. Bump them when
   new majors are out.
