# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

Security and privacy are foundational principles for **NodePilot**.

If you discover a security vulnerability:
1. **Do not** open a public GitHub issue.
2. Please report the vulnerability privately via **[GitHub Private Security Advisory](https://github.com/mehmetduran932/NodePilot/security/advisories/new)** or by reaching out directly to the repository maintainer.
3. Include details of the vulnerability, affected components (e.g. shim router, runtime installer, config parser, or desktop GUI), and minimal reproduction steps.

We appreciate responsible disclosure and will respond promptly to verify, assess, and release patched versions.

## Security Architecture & Boundaries

NodePilot is designed with defensive security controls:
- **Cryptographic Verification**: All downloaded Node.js runtime packages are verified against official SHA-256 checksums from `nodejs.org` before extraction.
- **Passive Project Scanning**: Project inspection parses structured files (`package.json`, `.nvmrc`, etc.) purely as static data. NodePilot never invokes package managers, executes project scripts, or runs untrusted local dependencies during scans.
- **Path Traversal Protection**: Archive extractions rigorously validate all member paths to prevent directory traversal (`ZipSlip`) attacks.
- **Least Privilege Principle**: NodePilot operates entirely in user space (`%LOCALAPPDATA%` on Windows, `~/.local/share` on macOS/Linux) and does not require administrative / root privileges for runtime management or execution.
