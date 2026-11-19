# Security & Transparency Policy

## 100% Open Source & Malware-Free

**gity x** is completely open source under the MIT License and contains **zero malware, zero spyware, zero telemetry, and zero ads**.

- **Local-only execution**: All Git operations run directly through your system's `git` binary. `gity x` never sends your code, credentials, or analytics to any third-party server.
- **Reproducible CI builds**: Every official installer (`.dmg`, `.exe`, `.deb`, `.AppImage`) in [GitHub Releases](https://github.com/williamjkc69/gity-x/releases) is compiled directly from this public repository's source code via transparent [GitHub Actions workflows](.github/workflows/release.yml).
- **Verifiable checksums**: Every release includes `SHA256SUMS.txt` so you can verify file integrity before running.

## Why macOS / Windows May Show an "Unverified Developer" Warning

Independent open-source apps that are not signed with paid Apple Developer ID ($99/year) or Windows Code Signing certificates trigger default OS warnings:

- **macOS Gatekeeper**: *"Apple could not verify 'gity x.app' is free of malware"* — this message does **not** mean malware was found; it simply means the binary is not notarized with a paid Apple Developer account.
- **Windows SmartScreen**: *"Windows protected your PC"* — shown for newly released unsigned executables.

You can inspect every line of code in `src/` or build from source with `cargo build --release`.

## Reporting a Vulnerability

If you discover a security issue, please open a private security advisory on GitHub or report it via [GitHub Issues](https://github.com/williamjkc69/gity-x/issues).
