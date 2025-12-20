# Feature 036: Mod Distribution System

This document describes the mod distribution system implemented in Feature 036.

## Overview

The mod distribution system provides a complete pipeline for downloading, verifying, and installing mods from remote registries. It supports:

- **Multiple registries** with priority ordering (local filesystem or HTTP)
- **SemVer-based dependency resolution** with conflict and cycle detection
- **Lockfile generation** for reproducible builds
- **SHA-256 integrity verification** (mandatory)
- **Ed25519 signature verification** (optional, feature-gated)
- **Caching** to avoid redundant downloads

## Quick Start

### Server Configuration

Create a `server_mods.toml` in your server root directory:

```toml
# Registry sources (priority order, lower = higher priority)
[[registries]]
name = "local"
url = "/srv/plix/mods"
priority = 1

[[registries]]
name = "official"
url = "https://mods.plix.dev/index.json"
priority = 100

# Required mods
[[mods]]
id = "core-lib"
version = "^1.0"

[[mods]]
id = "weapons-pack"
version = "=2.0.0"
pinned = true  # Don't auto-update

# Trust policy (optional)
[trust]
require_signature = false  # Set to true to require signed mods
allowed_keys = []          # Empty = allow any valid signature

# Download settings (optional)
[download]
connect_timeout_secs = 30
read_timeout_secs = 120
retries = 3
max_bundle_size = 52428800  # 50 MB

# Cache settings (optional)
[cache]
path = "/var/cache/plix/mods"  # Default: ~/.local/share/plix/mods/
```

### Server Startup

The mod distribution system integrates automatically at server startup:

```rust
use plix_server::mods::{init_mod_distribution, ModManager};

async fn start_server(server_root: &Path) {
    // 1. Download and install required mods
    let mod_paths = init_mod_distribution(server_root).await?;

    // 2. Initialize mod manager and load installed mods
    let mut mod_manager = ModManager::with_wasm_runtime(server_root);
    mod_manager.load_mods()?;

    // 3. Run game loop with mod events
    loop {
        mod_manager.emit_event(GameEvent::tick(tick_number));
        mod_manager.dispatch_events();
        // ... game logic
    }
}
```

## Architecture

### Crate Structure

```text
plix-mod-distribution/
├── src/
│   ├── lib.rs         # Core types: ModId, ModVersion, ModDependency
│   ├── errors.rs      # Error codes EMREG001-008
│   ├── config.rs      # Configuration parsing
│   ├── index.rs       # Registry index format
│   ├── registry.rs    # Local and HTTP registry sources
│   ├── resolver.rs    # Dependency resolution
│   ├── lockfile.rs    # Lockfile generation/parsing
│   ├── downloader.rs  # Bundle download with retry
│   ├── integrity.rs   # SHA-256 verification
│   ├── installer.rs   # Bundle extraction
│   ├── bundle.rs      # Bundle format and metadata
│   └── signatures.rs  # Ed25519 verification (feature-gated)
```

### Data Flow

1. **Configuration** - Parse `server_mods.toml`
2. **Registry Fetch** - Download index.json from each registry
3. **Resolution** - Resolve dependencies with SemVer constraints
4. **Lockfile** - Generate or update `mods.lock` for reproducibility
5. **Download** - Fetch bundles not in cache
6. **Verification** - Verify SHA-256 (and signatures if enabled)
7. **Installation** - Extract bundles to cache directory

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| EMREG001 | RegistryUnreachable | Cannot reach registry server |
| EMREG002 | InvalidIndex | Registry index format is invalid |
| EMREG003 | DownloadFailed | Bundle download failed |
| EMREG004 | HashMismatch | SHA-256 hash doesn't match |
| EMREG005 | SignatureInvalid | Ed25519 signature invalid or missing |
| EMREG006 | DependencyConflict | Conflicting version requirements |
| EMREG007 | VersionIncompatible | API or engine version mismatch |
| EMREG008 | CycleDetected | Circular dependency detected |

## Lockfile Format

The `mods.lock` file ensures reproducible mod installations:

```json
{
  "lockfile_version": 1,
  "generated_at": "2025-12-18T12:00:00Z",
  "mods": [
    {
      "id": "core-lib",
      "version": "1.2.0",
      "source": "official",
      "sha256": "abc123...",
      "download_url": "https://mods.plix.dev/bundles/core-lib-1.2.0.plixmod"
    }
  ]
}
```

## Registry Index Format

Registries must provide an `index.json`:

```json
{
  "registry_version": 1,
  "name": "Official Plix Mods",
  "base_url": "https://mods.plix.dev",
  "updated_at": "2025-12-18T12:00:00Z",
  "mods": [
    {
      "id": "core-lib",
      "name": "Core Library",
      "description": "Shared utilities for other mods",
      "author": "Plix Team",
      "versions": [
        {
          "version": "1.2.0",
          "sha256": "abc123...",
          "download_url": "bundles/core-lib-1.2.0.plixmod",
          "size": 1024,
          "dependencies": [],
          "api_version": 1,
          "engine": { "min": "0.36.0" },
          "published_at": "2025-12-01T10:00:00Z"
        }
      ]
    }
  ]
}
```

## Bundle Format (.plixmod)

Mod bundles are ZIP archives containing:

```text
my-mod-1.0.0.plixmod (ZIP)
├── mod.toml          # Manifest (required)
├── mod.wasm          # WASM bytecode (if WASM mod)
├── assets/           # Mod assets (optional)
└── ...
```

The `mod.toml` format:

```toml
[mod]
id = "my-mod"
name = "My Mod"
version = "1.0.0"
api_version = 1

[[dependencies]]
id = "core-lib"
version = "^1.0"
```

## Optional Signatures

Enable signature verification with the `signatures` feature:

```toml
[dependencies.plix-mod-distribution]
version = "0.1"
features = ["signatures"]
```

Configure trusted keys in `server_mods.toml`:

```toml
[trust]
require_signature = true
allowed_keys = ["abc123def456..."]  # Ed25519 public keys (hex)
```

## Cache Layout

```text
~/.local/share/plix/mods/
├── bundles/                    # Downloaded .plixmod files (by hash)
│   └── abc123...def456.plixmod
└── installed/                  # Extracted mod directories
    └── my-mod/
        └── 1.0.0/
            ├── mod.toml
            └── mod.wasm
```

## Testing

Run the distribution tests:

```bash
# Core tests
cargo test -p plix-mod-distribution

# With signature tests
cargo test -p plix-mod-distribution --features signatures

# Integration tests
cargo test -p plix-server mod_integration
```

Test fixtures are available at `tests/fixtures/mock_registry/`.
