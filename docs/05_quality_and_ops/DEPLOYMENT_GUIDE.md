# Deployment Guide

This document covers installation and distribution of CueDeck binaries.

## 1. Binary Distribution

### GitHub Releases

CueDeck binaries are distributed via GitHub Releases. Each release includes:

| Platform | Binary | Archive |
| :--- | :--- | :--- |
| **Linux x64** | `cue` | `cue-linux-x64.tar.gz` |
| **macOS x64** | `cue` | `cue-darwin-x64.tar.gz` |
| **macOS ARM** | `cue` | `cue-darwin-arm64.tar.gz` |
| **Windows x64** | `cue.exe` | `cue-windows-x64.zip` |

## 2. Installation

### Linux / macOS (curl)

```bash
# Download latest release
curl -fsSL https://github.com/cuedeck/cuedeck/releases/latest/download/cue-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz | tar xz

# Move to PATH
sudo mv cue /usr/local/bin/

# Verify
cue --version
```

### Windows (PowerShell)

```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/cuedeck/cuedeck/releases/latest/download/cue-windows-x64.zip" -OutFile "$env:TEMP\cue.zip"
Expand-Archive -Path "$env:TEMP\cue.zip" -DestinationPath "$env:LOCALAPPDATA\Programs\CueDeck"

# Add to PATH (User)
$env:PATH += ";$env:LOCALAPPDATA\Programs\CueDeck"
[Environment]::SetEnvironmentVariable("PATH", $env:PATH, "User")

# Verify
cue --version
```

### Cargo (from source)

```bash
cargo install cuedeck
```

## 3. Verification

After installation, verify the setup:

```bash
# Check version
cue --version
# Expected: cuedeck 2.1.0

# Run doctor
cue doctor
# Expected: All checks passed
```

## 4. Upgrading

Use the built-in upgrade command:

```bash
cue upgrade
```

This will:

1. Check for the latest version on GitHub Releases
2. Download the appropriate binary
3. Verify checksum
4. Replace the current binary (atomic swap)

## 5. CI/CD Pipeline

### GitHub Actions Release Workflow

**File**: `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: cue-linux-x64
          - os: macos-latest
            target: x86_64-apple-darwin
            name: cue-darwin-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: cue-darwin-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: cue-windows-x64
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build Release
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: target/${{ matrix.target }}/release/cue*

  publish:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Download All Artifacts
        uses: actions/download-artifact@v4
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            cue-linux-x64/*
            cue-darwin-x64/*
            cue-darwin-arm64/*
            cue-windows-x64/*
```

## 6. Release Checklist

Before tagging a release:

| Step | Command / Action | Verify |
| :--- | :--- | :--- |
| 1. Tests Pass | `cargo test --workspace` | All green ✅ |
| 2. Clippy Clean | `cargo clippy -- -D warnings` | No warnings |
| 3. Version Bump | Edit `Cargo.toml` version | Matches tag |
| 4. Changelog | Update `CHANGELOG.md` | Entry exists |
| 5. Doctor | `cargo run -- doctor` | All checks pass |
| 6. Tag | `git tag v2.1.0` | Semantic version |
| 7. Push | `git push origin v2.1.0` | Triggers CI |

## 7. Uninstallation

### Linux / macOS

```bash
sudo rm /usr/local/bin/cue
rm -rf ~/.config/cuedeck  # Global config (optional)
```

### Windows

```powershell
Remove-Item "$env:LOCALAPPDATA\Programs\CueDeck" -Recurse
```

---
**Related Docs**: [CLI_REFERENCE.md](../04_tools_and_data/CLI_REFERENCE.md), [CONTRIBUTING.md](../01_general/CONTRIBUTING.md), [TESTING_STRATEGY.md](./TESTING_STRATEGY.md)
