# Packaging Contracts

**Feature**: 041-cross-platform | **Date**: 2025-12-19

This document defines the contracts (file structures, naming conventions, validation rules) for packaging scripts and CI workflows.

---

## 1. Bundle Layout Contracts

### 1.1 Client Bundle Layout

All client bundles MUST follow this structure:

```
plix-client-{platform}-{version}/
├── plix-client{.exe}           # Main executable (required)
├── build_info.json             # Build metadata (required)
├── assets/                     # Game assets (required)
│   ├── ui/                     # CEF UI assets
│   │   ├── index.html
│   │   └── ...
│   └── arenas/                 # Arena definitions
├── config/                     # Default configs (optional)
│   └── defaults/
└── cef/                        # CEF runtime (if cef-ui enabled)
    ├── {platform-specific CEF files}
    └── ...
```

**Platform-specific variations:**

| Platform | Executable | CEF Location | Archive Format |
|----------|------------|--------------|----------------|
| Windows | `plix-client.exe` | `cef/*.dll` | `.zip` |
| Linux | `plix-client` | `cef/*.so` | `.tar.gz` |
| macOS | `Plix.app/Contents/MacOS/plix-client` | `Plix.app/Contents/Frameworks/` | `.zip` |

### 1.2 Server Bundle Layout

All server bundles MUST follow this structure:

```
plix-server-headless-{platform}-{version}/
├── plix-server-headless{.exe}  # Main executable (required)
├── build_info.json             # Build metadata (required)
├── configs/                    # Example configs (required)
│   └── examples/
│       ├── server.toml
│       └── server_mods.toml
├── docs/                       # Documentation (required)
│   └── README.md
├── run_server.sh               # Linux/macOS run script
└── run_server.ps1              # Windows run script
```

---

## 2. Naming Convention Contracts

### 2.1 Archive Naming

Archives MUST follow this pattern:

```
plix-{component}-{platform}-{version}.{extension}
```

| Component | Values |
|-----------|--------|
| `{component}` | `client`, `server-headless` |
| `{platform}` | `win64`, `linux-x86_64`, `macos` |
| `{version}` | Semantic version (e.g., `0.1.0`) |
| `{extension}` | `zip` (Windows/macOS), `tar.gz` (Linux) |

**Examples:**
- `plix-client-win64-0.1.0.zip`
- `plix-client-linux-x86_64-0.1.0.tar.gz`
- `plix-client-macos-0.1.0.zip`
- `plix-server-headless-linux-x86_64-0.1.0.tar.gz`

### 2.2 CI Artifact Naming

GitHub Actions artifacts MUST follow this pattern:

```
plix-{target-triple}
```

**Examples:**
- `plix-x86_64-unknown-linux-gnu`
- `plix-x86_64-pc-windows-msvc`
- `plix-x86_64-apple-darwin`

---

## 3. Validation Contracts

### 3.1 Pre-Archive Validation

Packaging scripts MUST validate before creating archive:

```bash
# Required files check
validate_required_files() {
    local bundle_dir=$1
    local binary=$2

    # Binary must exist and be executable
    [ -x "$bundle_dir/$binary" ] || exit 1

    # build_info.json must exist
    [ -f "$bundle_dir/build_info.json" ] || exit 1

    # Assets directory must exist
    [ -d "$bundle_dir/assets" ] || exit 1

    # For client: UI assets must exist
    if [[ "$binary" == *"client"* ]]; then
        [ -d "$bundle_dir/assets/ui" ] || exit 1
    fi
}
```

### 3.2 Post-Archive Validation (Smoke Test)

CI MUST validate after archive creation:

```bash
# Smoke test contract
smoke_test_bundle() {
    local archive=$1

    # Extract to temp directory
    temp_dir=$(mktemp -d)
    extract "$archive" "$temp_dir"

    # Check binary exists
    binary=$(find "$temp_dir" -name "plix-*" -type f -executable | head -1)
    [ -n "$binary" ] || exit 1

    # Check --help works
    timeout 5 "$binary" --help || exit 1

    # Check --version works
    "$binary" --version | grep -E "^[0-9]+\.[0-9]+\.[0-9]+" || exit 1

    # Cleanup
    rm -rf "$temp_dir"
}
```

---

## 4. Script Interface Contracts

### 4.1 Packaging Script Interface

All packaging scripts MUST accept these arguments:

```bash
./scripts/package/client_linux.sh \
    --binary-path <path>           # Path to compiled binary (required)
    --version <semver>             # Version string (required)
    --output-dir <path>            # Output directory (required)
    --assets-dir <path>            # Assets directory (default: ./assets)
    --cef-dir <path>               # CEF runtime directory (optional)
```

**Exit codes:**
- `0`: Success
- `1`: Missing required argument
- `2`: Binary not found
- `3`: Assets not found
- `4`: Archive creation failed

### 4.2 Packaging Script Output

Scripts MUST output:
- Created archive path to stdout (last line)
- Progress messages to stderr
- Generated checksum file alongside archive

```bash
# Example output
stderr: "Validating binary..."
stderr: "Copying assets..."
stderr: "Creating archive..."
stdout: "/output/plix-client-linux-x86_64-0.1.0.tar.gz"

# Checksum file created
/output/plix-client-linux-x86_64-0.1.0.tar.gz.sha256
```

---

## 5. build_info.json Contract

### 5.1 Required Fields

All `build_info.json` files MUST contain:

```json
{
  "version": "<semver>",           // Required: Cargo.toml version
  "commit_sha": "<40-char-hex>",   // Required: Full git SHA
  "commit_sha_short": "<7-char>",  // Required: Short SHA
  "build_timestamp": "<ISO8601>",  // Required: UTC timestamp
  "target_triple": "<target>",     // Required: Rust target
  "rust_version": "<version>"      // Required: Rustc version
}
```

### 5.2 Validation

```bash
validate_build_info() {
    local file=$1

    # Check required fields
    jq -e '.version' "$file" > /dev/null || exit 1
    jq -e '.commit_sha' "$file" > /dev/null || exit 1
    jq -e '.build_timestamp' "$file" > /dev/null || exit 1
    jq -e '.target_triple' "$file" > /dev/null || exit 1

    # Validate version format
    jq -r '.version' "$file" | grep -E '^[0-9]+\.[0-9]+\.[0-9]+' || exit 1

    # Validate SHA format
    jq -r '.commit_sha' "$file" | grep -E '^[a-f0-9]{40}$' || exit 1
}
```

---

## 6. CI Workflow Contracts

### 6.1 Matrix Configuration

Release workflow MUST use this matrix:

```yaml
strategy:
  fail-fast: true
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        archive_ext: tar.gz
        platform_name: linux-x86_64
      - os: windows-latest
        target: x86_64-pc-windows-msvc
        archive_ext: zip
        platform_name: win64
      - os: macos-latest
        target: x86_64-apple-darwin
        archive_ext: zip
        platform_name: macos
```

### 6.2 Required CI Steps

Each matrix job MUST include:

1. **Build**: `cargo build --release --target ${{ matrix.target }}`
2. **Generate build_info.json**: Via shadow-rs or build script
3. **Package**: Run platform-specific packaging script
4. **Validate**: Run smoke test on package
5. **Checksum**: Generate SHA256 checksum file
6. **Upload**: Upload artifact with target-based name

### 6.3 Release Job Contract

Release job MUST:
- Run only on tag push (`on: push: tags: ['v*']`)
- Wait for all build jobs (`needs: [build]`)
- Download all artifacts
- Create GitHub release
- Upload all archives to release assets
- Upload checksum files

---

## 7. Docker Contract

### 7.1 Dockerfile Requirements

```dockerfile
# Must use slim base image
FROM debian:bookworm-slim

# Must expose game port
EXPOSE 7777/udp

# Must create non-root user
RUN useradd -m plix

# Must use volume for persistence
VOLUME ["/data/world", "/data/mods"]

# Must use ENTRYPOINT for signal handling
ENTRYPOINT ["/app/plix-server-headless"]
```

### 7.2 Docker Compose Contract

```yaml
version: "3.8"
services:
  plix-server:
    image: plix-server:latest
    ports:
      - "7777:7777/udp"
    volumes:
      - ./data/world:/data/world
      - ./data/mods:/data/mods
      - ./config:/config:ro
    environment:
      - PLIX_CONFIG=/config/server.toml
    restart: unless-stopped
```
