# Contributing to NodePilot

Thank you for your interest in contributing to NodePilot!

NodePilot is an open-source tool designed to manage Node.js runtimes around projects rather than global shell state.

## 🛠️ Development Setup

1. **Prerequisites**:
   * Rust 1.80+ (`rustc`, `cargo`)
   * Node.js 18+ and npm
   * Windows 10/11 or macOS

2. **Clone & Build**:
   ```bash
   git clone https://github.com/nodepilot/nodepilot.git
   cd nodepilot

   # Run tests
   cargo test --workspace --exclude nodepilot-desktop

   # Build CLI & Shim
   cargo build -p nodepilot -p nodepilot-shim
   ```

3. **Frontend Build**:
   ```bash
   cd apps/desktop
   npm install
   npm run build
   ```

## 📐 Architecture Principles

* **No Global State Collisions**: Never switch active Node globally. Keep runtime execution bound to directory context.
* **Non-destructive Coexistence**: Never delete, rewrite, or damage third-party managers (nvm, fnm, Volta, mise).
* **Passive Project Scanning**: Never execute scripts or package lifecycle hooks when discovering projects.
* **Strong Typing**: Use typed Rust errors (`thiserror`); avoid `unwrap()` in recoverable paths.

## 🧪 Testing

Always ensure that new features are accompanied by automated tests:
```bash
cargo test --workspace --exclude nodepilot-desktop
```

## 📝 Pull Request Guidelines

1. Fork the repo and create a branch: `feature/my-feature` or `fix/my-bugfix`.
2. Format code: `cargo fmt`.
3. Check lints: `cargo clippy --workspace`.
4. Run test suites.
5. Submit your PR with a clear description of the problem solved.
