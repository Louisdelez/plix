# CLI Interface Contract

**Feature**: 029-patch-launcher
**Version**: 1.0
**Date**: 2025-12-18

## Overview

The launcher provides a command-line interface for both interactive use and automation.

## Binary Name

```
plix-launcher
```

## Usage

```
plix-launcher [OPTIONS] [-- GAME_ARGS...]
```

## Options

### Mode Flags (Mutually Exclusive)

| Flag | Short | Description |
|------|-------|-------------|
| `--check` | `-c` | Check for updates without downloading or launching |
| `--update` | `-u` | Download and install updates without launching |
| `--launch` | `-l` | Launch game without checking for updates |
| (default) | | Check, update if needed, then launch |

### Behavior Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--dry-run` | `-n` | Simulate operations without making changes |
| `--verbose` | `-v` | Enable verbose logging |
| `--quiet` | `-q` | Suppress non-error output |
| `--stay-open` | | Keep launcher open after game exits |

### Configuration Flags

| Flag | Argument | Description |
|------|----------|-------------|
| `--manifest-url` | URL | Override manifest URL |
| `--config` | PATH | Use custom config file |
| `--data-dir` | PATH | Override data directory |

### Information Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--version` | `-V` | Print launcher version and exit |
| `--help` | `-h` | Print help message and exit |

## Game Arguments

Arguments after `--` are passed directly to the game client:

```bash
plix-launcher -- --server 192.168.1.100:7777 --name "Player1"
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Network error (could not fetch manifest) |
| 3 | Checksum verification failed |
| 4 | Installation failed |
| 5 | Launch failed |
| 10 | Already running (single instance violation) |

## Output Format

### Standard Mode (stdout)

```
[INFO] Checking for updates...
[INFO] Current version: 1.2.0
[INFO] Latest version: 1.3.0
[INFO] Update available!
[INFO] Downloading 2 files (48.5 MB total)...
[INFO]   plix-client (47.0 MB)... done
[INFO]   assets/arenas/new_arena.toml (1.5 MB)... done
[INFO] Verifying files...
[INFO] Installing version 1.3.0...
[INFO] Launching game...
```

### Check Mode Output

```
Current: 1.2.0
Latest:  1.3.0
Status:  Update available
Files:   2 files (48.5 MB)
```

### Verbose Mode (stderr)

```
[DEBUG] Config loaded from /home/user/.config/plix/launcher.toml
[DEBUG] Fetching manifest from https://releases.plix.example/manifest.toml
[DEBUG] HTTP GET completed in 234ms
[DEBUG] Manifest parsed: version=1.3.0, files=5
[DEBUG] Local state loaded: version=1.2.0
[DEBUG] Version comparison: 1.2.0 < 1.3.0
[DEBUG] Files needing update: 2
[DEBUG] Starting download: plix-client
[DEBUG] Download complete: 47448064 bytes in 12.3s
[DEBUG] Checksum verified: a1b2c3d4...
...
```

### Quiet Mode

Only errors are printed:

```
[ERROR] Failed to download plix-client: Connection timed out
```

### Dry-Run Mode

```
[DRY-RUN] Would download: plix-client (47.0 MB)
[DRY-RUN] Would download: assets/arenas/new_arena.toml (1.5 MB)
[DRY-RUN] Would install to: /home/user/.local/share/plix/versions/1.3.0
[DRY-RUN] Would update current symlink
[DRY-RUN] Would launch: /home/user/.local/share/plix/current/plix-client
```

## Examples

### Normal Launch (Auto-Update)

```bash
plix-launcher
```

### Check Only (Automation)

```bash
plix-launcher --check
# Exit code 0 = up to date, output shows status
```

### Force Update

```bash
plix-launcher --update
# Downloads and installs without launching
```

### Skip Update Check

```bash
plix-launcher --launch
# Launches existing version immediately
```

### With Game Arguments

```bash
plix-launcher -- --server play.plix.example:7777
# Launches with server argument
```

### Custom Manifest URL

```bash
plix-launcher --manifest-url https://beta.plix.example/manifest.toml
# Uses beta channel
```

### Debugging

```bash
plix-launcher --verbose --dry-run
# Shows what would happen without making changes
```

## Environment Variables

| Variable | Description | Overrides |
|----------|-------------|-----------|
| `PLIX_MANIFEST_URL` | Default manifest URL | `--manifest-url` |
| `PLIX_DATA_DIR` | Data directory | `--data-dir` |
| `PLIX_LAUNCHER_VERBOSE` | Enable verbose (`1`) | `--verbose` |
| `RUST_LOG` | Log level (tracing) | (additive) |

Priority: CLI flag > Environment variable > Config file > Default

## Signal Handling

| Signal | Behavior |
|--------|----------|
| SIGINT (Ctrl+C) | Clean shutdown, remove partial downloads |
| SIGTERM | Clean shutdown, remove partial downloads |

On Windows, Ctrl+C and console close events trigger clean shutdown.
