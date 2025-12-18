# Quick Start: Plix Launcher

## For Players

### Installation

1. Download `plix-launcher` from the releases page
2. Place it anywhere on your system
3. Run it!

```bash
./plix-launcher
```

The launcher will:
- Check for the latest version
- Download the game if needed
- Launch automatically

### First Run

On first run, the launcher creates:
- `~/.config/plix/launcher.toml` - Configuration
- `~/.local/share/plix/` - Game files

### Playing

Just run the launcher each time you want to play:

```bash
./plix-launcher
```

It handles updates automatically.

### Offline Play

If you lose internet connection:
- The launcher will use your existing game version
- You'll see "Offline mode - using local version"

### Troubleshooting

**"Connection timed out"**
- Check your internet connection
- Try again later

**"Checksum verification failed"**
- The download was corrupted
- Delete `~/.local/share/plix/` and try again

**"Cannot proceed"**
- You're offline and have no game installed
- Connect to the internet for initial download

---

## For Developers

### Publishing a Release

1. Build your release binaries
2. Generate checksums:
   ```bash
   sha256sum plix-client > checksums.txt
   sha256sum assets/**/* >> checksums.txt
   ```
3. Create `manifest.toml`:
   ```toml
   manifest_version = 1
   version = "1.3.0"
   release_date = 1734480000

   [[files]]
   path = "plix-client"
   url = "https://your-cdn.example/v1.3.0/plix-client"
   size = 47448064
   sha256 = "a1b2c3d4..."
   executable = true
   ```
4. Upload files and manifest to your HTTP server
5. Players automatically receive updates

### Manifest Hosting Options

**GitHub Releases** (Recommended)
```
https://github.com/yourorg/plix/releases/download/v1.3.0/manifest.toml
```

**Static File Server**
```
https://releases.plix.example/manifest.toml
```

**CDN**
```
https://cdn.plix.example/releases/manifest.toml
```

### Testing Updates

1. Build launcher:
   ```bash
   cargo build --release -p plix-launcher
   ```

2. Test with local manifest:
   ```bash
   ./plix-launcher --manifest-url file:///path/to/manifest.toml --dry-run
   ```

3. Verify behavior:
   ```bash
   ./plix-launcher --check --verbose
   ```

---

## For Server Admins

### Version Compatibility

The launcher ensures players have compatible versions:

1. Launcher downloads matching client version
2. Client reports version to server on connect
3. Server can reject incompatible versions

### Recommended Setup

1. Host manifest alongside your server releases
2. Point launcher at your manifest URL
3. Players automatically stay compatible

### Custom Manifest URL

Players can use your server's manifest:

```bash
./plix-launcher --manifest-url https://your-server.example/manifest.toml
```

Or set in config:

```toml
# ~/.config/plix/launcher.toml
manifest_url = "https://your-server.example/manifest.toml"
```

---

## CLI Reference

```
plix-launcher [OPTIONS] [-- GAME_ARGS...]

Options:
  -c, --check        Check for updates only
  -u, --update       Update without launching
  -l, --launch       Launch without update check
  -n, --dry-run      Simulate without changes
  -v, --verbose      Verbose output
  -q, --quiet        Errors only
  -V, --version      Print version
  -h, --help         Print help

Examples:
  plix-launcher                     # Normal use
  plix-launcher --check             # Check for updates
  plix-launcher -- --server ip:port # Pass args to game
```

---

## Directory Structure

```
~/.config/plix/
└── launcher.toml        # Launcher config

~/.local/share/plix/
├── versions/            # Downloaded versions
│   └── 1.3.0/
├── current/             # Active version
├── launcher/
│   └── state.toml       # Install state
└── logs/
    └── launcher.log     # Launcher log
```

---

## Configuration

### launcher.toml

```toml
# Manifest URL (where to check for updates)
manifest_url = "https://releases.plix.example/manifest.toml"

# Keep launcher open after game exits
stay_open = false

# HTTP timeout in seconds
timeout_seconds = 30

# Download retry attempts
max_retries = 3

# Enable verbose logging
verbose = false
```

All settings can be overridden via CLI flags or environment variables.
