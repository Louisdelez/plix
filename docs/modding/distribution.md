# Plix Mod Distribution Guide

How to package, distribute, and install Plix mods.

## Bundle Format (.plixmod)

A `.plixmod` file is a ZIP archive containing:

```
my-mod-1.0.0.plixmod
├── mod.toml          # Manifest (required)
├── mod.wasm          # Compiled WebAssembly (required)
└── assets/           # Optional assets directory
    ├── textures/
    ├── sounds/
    └── data/
```

### Manifest (mod.toml)

```toml
# Required fields
id = "my-mod"           # Unique identifier (kebab-case, 3-64 chars)
name = "My Awesome Mod" # Display name
version = "1.0.0"       # SemVer version
api_version = 1         # SDK API version

# Optional fields
author = "Your Name"
description = "What this mod does"
homepage = "https://github.com/you/my-mod"
license = "MIT"

# Capabilities (permissions required by the mod)
[capabilities]
world_read = true       # Read blocks
world_write = true      # Modify blocks
entity_read = true      # Query entities
entity_write = true     # Damage/push entities
net_send = true         # Send messages
event_cancel_chat = true    # Cancel chat events
event_cancel_blocks = true  # Cancel block events
```

### Mod ID Rules

- 3-64 characters
- Lowercase letters, digits, and hyphens only
- Cannot start or end with hyphen
- No consecutive hyphens

Valid: `my-mod`, `chat-filter-v2`, `mod123`
Invalid: `MyMod`, `-mod`, `my--mod`, `ab`

## Creating a Bundle

### Using plix-mod CLI (Recommended)

```bash
# Build your mod
plix-mod build --release

# Create bundle
plix-mod pack

# Output: my-mod-1.0.0.plixmod
```

### Manual Creation

```bash
# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Create bundle structure
mkdir bundle
cp mod.toml bundle/
cp target/wasm32-unknown-unknown/release/my_mod.wasm bundle/mod.wasm

# Create ZIP (entries must be sorted, use epoch timestamp)
cd bundle
zip -r ../my-mod-1.0.0.plixmod mod.toml mod.wasm
```

## Deterministic Builds

Plix bundles are deterministic: same inputs always produce the same SHA-256 hash.

Requirements for determinism:
- Entries sorted alphabetically
- File timestamps set to 1980-01-01 00:00:00 (ZIP epoch)
- Deflate compression level 6
- No extra file attributes

This enables:
- Reproducible builds
- Content-addressable caching
- Integrity verification

## Bundle Validation

Always validate before distribution:

```bash
plix-mod validate my-mod-1.0.0.plixmod
```

Validation checks:
- **E001**: Valid mod ID format
- **E003**: mod.wasm present with required exports
- **E004**: mod.toml valid and complete
- **E005**: Bundle size ≤ 10 MB
- **E006**: All capabilities are known
- **E007**: API version compatible

### JSON Output

```bash
plix-mod validate my-mod-1.0.0.plixmod --json
```

```json
{
  "valid": true,
  "errors": [],
  "warnings": [],
  "info": {
    "id": "my-mod",
    "version": "1.0.0",
    "size": 45678,
    "sha256": "abc123..."
  }
}
```

## Size Limits

Maximum bundle size: **10 MB**

Tips to reduce size:
- Use `--release` builds with LTO
- Optimize WASM with `wasm-opt`
- Compress assets
- Remove unused dependencies

```toml
# Cargo.toml optimizations
[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Strip symbols
```

## Installation

### Local Installation

```bash
# Install to local cache
plix-mod install my-mod-1.0.0.plixmod

# With force overwrite
plix-mod install my-mod-1.0.0.plixmod --force
```

Default cache locations:
- Linux: `~/.local/share/plix/mods/`
- macOS: `~/Library/Application Support/plix/mods/`
- Windows: `%APPDATA%\plix\mods\`

### Server Installation

Copy the bundle to your server's mods directory and add to `mods.toml`:

```toml
# server/mods.toml
[[mods]]
id = "my-mod"
version = "1.0.0"
# sha256 = "abc123..."  # Optional integrity check
```

## Required WASM Exports

Your mod WASM must export these functions:

| Export | Signature | Purpose |
|--------|-----------|---------|
| `mod_init` | `() -> i32` | Called when mod loads |
| `mod_on_event` | `(i32, i32, i32) -> i32` | Event handler |
| `mod_shutdown` | `() -> i32` | Called when mod unloads |

The `#[plix_mod]` macro generates these automatically.

## Capabilities

Mods must declare required capabilities in `mod.toml`. The server grants or denies based on configuration.

| Capability | Description | Risk Level |
|------------|-------------|------------|
| `world_read` | Read blocks | Low |
| `world_write` | Modify blocks | Medium |
| `entity_read` | Query entities | Low |
| `entity_write` | Damage/push entities | High |
| `net_send` | Send messages | Medium |
| `event_cancel_chat` | Cancel chat events | Medium |
| `event_cancel_blocks` | Cancel block events | Medium |

## Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes to mod behavior
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

API version tracks SDK compatibility:
- `api_version = 1`: Current stable API
- Higher versions may add new features

## Security Considerations

- Mods run in WebAssembly sandbox
- Capabilities limit what mods can do
- Server operators control which mods are loaded
- Always validate bundles before use
- Check SHA-256 hashes for integrity

## Troubleshooting

### Bundle validation fails

```
E003: Missing required exports
```
Solution: Ensure you're using `#[plix_mod]` on both struct and impl.

```
E005: Bundle too large
```
Solution: Optimize WASM size, reduce assets, or split into multiple mods.

### Mod doesn't load

1. Check server logs for errors
2. Verify capabilities are granted
3. Ensure API version is compatible
4. Validate bundle integrity with SHA-256
