# Quickstart: Cross-Platform Packaging & Headless Server

**Feature**: 041-cross-platform | **Date**: 2025-12-19

## Prerequisites

- Rust 1.83+ (stable)
- Git
- Platform-specific tools:
  - **Linux**: tar, gzip
  - **Windows**: PowerShell 5.1+
  - **macOS**: zip, Xcode Command Line Tools (optional for signing)

## Development Setup

```bash
# Clone and enter project
cd plix

# Install dependencies
cargo build

# Run tests
cargo test --all
```

## Building for Release

### Client

```bash
# Linux
cargo build --release --bin plix-client

# Windows
cargo build --release --bin plix-client --target x86_64-pc-windows-msvc

# macOS
cargo build --release --bin plix-client --target x86_64-apple-darwin
```

### Headless Server

```bash
# Linux
cargo build --release --bin plix-server-headless

# Windows
cargo build --release --bin plix-server-headless --target x86_64-pc-windows-msvc

# macOS
cargo build --release --bin plix-server-headless --target x86_64-apple-darwin
```

## Packaging

### Client Bundle (Linux)

```bash
./scripts/package/client_linux.sh \
    --binary-path target/release/plix-client \
    --version 0.1.0 \
    --output-dir dist/ \
    --assets-dir assets/
```

Output: `dist/plix-client-linux-x86_64-0.1.0.tar.gz`

### Server Bundle (Linux)

```bash
./scripts/package/server_linux.sh \
    --binary-path target/release/plix-server-headless \
    --version 0.1.0 \
    --output-dir dist/
```

Output: `dist/plix-server-headless-linux-x86_64-0.1.0.tar.gz`

## Running Headless Server

### From Bundle

```bash
# Extract
tar xzf plix-server-headless-linux-x86_64-0.1.0.tar.gz
cd plix-server-headless-linux-x86_64-0.1.0

# Edit config
cp configs/examples/server.toml ./server.toml
vim server.toml

# Run
./run_server.sh
# or directly:
./plix-server-headless --config server.toml
```

### With Docker

```bash
# Build image
docker build -t plix-server -f deploy/docker/Dockerfile .

# Run
docker run -d \
    -p 7777:7777/udp \
    -v $(pwd)/data:/data \
    -v $(pwd)/config:/config:ro \
    plix-server
```

## CI/CD Release

Releases are automated via GitHub Actions:

1. **Push a tag**: `git tag v0.1.0 && git push --tags`
2. **CI builds** all platforms in parallel
3. **Artifacts uploaded** to GitHub Release

Manual trigger:
```bash
gh workflow run release.yml --ref v0.1.0
```

## Smoke Testing

### Validate Package

```bash
# Check binary runs
./plix-server-headless --version

# Check help works
./plix-server-headless --help

# Test config validation
./plix-server-headless --config invalid.toml
# Should exit with code 1
```

### CI Smoke Test

```bash
# Run packaging validation
./scripts/validate_bundle.sh dist/plix-client-linux-x86_64-0.1.0.tar.gz
```

## Exit Codes Reference

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 64 | Port bind failed |
| 65 | Asset load failed |
| 66 | Persistence error |

## Configuration Files

### server.toml

```toml
[network]
bind_address = "0.0.0.0"
port = 7777
max_players = 32

[game]
tick_rate = 30
arena = "ffa_small"

[server]
name = "My Plix Server"
motd = "Welcome!"

[logging]
level = "info"

[persistence]
autosave_interval_secs = 300
shutdown_timeout_secs = 5
```

## Directory Structure

```
~/.config/plix/           # User config (Linux)
├── server.toml
└── server_mods.toml

~/.local/share/plix/      # User data (Linux)
├── worlds/
└── mods_cache/

# Or with Docker volumes:
/data/world/              # World persistence
/data/mods/               # Mod cache
/config/                  # Read-only config mount
```

## Troubleshooting

### Port Already in Use (Exit 64)

```bash
# Check what's using the port
lsof -i :7777
# or
netstat -tulpn | grep 7777

# Kill existing process or use different port
./plix-server-headless --port 7778
```

### Missing Assets (Exit 65)

```bash
# Verify assets directory
ls assets/arenas/
ls assets/ui/

# Specify custom path
./plix-server-headless --assets-dir /path/to/assets
```

### Permission Denied

```bash
# Make binary executable
chmod +x plix-server-headless

# Check file permissions
ls -la plix-server-headless
```
