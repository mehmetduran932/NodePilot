# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in NodePilot, please do **NOT** open a public GitHub issue.

Please report the vulnerability by emailing the security team or opening a confidential security advisory on GitHub:

* Email: `security@nodepilot.dev`

Please include:
* Description of the vulnerability
* Steps to reproduce or proof-of-concept
* Potential impact and affected operating systems

## Security Commitments

* **Cryptographic Verification**: NodePilot validates the official SHA-256 checksum of every downloaded Node.js distribution before extraction.
* **Extraction Path Traversal Defense**: All archive extraction routines validate paths to prevent directory traversal (`ZipSlip`).
* **Passive Execution**: NodePilot never executes `postinstall`, `npm install`, or untrusted project scripts during project discovery.
* **Telemetry**: NodePilot does not collect or transmit user environment secrets or source code.
