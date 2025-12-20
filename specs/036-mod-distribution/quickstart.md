# Quickstart: Mod Distribution

**Feature Branch**: `036-mod-distribution`
**Date**: 2025-12-18

## Overview

This guide shows how to configure and use the mod distribution system for plix servers.

## Prerequisites

- plix server with features 034 (Mod API Core) and 035 (Sandboxed Mod Runtime)
- Network access to mod registries (or local registry)

## 1. Basic Configuration

Create `server_mods.toml` in your server directory:

```toml
# Minimal configuration - official registry + one mod
[[registries]]
name = "official"
url = "https://mods.plix.dev/index.json"

[[mods]]
id = "weapons-pack"
version = "^2.0"
```

Start your server - mods will be automatically downloaded and installed.

## 2. Version Constraints

Use SemVer constraints to control which versions are installed:

```toml
[[mods]]
id = "core-lib"
version = "^1.0"          # >=1.0.0, <2.0.0 (recommended)

[[mods]]
id = "weapons-pack"
version = "~2.1"          # >=2.1.0, <2.2.0 (patch updates only)

[[mods]]
id = "legacy-mod"
version = "=1.5.3"        # Exact version only
pinned = true             # Ignore newer compatible versions

[[mods]]
id = "experimental"
version = ">=0.1, <1.0"   # Range constraint
```

## 3. Multiple Registries

Configure registries in priority order (lower = higher priority):

```toml
# Local registry has highest priority
[[registries]]
name = "local"
url = "/srv/plix/mods"
priority = 1

# Official registry as fallback
[[registries]]
name = "official"
url = "https://mods.plix.dev/index.json"
priority = 100
```

## 4. Lockfile for Reproducibility

After first run, a `mods.lock` file is generated:

```json
{
  "version": 1,
  "generated_at": "2025-12-18T12:00:00Z",
  "engine_version": "0.36.0",
  "mods": [
    {
      "id": "core-lib",
      "version": "1.2.0",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "source": "official",
      "download_url": "https://mods.plix.dev/bundles/core-lib-1.2.0.plixmod",
      "dependencies": []
    }
  ]
}
```

**To update mods**: Delete `mods.lock` and restart, or use the update command.

**To reproduce exact versions**: Copy `mods.lock` to another server.

## 5. Optional Signature Verification

For production servers requiring verified mods:

```toml
[trust]
require_signature = true
allowed_keys = [
    "abc123def456abc1",    # Trusted publisher 1
    "fedcba9876543210"     # Trusted publisher 2
]
```

## 6. Local Registry Setup

Create a local registry for offline operation or private mods:

```
local-registry/
├── index.json
└── bundles/
    └── my-mod-1.0.0.plixmod
```

**index.json**:
```json
{
  "registry_version": 1,
  "name": "Local Mods",
  "mods": [
    {
      "id": "my-mod",
      "name": "My Local Mod",
      "versions": [
        {
          "version": "1.0.0",
          "sha256": "<sha256-hash-here>",
          "download_url": "bundles/my-mod-1.0.0.plixmod",
          "size": 12345,
          "dependencies": [],
          "api_version": 1
        }
      ]
    }
  ]
}
```

## 7. Creating a Mod Bundle

Package your mod as a `.plixmod` file:

```bash
# Required structure:
my-mod/
├── mod.toml           # Manifest (required)
├── mod.wasm           # WASM module (optional)
└── assets/            # Assets directory (optional)
    └── textures/

# Create bundle (deterministic zip):
cd my-mod
zip -r -X ../my-mod-1.0.0.plixmod mod.toml mod.wasm assets/

# Calculate SHA-256:
sha256sum ../my-mod-1.0.0.plixmod
```

## 8. Error Codes

| Code | Meaning | Resolution |
|------|---------|------------|
| EMREG001 | Registry unreachable | Check network, verify URL |
| EMREG002 | Invalid index format | Contact registry maintainer |
| EMREG003 | Download failed | Check network, retry later |
| EMREG004 | Hash mismatch | Re-download, check registry integrity |
| EMREG005 | Invalid signature | Verify signing key is in allowed_keys |
| EMREG006 | Dependency conflict | Adjust version constraints |
| EMREG007 | Version incompatible | Update engine or use compatible mod version |
| EMREG008 | Dependency cycle | Contact mod authors |

## 9. Cache Management

Mods are cached at `~/.local/share/plix/mods/`:

```bash
# View cache size
du -sh ~/.local/share/plix/mods/

# Clear cache (forces re-download)
rm -rf ~/.local/share/plix/mods/bundles/*
rm -rf ~/.local/share/plix/mods/installed/*
```

## 10. Common Workflows

### Add a New Mod

1. Add to `server_mods.toml`:
   ```toml
   [[mods]]
   id = "new-mod"
   version = "^1.0"
   ```
2. Restart server

### Update All Mods

1. Delete `mods.lock`
2. Restart server (generates new lockfile with latest compatible versions)

### Pin Current Versions

1. Copy versions from `mods.lock` to `server_mods.toml`
2. Set `pinned = true` for each mod

### Deploy to Production

1. Test on staging server
2. Copy `mods.lock` to production server
3. Start production server (uses exact pinned versions)

## API Usage (for integrators)

```rust
use plix_mod_distribution::{DistributionConfig, resolve_and_install};

// Load config
let config = DistributionConfig::load("server_mods.toml")?;

// Resolve dependencies and install
let result = resolve_and_install(&config).await?;

println!("Installed {} mods", result.mods.len());
for m in &result.mods {
    println!("  {} @ {}", m.id, m.version);
}
```

## Next Steps

- See [data-model.md](./data-model.md) for type definitions
- See [contracts/](./contracts/) for JSON schemas
- See [research.md](./research.md) for design decisions
