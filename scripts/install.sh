#!/usr/bin/env bash
set -e

# NodePilot One-Line Terminal Installer for Unix / macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/mehmetduran932/NodePilot/master/scripts/install.sh | bash

REPO="mehmetduran932/NodePilot"
INSTALL_DIR="${HOME}/.nodepilot/bin"

echo "Installing NodePilot (Cross-Platform Project-Aware Node.js Manager)..."

mkdir -p "${INSTALL_DIR}"
TMP_DIR=$(mktemp -d)

trap 'rm -rf "${TMP_DIR}"' EXIT

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "${OS}" = "darwin" ]; then
    if [ "${ARCH}" = "arm64" ]; then
        ASSET_KEYWORD="macos-arm64"
    else
        ASSET_KEYWORD="macos-x64"
    fi
else
    echo "Notice: Automatic installation archive is currently tailored for macOS. For Linux, build via 'cargo install --git https://github.com/${REPO}'."
    exit 1
fi

echo "Querying latest release from GitHub (${REPO})..."
LATEST_JSON=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest")
DOWNLOAD_URL=$(echo "${LATEST_JSON}" | grep "browser_download_url" | grep -i "${ASSET_KEYWORD}" | cut -d '"' -f 4 | head -n 1)

if [ -z "${DOWNLOAD_URL}" ]; then
    echo "Error: Could not resolve download URL for ${OS}-${ARCH}"
    exit 1
fi

echo "Downloading ${DOWNLOAD_URL}..."
curl -sSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/nodepilot.tar.gz"

echo "Extracting archive..."
tar -xzf "${TMP_DIR}/nodepilot.tar.gz" -C "${TMP_DIR}"

cp "${TMP_DIR}/nodepilot" "${INSTALL_DIR}/nodepilot"
chmod +x "${INSTALL_DIR}/nodepilot"

if [ -f "${TMP_DIR}/nodepilot-shim" ]; then
    cp "${TMP_DIR}/nodepilot-shim" "${INSTALL_DIR}/nodepilot-shim"
    chmod +x "${INSTALL_DIR}/nodepilot-shim"
fi

echo "Enabling shell integration and tool shims..."
"${INSTALL_DIR}/nodepilot" integration enable

echo ""
echo "[v] NodePilot successfully installed to: ${INSTALL_DIR}"
echo "Restart your terminal or run: source ~/.nodepilot/nodepilot.env"
echo "Try: nodepilot --help or nodepilot current"
