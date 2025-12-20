# Publishing Plix Mods

This guide covers how to package, distribute, and maintain Plix mods.

## Mod Package Format

### Directory Structure

```
my-mod/
  plix-mod.toml      # Mod manifest (required)
  my-mod.wasm        # Compiled WASM module (required)
  assets/            # Optional assets
    icon.png         # Mod icon (128x128 PNG)
    config.toml      # Default configuration
  README.md          # Mod documentation
  LICENSE            # License file
  CHANGELOG.md       # Version history
```

### Bundle Format

Mods are distributed as `.plixmod` files, which are ZIP archives:

```bash
# Create a mod bundle
zip -r my-mod-1.0.0.plixmod my-mod/

# Contents:
# my-mod-1.0.0.plixmod
#   plix-mod.toml
#   my-mod.wasm
#   assets/
#   README.md
#   ...
```

## Manifest Reference

### Complete Example

```toml
[mod]
# Unique identifier (lowercase, alphanumeric, hyphens)
id = "my-awesome-mod"

# Display name
name = "My Awesome Mod"

# Semantic version
version = "1.0.0"

# Required Plix mod API version
api_version = "1.0"

# Short description (max 200 chars)
description = "Adds awesome features to your Plix server"

# Author information
authors = [
    "Alice Developer <alice@example.com>",
    "Bob Coder <bob@example.com>"
]

# Homepage URL (optional)
homepage = "https://github.com/username/my-awesome-mod"

# Repository URL (optional)
repository = "https://github.com/username/my-awesome-mod"

# License identifier (SPDX)
license = "MIT"

# Keywords for search (optional)
keywords = ["gameplay", "pvp", "stats"]

# Minimum Plix version (optional)
min_plix_version = "1.0.0"

[capabilities]
# Declare required capabilities
player_events = true
chat_events = true
timer = true
storage = true

[dependencies]
# Other mods this depends on (optional)
# Format: "mod-id" = "version-requirement"
# some-lib-mod = ">=1.0.0"
```

## Versioning

### Semantic Versioning

Follow [SemVer](https://semver.org/) for mod versions:

- **MAJOR**: Breaking changes to configuration or behavior
- **MINOR**: New features, backward-compatible
- **PATCH**: Bug fixes

### API Version Compatibility

```toml
# Specify the mod API version you built against
api_version = "1.0"
```

Your mod will work with any Plix engine where:
- Engine major version matches your `api_version` major
- Engine minor version is ≥ your `api_version` minor

## Building for Release

### Optimized Build

```bash
# Build release WASM
cargo build --target wasm32-unknown-unknown --release

# Optimize with wasm-opt (recommended)
wasm-opt -O3 \
  target/wasm32-unknown-unknown/release/my_mod.wasm \
  -o my-mod.wasm

# Strip debug info (optional, reduces size)
wasm-strip my-mod.wasm
```

### Size Optimization

For smaller bundles:

```toml
# Cargo.toml
[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Strip symbols
```

## Distribution Methods

### 1. Direct Download

Host the `.plixmod` file on your own server or file host.

**Requirements:**
- Provide SHA-256 checksum
- Maintain version history
- Document installation steps

### 2. GitHub Releases

Use GitHub Releases for version management:

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Build
        run: |
          cargo build --target wasm32-unknown-unknown --release

      - name: Package
        run: |
          mkdir -p dist/my-mod
          cp target/wasm32-unknown-unknown/release/*.wasm dist/my-mod/
          cp plix-mod.toml dist/my-mod/
          cp -r assets dist/my-mod/ 2>/dev/null || true
          cp README.md dist/my-mod/
          cd dist && zip -r my-mod-${GITHUB_REF#refs/tags/v}.plixmod my-mod/

      - name: Release
        uses: softprops/action-gh-release@v1
        with:
          files: dist/*.plixmod
```

### 3. Plix Mod Registry (Future)

A centralized mod registry is planned for future versions.

## Installation Instructions

### For Users

Document installation for your mod:

```markdown
## Installation

1. Download `my-mod-1.0.0.plixmod` from the releases page
2. Verify the checksum: `sha256sum my-mod-1.0.0.plixmod`
3. Copy to your server's `mods/` directory
4. Restart the server
5. (Optional) Configure in `mods/my-mod/config.toml`
```

### Server Configuration

```bash
# Server mods directory structure
server/
  mods/
    my-mod-1.0.0.plixmod
    other-mod-2.1.0.plixmod
```

Mods are loaded automatically on server startup.

## Signing Mods (Recommended)

For trusted distribution, sign your mod bundles:

```bash
# Generate a signing key (one time)
openssl genpkey -algorithm Ed25519 -out mod-signing-key.pem

# Export public key for verification
openssl pkey -in mod-signing-key.pem -pubout -out mod-signing-key.pub

# Sign the mod bundle
openssl pkeyutl -sign \
  -inkey mod-signing-key.pem \
  -rawin \
  -in my-mod-1.0.0.plixmod \
  -out my-mod-1.0.0.plixmod.sig

# Users can verify with your public key
openssl pkeyutl -verify \
  -pubin -inkey mod-signing-key.pub \
  -rawin \
  -in my-mod-1.0.0.plixmod \
  -sigfile my-mod-1.0.0.plixmod.sig
```

Publish your public key in your repository and documentation.

## Checksums

Always provide checksums for verification:

```bash
# Generate SHA-256 checksum
sha256sum my-mod-1.0.0.plixmod > my-mod-1.0.0.plixmod.sha256

# Users verify with:
sha256sum -c my-mod-1.0.0.plixmod.sha256
```

## Documentation

### README Template

```markdown
# My Awesome Mod

[Brief description]

## Features

- Feature 1
- Feature 2

## Requirements

- Plix 1.0.0 or higher
- Mod API 1.0

## Installation

[Installation steps]

## Configuration

```toml
# config.toml options
option_1 = "default"
option_2 = true
```

## Commands

| Command | Description |
|---------|-------------|
| `/mymod help` | Show help |

## Permissions/Capabilities

This mod requires:
- `player_events` - [Why needed]
- `timer` - [Why needed]

## License

MIT License - See [LICENSE](LICENSE)

## Support

[How to get help, report issues]
```

## Maintenance

### Update Process

1. Update version in `plix-mod.toml`
2. Update `CHANGELOG.md`
3. Build and test
4. Create release with checksums
5. Announce to users

### Deprecation

When discontinuing a mod:

1. Announce deprecation with timeline
2. Update README with notice
3. Consider transferring to new maintainer
4. Archive repository (don't delete)

## Best Practices

1. **Test thoroughly** before release
2. **Document everything** - configuration, commands, behavior
3. **Use semantic versioning** consistently
4. **Provide checksums** for all downloads
5. **Sign releases** for trusted mods
6. **Keep dependencies minimal**
7. **Respond to issues** promptly
8. **Maintain a changelog** for every version

## Related Documents

- [SDK Reference](sdk-v1.md)
- [Stability Policy](stability.md)
- [Compatibility Guide](compatibility.md)
