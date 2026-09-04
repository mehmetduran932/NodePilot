# Package Manager Distribution Guide

NodePilot supports automated distribution via Homebrew (macOS) and WinGet (Windows).

---

## 🍎 macOS Homebrew Cask

Create or update the cask recipe at `Casks/nodepilot.rb`:

```ruby
cask "nodepilot" do
  arch arm: "aarch64", intel: "x64"

  version "0.1.0"
  sha256 arm:   "<INSERT_MACOS_ARM64_SHA256>",
         intel: "<INSERT_MACOS_X64_SHA256>"

  url "https://github.com/nodepilot/nodepilot/releases/download/v#{version}/NodePilot_#{version}_#{arch}.dmg"
  name "NodePilot"
  desc "Project-aware Node.js runtime manager and desktop dashboard"
  homepage "https://github.com/nodepilot/nodepilot"

  auto_updates true

  app "NodePilot.app"
  binary "#{appdir}/NodePilot.app/Contents/MacOS/nodepilot"

  zap trash: [
    "~/.nodepilot",
    "~/Library/Application Support/com.nodepilot.desktop",
    "~/Library/Caches/com.nodepilot.desktop",
    "~/Library/Preferences/com.nodepilot.desktop.plist",
  ]
end
```

### Installation & Upgrade Commands

```bash
# Install
brew install --cask nodepilot

# Upgrade
brew upgrade --cask nodepilot
```

---

## 🪟 Windows Package Manager (WinGet)

Create the manifest files under `manifests/n/NodePilot/NodePilot/<version>/`:

### `NodePilot.NodePilot.yaml`
```yaml
PackageIdentifier: NodePilot.NodePilot
PackageVersion: 0.1.0
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
```

### `NodePilot.NodePilot.installer.yaml`
```yaml
PackageIdentifier: NodePilot.NodePilot
PackageVersion: 0.1.0
InstallerType: msi
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/nodepilot/nodepilot/releases/download/v0.1.0/NodePilot_0.1.0_x64_en-US.msi
    InstallerSha256: <INSERT_WINDOWS_X64_SHA256>
ManifestType: installer
ManifestVersion: 1.6.0
```

### Installation & Upgrade Commands

```powershell
# Install
winget install NodePilot.NodePilot

# Upgrade
winget upgrade NodePilot.NodePilot
```
