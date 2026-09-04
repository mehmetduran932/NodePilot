# Contributing to NodePilot

Thank you for your interest in contributing to **NodePilot**! We welcome bug reports, feature requests, documentation improvements, and code contributions.

---

## 🍴 Contribution Workflow (Fork & Pull Request)

To maintain code quality and keep the main repository clean, all external contributions follow the **Fork & Pull Request** model.

### Step 1: Fork the Repository
1. Click the **Fork** button at the top right of [NodePilot Repository](https://github.com/mehmetduran932/NodePilot).
2. Clone your forked repository locally:
   ```powershell
   git clone https://github.com/<your-username>/NodePilot.git
   cd NodePilot
   ```

### Step 2: Create a Dedicated Feature Branch
Branch names must follow descriptive prefixes:
- `feature/<name>` for new features (e.g., `feature/bun-runtime-support`)
- `fix/<name>` or `bugfix/<name>` for bug fixes (e.g., `fix/shim-elevation-handling`)
- `docs/<name>` for documentation updates (e.g., `docs/update-cli-guide`)
- `refactor/<name>` for code refactoring (e.g., `refactor/compatibility-engine`)
- `test/<name>` for adding or updating unit and integration tests

```powershell
git checkout -b feature/my-feature-name
```

### Step 3: Implement & Test Your Changes
- Ensure your changes adhere to idiomatic Rust standards:
  ```powershell
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  ```
- Run the test suite before committing:
  ```powershell
  cargo test --workspace --exclude nodepilot-desktop
  ```
- If you made changes to the desktop GUI (`apps/desktop`):
  ```powershell
  cd apps/desktop
  npm test
  npm run build
  cd ../..
  ```

### Step 4: Commit Your Changes
Use conventional commit messages:
```powershell
git commit -m "feat(shim): resolve ecosystem binaries through active node prefix"
git commit -m "fix(installer): handle interrupted archive download gracefully"
```

### Step 5: Push and Open a Pull Request
1. Push the branch to your fork:
   ```powershell
   git push origin feature/my-feature-name
   ```
2. Navigate to the original [NodePilot Repository](https://github.com/mehmetduran932/NodePilot) and click **New Pull Request**.
3. Fill out the PR description with:
   - Summary of changes
   - Problem solved or feature added
   - Testing steps performed and verification screenshots (if UI changes)

---

## 🔒 Branch Protection & CI Policies

- **Protected Branches**: `master` and `main` branches are protected. Direct pushes are disabled.
- **Review Requirements**: Every PR requires review and approval before merging.
- **CI Status**: All automated GitHub Actions builds, tests, and formatting checks must pass.
- **Branch Retention**: Merged PR branches are preserved in repository history.

---

## 🐞 Reporting Issues & Bugs

If you find a bug or have a suggestion:
1. Check the [Issues Tab](https://github.com/mehmetduran932/NodePilot/issues) to ensure it has not already been reported.
2. Open a new issue with detailed reproduction steps, OS version, Node.js versions involved, and relevant logs.
