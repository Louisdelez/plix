# Client Packaging Guide

This guide covers building and packaging the Plix client for distribution.

## Build Requirements

- Rust 1.83+ (stable)
- Platform-specific tools:
  - **Linux**: tar, gzip, sha256sum
  - **Windows**: PowerShell 5.1+
  - **macOS**: zip, shasum

### Optional (for CEF UI)

- CEF binaries from Spotify CDN
- On macOS: Xcode Command Line Tools for signing

## Building the Client

### Release Build

```bash
# Linux
cargo build --release --bin plix-client

# Windows (from Windows machine)
cargo build --release --bin plix-client --target x86_64-pc-windows-msvc

# macOS
cargo build --release --bin plix-client --target x86_64-apple-darwin
```

### With CEF UI Feature

```bash
cargo build --release --bin plix-client --features cef-ui
```

## Packaging

### Linux

```bash
./scripts/package/client_linux.sh \
    --binary-path target/release/plix-client \
    --version 0.1.0 \
    --output-dir dist/ \
    --assets-dir assets/
```

Output: `dist/plix-client-linux-x86_64-0.1.0.tar.gz`

### Windows

```powershell
.\scripts\package\client_windows.ps1 `
    -BinaryPath target\release\plix-client.exe `
    -Version 0.1.0 `
    -OutputDir dist\ `
    -AssetsDir assets\
```

Output: `dist/plix-client-win64-0.1.0.zip`

### macOS

```bash
./scripts/package/client_macos.sh \
    --binary-path target/release/plix-client \
    --version 0.1.0 \
    --output-dir dist/ \
    --assets-dir assets/
```

Output: `dist/plix-client-macos-0.1.0.zip`

## Bundle Structure

### Linux/Windows

```
plix-client-{platform}-{version}/
├── plix-client{.exe}       # Main binary
├── build_info.json         # Build metadata
├── assets/                 # Game assets
│   ├── ui/                 # UI assets
│   └── arenas/             # Arena definitions
├── config/                 # Default configs
│   └── defaults/
└── cef/                    # CEF runtime (if enabled)
    ├── libcef.so/.dll
    └── ...
```

### macOS (.app Bundle)

```
plix-client-macos-{version}/
├── build_info.json
└── Plix.app/
    └── Contents/
        ├── MacOS/
        │   └── plix-client
        ├── Resources/
        │   └── assets/
        ├── Frameworks/
        │   └── Chromium Embedded Framework.framework/
        ├── Info.plist
        └── PkgInfo
```

## CEF Bundling

### Automatic Download

CEF binaries are downloaded from Spotify CDN during build:

```
https://cef-builds.spotifycdn.com/cef_binary_{version}_{platform}.tar.gz
```

### Manual CEF Setup

1. Download CEF from https://cef-builds.spotifycdn.com/
2. Extract to a known location
3. Pass to packaging script:

```bash
./scripts/package/client_linux.sh \
    --binary-path target/release/plix-client \
    --version 0.1.0 \
    --output-dir dist/ \
    --assets-dir assets/ \
    --cef-dir /path/to/cef
```

### CEF Files Required

**Linux:**
- libcef.so
- icudtl.dat
- resources.pak
- chrome_*_percent.pak
- locales/

**Windows:**
- libcef.dll
- chrome_elf.dll
- d3dcompiler_47.dll
- icudtl.dat
- resources.pak
- chrome_*_percent.pak
- v8_context_snapshot.bin
- locales/

**macOS:**
- Chromium Embedded Framework.framework/
- Helper apps (optional)

## Validation

### Smoke Test

```bash
# Linux/macOS
./scripts/package/smoke_client_bundle.sh dist/plix-client-linux-x86_64-0.1.0.tar.gz

# Windows
.\scripts\package\smoke_client_bundle.ps1 dist\plix-client-win64-0.1.0.zip
```

### Manual Validation

```bash
# Extract
tar xzf plix-client-linux-x86_64-0.1.0.tar.gz
cd plix-client-linux-x86_64-0.1.0

# Verify binary
./plix-client --version
./plix-client --help

# Verify assets
ls -la assets/
ls -la assets/ui/

# Verify build info
cat build_info.json
```

## build_info.json

Every bundle includes `build_info.json`:

```json
{
  "version": "0.1.0",
  "commit_sha": "abc1234567890...",
  "commit_sha_short": "abc1234",
  "build_timestamp": "2025-12-19T14:30:00Z",
  "build_date": "2025-12-19",
  "target_triple": "x86_64-unknown-linux-gnu",
  "rust_version": "1.83.0",
  "branch": "main",
  "is_dirty": false
}
```

## Checksums

Each bundle has a corresponding `.sha256` file:

```bash
# Verify checksum
sha256sum -c plix-client-linux-x86_64-0.1.0.tar.gz.sha256
```

## CI Integration

The release workflow automatically:
1. Builds client for all platforms
2. Packages with platform-specific scripts
3. Runs smoke tests
4. Generates checksums
5. Uploads to GitHub Releases

See `.github/workflows/release.yml` for details.
