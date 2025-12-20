# CI Artifacts Guide

This guide covers the artifacts produced by the Plix CI/CD pipeline.

## Release Artifacts

On each tagged release (e.g., `v0.1.0`), the CI produces:

### Client Bundles

| Platform | Artifact | Format |
|----------|----------|--------|
| Linux x86_64 | `plix-client-linux-x86_64-{version}.tar.gz` | tar.gz |
| Windows x64 | `plix-client-win64-{version}.zip` | zip |
| macOS | `plix-client-macos-{version}.zip` | zip |

### Server Bundles

| Platform | Artifact | Format |
|----------|----------|--------|
| Linux x86_64 | `plix-server-headless-linux-x86_64-{version}.tar.gz` | tar.gz |
| Windows x64 | `plix-server-headless-win64-{version}.zip` | zip |
| macOS | `plix-server-headless-macos-{version}.zip` | zip |

### Checksums

Each artifact has a corresponding SHA256 checksum file:
- `plix-client-linux-x86_64-{version}.tar.gz.sha256`
- etc.

## Downloading Artifacts

### From GitHub Releases

1. Go to [Releases](https://github.com/your-org/plix/releases)
2. Find the desired version
3. Download the artifact for your platform
4. Optionally verify with checksum

### From CI Artifacts (Pre-release)

During development, artifacts are available from GitHub Actions:

1. Go to [Actions](https://github.com/your-org/plix/actions)
2. Find the workflow run
3. Download from "Artifacts" section

### Using GitHub CLI

```bash
# List releases
gh release list

# Download from latest release
gh release download --pattern '*.tar.gz'

# Download specific version
gh release download v0.1.0 --pattern 'plix-client-linux*'

# Download all artifacts
gh release download v0.1.0
```

### Using curl

```bash
# Get release info
curl -s https://api.github.com/repos/your-org/plix/releases/latest | \
    jq -r '.assets[].browser_download_url'

# Download specific asset
curl -LO https://github.com/your-org/plix/releases/download/v0.1.0/plix-client-linux-x86_64-0.1.0.tar.gz
```

## Verifying Downloads

### SHA256 Checksum

```bash
# Download checksum file
curl -LO https://github.com/your-org/plix/releases/download/v0.1.0/plix-client-linux-x86_64-0.1.0.tar.gz.sha256

# Verify
sha256sum -c plix-client-linux-x86_64-0.1.0.tar.gz.sha256
```

```powershell
# Windows
(Get-FileHash plix-client-win64-0.1.0.zip -Algorithm SHA256).Hash -eq (Get-Content plix-client-win64-0.1.0.zip.sha256).Split()[0]
```

## CI Workflow

### Trigger Conditions

The release workflow runs on:
- Push to tags matching `v*` (e.g., `v0.1.0`, `v1.0.0-beta.1`)
- Manual dispatch (workflow_dispatch)

### Build Matrix

```yaml
matrix:
  include:
    - os: ubuntu-latest
      target: x86_64-unknown-linux-gnu
      platform_name: linux-x86_64

    - os: windows-latest
      target: x86_64-pc-windows-msvc
      platform_name: win64

    - os: macos-latest
      target: x86_64-apple-darwin
      platform_name: macos
```

### Pipeline Steps

1. **Checkout**: Clone repository
2. **Setup Rust**: Install Rust toolchain
3. **Cache**: Restore Cargo cache (Swatinem/rust-cache)
4. **Build Client**: `cargo build --release --bin plix-client`
5. **Build Server**: `cargo build --release --bin plix-server-headless`
6. **Package Client**: Run platform-specific packaging script
7. **Package Server**: Run platform-specific packaging script
8. **Smoke Tests**: Validate bundles
9. **Upload Artifacts**: Upload to workflow artifacts
10. **Release**: Create GitHub release (on tag push)

## Manual Trigger

Trigger a release build manually:

```bash
# Using GitHub CLI
gh workflow run release.yml --ref main -f version=0.1.0

# Or from the GitHub Actions UI
```

## Artifact Naming Convention

```
plix-{component}-{platform}-{version}.{extension}
```

Where:
- `{component}`: `client` or `server-headless`
- `{platform}`: `linux-x86_64`, `win64`, `macos`
- `{version}`: Semantic version (e.g., `0.1.0`)
- `{extension}`: `tar.gz` (Linux) or `zip` (Windows/macOS)

Examples:
- `plix-client-linux-x86_64-0.1.0.tar.gz`
- `plix-server-headless-win64-0.1.0.zip`
- `plix-client-macos-1.0.0-beta.1.zip`

## Retention Policy

- **Workflow Artifacts**: 7 days
- **Release Assets**: Permanent (until release is deleted)

## Cache Configuration

The workflow uses Swatinem/rust-cache for faster builds:

- Cache key based on `Cargo.lock` hash
- Separate cache per target triple
- Typical cache hit rate: 60-80%
- Build time reduction: 40-60%

## Troubleshooting CI

### Build Failures

Check the workflow logs:
```bash
gh run view --log-failed
```

### Artifact Download Issues

```bash
# List available artifacts
gh run view --json artifacts

# Download specific artifact
gh run download <run-id> -n plix-client-linux-x86_64
```

### Cache Issues

Clear cache manually if needed:
```bash
gh cache delete --all
```

Or wait for cache expiry (7 days).
