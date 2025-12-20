# Plix Compatibility Matrix

This document defines version compatibility rules for Plix components.

## Version Components

Plix uses multiple version numbers for different compatibility domains:

| Version Type | Format | Location | Purpose |
|--------------|--------|----------|---------|
| Game Version | MAJOR.MINOR.PATCH | `Cargo.toml` | Overall release version |
| Protocol Version | MAJOR.MINOR | Protocol handshake | Client-server network compatibility |
| Mod API Version | MAJOR.MINOR | Mod manifest | Mod engine compatibility |
| Content Schema | MAJOR.MINOR | Content registry | Content file format compatibility |
| Config Version | INTEGER | Config files | Configuration migration |

## Compatibility Rules

### Client ↔ Server (Protocol)

| Client | Server | Compatible | Notes |
|--------|--------|------------|-------|
| 1.0 | 1.0 | Yes | Same version |
| 1.0 | 1.1 | Yes | Server minor upgrade OK |
| 1.1 | 1.0 | No | Client cannot be newer |
| 1.x | 2.x | No | Major version mismatch |
| 2.x | 1.x | No | Major version mismatch |

**Rule**: Protocol major versions must match. Client minor version must be ≤ server minor version.

```
Client.major == Server.major
Client.minor <= Server.minor
```

### Mod ↔ Engine (Mod API)

| Mod | Engine | Compatible | Notes |
|-----|--------|------------|-------|
| 1.0 | 1.0 | Yes | Same version |
| 1.0 | 1.5 | Yes | Engine minor upgrade OK |
| 1.5 | 1.0 | No | Mod requires newer engine |
| 1.x | 2.x | No | Major version mismatch |

**Rule**: API major versions must match. Mod minor version must be ≤ engine minor version.

```
Mod.major == Engine.major
Mod.minor <= Engine.minor
```

### Content ↔ Server (Content Schema)

| Content | Server | Compatible | Notes |
|---------|--------|------------|-------|
| 1.0 | 1.0 | Yes | Same version |
| 1.0 | 1.5 | Yes | Server minor upgrade OK |
| 1.5 | 1.0 | No | Content requires newer server |
| 1.x | 2.x | No | Major version mismatch |

**Rule**: Schema major versions must match. Content minor version must be ≤ server minor version.

### Save Data Migration

| Save Version | Server Version | Action |
|--------------|----------------|--------|
| 0 (v0.x era) | 1.0.0 | Auto-migrate with backup |
| 1 | 1.0.0 | Compatible, no migration |
| 1 | 1.1.0 | Compatible, minor migration may apply |
| 1 | 2.0.0 | Major migration required |

**Rule**: Migration runs automatically on server startup (with `--migrate` flag).

## Version Display

### Finding Your Version

```bash
# Client version
plix-client --version

# Server version
plix-server --version

# Tools version
plix-tools --version
```

### Version Format

```
plix-client 1.0.0 (abc1234)
```

- `1.0.0` - Semantic version
- `abc1234` - Git commit short hash

## Cross-Platform Compatibility

### Binary Compatibility

| Platform | Client | Server |
|----------|--------|--------|
| Linux x86_64 | Yes | Yes |
| Windows x64 | Yes | Yes |
| macOS x64 | Yes | Yes |
| macOS ARM64 | Planned | Planned |

### Save Data

Save data is portable across platforms (binary format with defined endianness).

### Configuration

Configuration files (TOML) are portable across platforms.

## Upgrade Paths

### Client Upgrades

1. Download new client version
2. Replace existing installation
3. Launch - settings are preserved

### Server Upgrades

1. Stop the server
2. Back up configuration and world data
3. Download new server version
4. Run with `--migrate --dry-run` to preview changes
5. Run with `--migrate` to apply migrations
6. Start the server

### Mod Upgrades

1. Check mod compatibility with target engine version
2. Stop the server
3. Update mod files
4. Restart the server

## Breaking Changes Policy

### Major Version (1.x → 2.x)

May include:
- Protocol changes requiring client update
- Mod API changes requiring mod updates
- Save format changes requiring migration
- Removed deprecated features

### Minor Version (1.0 → 1.1)

Will NOT include:
- Breaking protocol changes
- Breaking mod API changes
- Incompatible save format changes

May include:
- New protocol features (backward-compatible)
- New mod API functions
- New configuration options with defaults

### Patch Version (1.0.0 → 1.0.1)

Only includes:
- Bug fixes
- Security patches
- Documentation updates

## Testing Compatibility

### For Mod Developers

```rust
// Check engine version in mod
let engine_version = plix_mod::api::engine_version();
if engine_version.minor < 2 {
    // Handle older engine
}
```

### For Server Operators

```bash
# Validate content compatibility
plix-server --validate-content

# Check migration requirements
plix-server --migrate --dry-run
```

## Troubleshooting

### "Protocol version mismatch"

Client and server have different major versions or client is newer.

**Solution**: Update client or server to matching versions.

### "Mod requires newer engine"

Mod was built for a newer engine version.

**Solution**: Update the server or find an older mod version.

### "Migration required"

Server detected old data format.

**Solution**: Run `plix-server --migrate` to upgrade data.

## Related Documents

- [Migration Guide](migration-guide.md)
- [Mod Stability Policy](../modding/stability.md)
- [Upgrade Guide](../server/upgrading.md)
