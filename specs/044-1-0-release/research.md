# Research: 1.0 Release

**Feature**: 044-1-0-release | **Date**: 2025-12-20

## R1: Semantic Versioning Best Practices

**Decision**: Follow Semantic Versioning 2.0.0 (semver.org) strictly

**Rationale**:
- Industry standard for version numbering
- Clear compatibility expectations (MAJOR.MINOR.PATCH)
- Tooling support (Cargo, npm, etc.) built around semver
- Community familiarity reduces confusion

**Alternatives Considered**:
- CalVer (calendar versioning): Rejected - doesn't communicate compatibility
- Simple incrementing numbers: Rejected - no semantic meaning
- Date-based versions: Rejected - no stability guarantees

**Implementation**:
- MAJOR: Breaking changes to protocol, mod API, or content schema
- MINOR: Backward-compatible new features
- PATCH: Backward-compatible bug fixes

## R2: Version Display Locations

**Decision**: Display version in 5 canonical locations (FR-022)

**Rationale**:
- Consistent debugging experience
- Easy identification of version mismatches
- Standard practice for production software

**Locations**:
1. Client: Startup log + About panel (UI)
2. Server: Startup log + console banner
3. CLI tools: `--version` flag output
4. Mod API: Constant accessible to mods
5. Protocol: Handshake version field (already exists)

**Existing Infrastructure**:
- `plix-common::BuildInfo` via shadow-rs - already embeds version
- Protocol `protocol_version: u8` in Connect message - already exists
- Need to add: UI About panel, version constants for mod API

## R3: Migration Framework Design

**Decision**: Sequential version-to-version migrations with automatic backup

**Rationale**:
- Predictable upgrade path (N → N+1 → ... → current)
- Each migration is small and testable
- Rollback possible via backup restoration
- Industry standard (Rails, Django, Alembic)

**Alternatives Considered**:
- Direct N → current migrations: Rejected - combinatorial explosion of paths
- Manual migrations: Rejected - error-prone, bad UX
- No migrations (breaking changes only): Rejected - hostile to users

**Implementation Pattern**:
```rust
trait Migration {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, data: &mut Data) -> Result<(), MigrationError>;
}

fn run_migrations(data: &mut Data, current: u32, target: u32) -> Result<()> {
    for version in current..target {
        let migration = get_migration(version, version + 1)?;
        migration.migrate(data)?;
    }
    Ok(())
}
```

## R4: Backup Strategy

**Decision**: Timestamped backups with 3-rolling retention (FR-007)

**Rationale**:
- Balances disk space with recovery options
- Allows recovery from recent bad migrations
- Simple to implement and understand
- 3 backups covers: pre-migration, previous, and one buffer

**Implementation**:
- Backup location: Adjacent to data (e.g., `config.toml.bak.2025-12-20T14-30-00`)
- Naming: `{filename}.bak.{ISO8601-timestamp}`
- Rotation: On new backup, delete oldest if count > 3
- Logging: Log backup path, size, and result

**Recovery Flow**:
1. Migration fails → data unchanged (atomic)
2. User restores from backup manually if needed
3. `--restore-backup` CLI flag for guided restoration

## R5: Content Schema Versioning

**Decision**: Add `schema_version` field to content registry, not individual files

**Rationale**:
- Single source of truth for content format version
- Avoids version sprawl across 50+ TOML files
- Registry validation can check compatibility

**Alternatives Considered**:
- Per-file schema_version: Rejected - maintenance burden, version sprawl
- No versioning: Rejected - breaks migration capability

**Implementation**:
- Add `CONTENT_SCHEMA_VERSION: (u8, u8)` constant to plix-server content module
- Content registry validates against this version at load time
- Migration scripts upgrade content format if needed

## R6: Mod API Stability Markers

**Decision**: Use Rust attributes for stability marking (FR-004)

**Rationale**:
- Compile-time visibility for mod developers
- Documentation automatically reflects status
- Standard Rust pattern (stdlib uses similar approach)

**Implementation**:
```rust
/// Stable API - guaranteed for v1.x lifetime
#[doc(alias = "stable")]
pub fn get_player_position(id: PlayerId) -> Vec3 { ... }

/// Experimental API - may change in minor versions
#[deprecated(note = "Experimental: API may change")]
pub fn get_experimental_feature() -> Result<()> { ... }
```

**Documentation**:
- Generate stability summary from doc attributes
- Clear sections in SDK docs: Stable, Experimental, Deprecated

## R7: Open-Source Governance Files

**Decision**: Standard GitHub community files at repo root

**Rationale**:
- GitHub automatically detects and displays these
- Community familiarity with standard locations
- Templates available (Contributor Covenant, etc.)

**Files**:
| File | Purpose | Template Source |
|------|---------|-----------------|
| LICENSE | MIT license text | choosealicense.com |
| README.md | Project overview | Custom |
| CONTRIBUTING.md | Contribution guide | GitHub template |
| CODE_OF_CONDUCT.md | Community standards | Contributor Covenant 2.1 |
| SECURITY.md | Vulnerability reporting | GitHub template |
| ROADMAP.md | Future plans | Custom |

## R8: Release Workflow

**Decision**: GitHub Actions with GPG-signed tags and SHA-256 checksums

**Rationale**:
- Automated, reproducible releases
- Cryptographic verification of authenticity
- Industry standard for open-source projects

**Workflow**:
1. Tag commit with `git tag -s v1.0.0`
2. CI detects signed tag, runs full test suite
3. Build artifacts for all platforms
4. Generate SHA-256 checksums (`sha256sum`)
5. Create GitHub Release with artifacts and checksums
6. Publish release notes

**Existing Infrastructure**:
- `.github/workflows/release.yml` exists from Feature 041
- Need to add: GPG signing verification, checksum generation

## R9: Documentation Structure

**Decision**: Modular Markdown under `docs/` with category subdirectories

**Rationale**:
- Clear organization by audience (user, server admin, mod developer)
- Easy to navigate and maintain
- Can be published via GitHub Pages or mdbook

**Structure**:
```
docs/
├── user/           # End-user documentation
├── server/         # Server administrator docs
├── modding/        # Mod developer SDK docs
└── release/        # Release-specific docs
```

**Tooling**:
- mdbook or similar for HTML generation (optional, not in scope)
- Markdown linting in CI (optional, not in scope)

## R10: Compatibility Matrix

**Decision**: Document compatibility rules in README and SDK docs

**Rationale**:
- Clear expectations for users and developers
- Prevents version mismatch confusion
- Standard practice for multi-component systems

**Matrix**:
| Component A | Component B | Compatibility Rule |
|-------------|-------------|-------------------|
| Client v1.x | Server v1.x | Compatible (same major) |
| Mod v1.0 | Engine v1.x | Compatible (minor upgrades OK) |
| Protocol v1.x | Protocol v1.y | Compatible (same major) |
| Content v1.x | Engine v1.x | Compatible (same major) |

## Summary

All research questions resolved. No NEEDS CLARIFICATION items remain.

**Key Findings**:
1. Existing infrastructure covers ~60% of requirements (BuildInfo, protocol version)
2. Main new work: migration framework, governance files, documentation
3. No new crates needed - extend existing modules
4. Standard patterns apply (semver, Contributor Covenant, GitHub Actions)
