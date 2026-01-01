# Release Process

## Prerequisites

1. ✅ All tests passing: `cargo test --workspace`
2. ✅ Benchmark validated: `cargo test --package cue_core --test benchmark -- --ignored`
3. ✅ CHANGELOG.md updated with release notes
4. ✅ Version bumped in `Cargo.toml` (workspace)

## Release Steps

### 1. Version Bump

Update version in **workspace** `Cargo.toml`:

```toml
[workspace.package]
version = "0.2.0"  # Bump this
```

### 2. Update CHANGELOG

Add entry to `docs/01_general/CHANGELOG.md`:

```markdown
## [0.2.0] - 2026-01-XX

### Added
- Feature 1
- Feature 2
...
```

### 3. Commit and Tag

```bash
git add .
git commit -m "chore: bump version to v0.2.0"
git tag v0.2.0
git push origin main --tags
```

### 4. Automated Build

GitHub Actions will automatically (triggered by tag `v*.*.*`):

- ✅ Build binaries for 4 platforms
- ✅ Strip debug symbols (smaller downloads)
- ✅ Package as tar.gz/zip with SHA256 checksums
- ✅ Create GitHub release with all assets

**Expected assets** (for v0.2.0):

```
cue-v0.2.0-x86_64-pc-windows-msvc.zip
cue-v0.2.0-x86_64-pc-windows-msvc.zip.sha256
cue-v0.2.0-x86_64-unknown-linux-gnu.tar.gz
cue-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256
cue-v0.2.0-x86_64-apple-darwin.tar.gz
cue-v0.2.0-x86_64-apple-darwin.tar.gz.sha256
cue-v0.2.0-aarch64-apple-darwin.tar.gz
cue-v0.2.0-aarch64-apple-darwin.tar.gz.sha256
```

### 5. Verify Release

1. Check <https://github.com/TCTri205/CueDeck/releases/latest>
2. Verify all 8 files present (4 archives + 4 checksums)
3. Download one archive and verify checksum:

```bash
# Linux/macOS
sha256sum -c cue-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256

# Windows PowerShell
$expected = Get-Content cue-v0.2.0-x86_64-pc-windows-msvc.zip.sha256
$actual = (Get-FileHash cue-v0.2.0-x86_64-pc-windows-msvc.zip).Hash.ToLower()
if ($expected -match $actual) { "✓ Checksum valid" } else { "✗ Checksum mismatch" }
```

### 6. Test Self-Updater

```bash
# Build older version for testing
git checkout v0.1.0
cargo build --release
./target/release/cue --version  # Should show v0.1.0

# Test upgrade
./target/release/cue upgrade

# Verify
cue --version  # Should show v0.2.0
```

## Manual Release (Workflow Dispatch)

For beta/RC releases without creating a tag:

1. Go to GitHub → Actions → Release
2. Click "Run workflow"
3. Enter version: `v0.2.0-beta.1`
4. Click "Run workflow"

## Rollback

If release has critical bugs:

```bash
# Delete GitHub release
gh release delete v0.2.0 --yes

# Delete local and remote tag
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0

# Revert version bump
git revert HEAD
git push origin main
```

## Troubleshooting

### Build Fails on Specific Platform

- Check GitHub Actions logs for specific runner
- Test locally with cross-compilation:

  ```bash
  rustup target add x86_64-unknown-linux-gnu
  cargo build --release --target x86_64-unknown-linux-gnu
  ```

### Self-Updater Can't Find Release

- Verify repository is public or release is published (not draft)
- Check asset naming matches: `cue-{version}-{target}.{ext}`
- Verify `self_update` configuration in `cmd_upgrade()`:

  ```rust
  .repo_owner("TCTri205")
  .repo_name("CueDeck")
  .bin_name("cue")
  ```

### Checksum Mismatch

- Re-run release workflow (delete release and tag first)
- Verify no corruption during upload

---

## Best Practices

1. **Never force-push tags** - They're immutable release markers
2. **Always test upgrade path** - Build old version and test `cue upgrade`
3. **Keep CHANGELOG updated** - Essential for auto-generated release notes
4. **Use semantic versioning** - Major.Minor.Patch (breaking.feature.fix)
5. **Test on all platforms** - Download and run each binary manually for major releases

---

## CI/CD Architecture

### Release Workflow (`release.yml`)

**Trigger**: Git tag `v*.*.*` or manual workflow dispatch

**Jobs**:

1. **Build**: Multi-platform matrix (Windows, Linux, macOS x64/ARM)
   - Install Rust toolchain
   - Build release binary with target-specific flags
   - Package as archive (tar.gz for Unix, zip for Windows)
   - Generate SHA256 checksums (platform-specific commands)
   - Upload artifacts

2. **Release**: Create GitHub release
   - Download all artifacts
   - Validate asset completeness
   - Create release with auto-generated notes
   - Attach all binaries and checksums

**Key Features**:

- ✅ Asset naming: `cue-{version}-{target}.{ext}` (self_update compatible)
- ✅ Windows checksums: PowerShell `Get-FileHash`
- ✅ Version extraction: Works for both tag and manual triggers
- ✅ Asset validation: Pre-release verification step

### CI Workflow (`ci.yml`)

**Trigger**: Push to `main`/`develop` or pull request

**Jobs**:

1. **Check**: Cargo check on Ubuntu
2. **Test**: Multi-platform testing (Ubuntu, Windows, macOS)
3. **Fmt**: Rustfmt validation
4. **Clippy**: Linting with warnings as errors
5. **Coverage**: Code coverage with codecov upload
6. **Benchmark**: Performance regression detection (PR only)

**Optimizations**:

- Dependency caching with `Swatinem/rust-cache`
- Conditional benchmarks (avoid waste on regular pushes)
- Parallel matrix execution

---

## Release Checklist

Before creating a release:

- [ ] All tests passing (`cargo test --workspace`)
- [ ] Benchmark performance acceptable
- [ ] CHANGELOG.md updated
- [ ] Version bumped in `Cargo.toml`
- [ ] Documentation reflects new features
- [ ] Breaking changes documented
- [ ] Migration guide (if needed)

After release created:

- [ ] Verify all 8 assets uploaded
- [ ] Download and test one binary per platform
- [ ] Verify checksums
- [ ] Test `cue upgrade` from previous version
- [ ] Update documentation website (if exists)
- [ ] Announce release (GitHub discussions, social media, etc.)
