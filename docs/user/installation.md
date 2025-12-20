# Plix Installation Guide

This guide covers installing Plix on your system.

## System Requirements

### Minimum
- **OS**: Windows 10, macOS 11+, or Linux (Ubuntu 22.04+)
- **CPU**: 64-bit processor, 2+ cores
- **RAM**: 4 GB
- **GPU**: Vulkan 1.2 compatible graphics card
- **Storage**: 500 MB free space
- **Network**: Broadband internet for multiplayer

### Recommended
- **CPU**: 4+ cores, 3.0 GHz
- **RAM**: 8 GB
- **GPU**: Dedicated graphics with 2+ GB VRAM

## Installation Methods

### Option 1: Download Release (Recommended)

1. Go to the [Releases page](https://github.com/your-org/plix/releases)
2. Download the appropriate archive for your platform:
   - Windows: `plix-1.0.0-windows-x64.zip`
   - macOS: `plix-1.0.0-macos-arm64.tar.gz` or `plix-1.0.0-macos-x64.tar.gz`
   - Linux: `plix-1.0.0-linux-x64.tar.gz`
3. Extract the archive to your preferred location
4. Run the launcher or client executable

### Verifying Downloads

Each release includes SHA-256 checksums in `SHA256SUMS`. Verify your download:

```bash
# Linux/macOS
sha256sum -c SHA256SUMS

# Windows (PowerShell)
Get-FileHash plix-1.0.0-windows-x64.zip -Algorithm SHA256
```

### Option 2: Build from Source

Requirements:
- Rust 1.83+ (stable toolchain)
- Git

```bash
# Clone the repository
git clone https://github.com/your-org/plix.git
cd plix

# Build release binaries
cargo build --release

# Binaries are in target/release/
```

## Directory Structure

After installation:

```
plix/
  plix-client       # Game client
  plix-server       # Dedicated server
  plix-tools        # Development utilities
  assets/           # Game assets (arenas, content)
```

## Configuration Paths

Plix stores configuration in platform-specific locations:

| Platform | Config Path |
|----------|-------------|
| Linux    | `~/.config/plix/` |
| macOS    | `~/Library/Application Support/plix/` |
| Windows  | `%APPDATA%\plix\` |

## First Launch

1. Run `plix-client`
2. The game will create default configuration files on first launch
3. Connect to a server using the server browser or direct IP

See [Getting Started](getting-started.md) for gameplay basics.

## Troubleshooting

### "Vulkan not found" error
Install or update your graphics drivers. Ensure your GPU supports Vulkan 1.2.

### Connection timeout
- Verify the server address is correct
- Check firewall settings (UDP port 7777 by default)
- Ensure the server is running and reachable

### Black screen on launch
- Update graphics drivers
- Try windowed mode: `plix-client --windowed`
- Check logs in `~/.config/plix/logs/`

## Updating

When updating to a new version:

1. Download the new release
2. Back up your config folder (optional)
3. Extract over your existing installation
4. Your settings will be migrated automatically

See [Upgrading](../server/upgrading.md) for server upgrade procedures.

## Uninstalling

1. Delete the installation directory
2. Optionally remove configuration:
   - Linux: `rm -rf ~/.config/plix ~/.local/share/plix`
   - macOS: `rm -rf ~/Library/Application\ Support/plix`
   - Windows: Remove `%APPDATA%\plix` and `%LOCALAPPDATA%\plix`
