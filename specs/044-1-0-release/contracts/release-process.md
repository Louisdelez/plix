# Contract: Release Process

**Feature**: 044-1-0-release | **Date**: 2025-12-20

## Overview

Automated release workflow for creating signed, verified release artifacts.

## Release Workflow

### Step 1: Version Bump

```bash
# Update workspace version in Cargo.toml
# Cargo.toml: version = "1.0.0"

# Commit the version change
git add Cargo.toml
git commit -m "chore: bump version to 1.0.0"
```

### Step 2: Create GPG-Signed Tag

```bash
# Create signed tag with release notes
git tag -s v1.0.0 -m "Release v1.0.0

## Highlights
- First stable release
- Full multiplayer support
- Mod API v1.0

## Breaking Changes
None (initial release)

## Migration
No migration needed for new installations.
See docs/release/migration-guide.md for upgrades."
```

### Step 3: Push Tag

```bash
git push origin v1.0.0
```

## CI/CD Workflow

```yaml
# .github/workflows/release.yml

name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  verify-tag:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Verify GPG signature
        run: |
          git verify-tag ${{ github.ref_name }} || {
            echo "ERROR: Tag is not GPG-signed"
            exit 1
          }

  build:
    needs: verify-tag
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: plix-linux-x86_64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: plix-windows-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: plix-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: plix-macos-arm64
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build client
        run: cargo build --release --target ${{ matrix.target }} -p plix-client

      - name: Build server
        run: cargo build --release --target ${{ matrix.target }} -p plix-server

      - name: Package artifacts
        run: |
          mkdir -p dist
          # Platform-specific packaging
          tar -czf dist/${{ matrix.artifact }}-client.tar.gz \
            -C target/${{ matrix.target }}/release plix-client
          tar -czf dist/${{ matrix.artifact }}-server.tar.gz \
            -C target/${{ matrix.target }}/release plix-server

      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: dist/

  checksums:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Generate SHA-256 checksums
        run: |
          cd artifacts
          find . -name "*.tar.gz" -exec sha256sum {} \; > SHA256SUMS.txt
          cat SHA256SUMS.txt

      - uses: actions/upload-artifact@v4
        with:
          name: checksums
          path: artifacts/SHA256SUMS.txt

  release:
    needs: [build, checksums]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4

      - uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            artifacts/**/*.tar.gz
            artifacts/checksums/SHA256SUMS.txt
          body_path: docs/release/CHANGELOG.md
          draft: false
          prerelease: ${{ contains(github.ref_name, 'alpha') || contains(github.ref_name, 'beta') || contains(github.ref_name, 'rc') }}
```

## Artifact Naming Convention

```
plix-{platform}-{arch}-{component}.tar.gz

Examples:
- plix-linux-x86_64-client.tar.gz
- plix-linux-x86_64-server.tar.gz
- plix-windows-x86_64-client.tar.gz
- plix-windows-x86_64-server.tar.gz
- plix-macos-x86_64-client.tar.gz
- plix-macos-arm64-client.tar.gz
```

## SHA-256 Checksum File Format

```
# SHA256SUMS.txt

e3b0c44298fc1c149afbf4c8996fb924...  plix-linux-x86_64-client.tar.gz
a7ffc6f8bf1ed76651c14756a061d662...  plix-linux-x86_64-server.tar.gz
...
```

## Verification Instructions

```bash
# Download artifact and checksum file
curl -LO https://github.com/org/plix/releases/download/v1.0.0/plix-linux-x86_64-client.tar.gz
curl -LO https://github.com/org/plix/releases/download/v1.0.0/SHA256SUMS.txt

# Verify checksum
sha256sum -c SHA256SUMS.txt --ignore-missing

# Expected output:
# plix-linux-x86_64-client.tar.gz: OK
```

## Version Display Locations

| Location | How to Access | Format |
|----------|---------------|--------|
| Client startup log | Automatic | `Plix Client 1.0.0 (abc1234) built 2025-12-20` |
| Client About panel | Menu → About | Full version info |
| Server startup log | Automatic | `Plix Server 1.0.0 (abc1234) built 2025-12-20` |
| CLI --version | `plix-client --version` | `plix-client 1.0.0` |
| Mod API | `plix_mod_core::MOD_API_VERSION` | `ModApiVersion { major: 1, minor: 0 }` |

## Pre-Release Checklist

```markdown
## Release Checklist: v1.0.0

### Code Quality
- [ ] All tests pass (`cargo test --all`)
- [ ] No clippy warnings (`cargo clippy --all-targets`)
- [ ] Code formatted (`cargo fmt --all --check`)
- [ ] No experimental features enabled by default

### Documentation
- [ ] CHANGELOG.md updated
- [ ] Migration guide complete
- [ ] Known issues documented
- [ ] User docs reviewed

### Governance
- [ ] LICENSE file present (MIT)
- [ ] README.md up to date
- [ ] CONTRIBUTING.md present
- [ ] CODE_OF_CONDUCT.md present
- [ ] SECURITY.md present
- [ ] ROADMAP.md published

### Artifacts
- [ ] Builds succeed on all platforms
- [ ] Version string correct in all locations
- [ ] SHA-256 checksums generated
- [ ] Tag GPG-signed

### Smoke Tests
- [ ] Client launches on Windows
- [ ] Client launches on Linux
- [ ] Client launches on macOS
- [ ] Server starts headless
- [ ] Client connects to server
- [ ] Tutorial quest completable
- [ ] Dungeon completable
```
