# Research: Patch Updater & Launcher

**Feature**: 029-patch-launcher
**Date**: 2025-12-18

## Technical Decisions

### 1. HTTP Client Mode

**Decision**: Use `reqwest::blocking` (synchronous HTTP)

**Rationale**:
- Launcher operations are inherently sequential: check → download → verify → install → launch
- No benefit from async concurrency for single-file downloads
- Simpler error handling without async/await
- Existing pattern in plix-client uses blocking for server browser fetch

**Alternatives Considered**:
- `reqwest` async: Adds complexity without benefit for sequential operations
- `ureq`: Lighter alternative but reqwest already in workspace dependencies

### 2. Checksum Algorithm

**Decision**: SHA256 (via `sha2` crate)

**Rationale**:
- Industry standard for file integrity verification
- Fast enough for game files (hundreds of MB)
- Already used implicitly in Rust ecosystem (cargo)
- No cryptographic signature needed per spec (checksum only)

**Alternatives Considered**:
- SHA1: Deprecated, collision vulnerabilities
- MD5: Broken, not suitable for integrity verification
- BLAKE3: Faster but less universal, adds dependency

### 3. Version Comparison

**Decision**: Semantic versioning with `semver` crate

**Rationale**:
- Standard version format (MAJOR.MINOR.PATCH)
- Handles pre-release versions (1.0.0-beta.1)
- Well-tested library in Rust ecosystem
- Spec explicitly mentions semantic versioning

**Alternatives Considered**:
- String comparison: Breaks on "1.10.0" vs "1.9.0"
- Custom parser: Reinventing the wheel

### 4. Manifest Format

**Decision**: TOML

**Rationale**:
- Consistent with other plix configuration files
- Human-readable and editable
- Already in workspace dependencies (serde + toml)
- Easy to validate with serde

**Alternatives Considered**:
- JSON: Less readable, more verbose
- YAML: Adds dependency, indentation-sensitive

### 5. Directory Structure

**Decision**: XDG Base Directory Specification

**Rationale**:
- Standard on Linux (`~/.config/`, `~/.local/share/`)
- `dirs-next` crate handles platform differences
- Windows maps to `%APPDATA%` appropriately
- Consistent with existing plix profile storage

**Paths**:
| Purpose | Linux | Windows |
|---------|-------|---------|
| Config | `~/.config/plix/launcher.toml` | `%APPDATA%\plix\launcher.toml` |
| Data | `~/.local/share/plix/` | `%LOCALAPPDATA%\plix\` |
| Logs | `~/.local/share/plix/logs/` | `%LOCALAPPDATA%\plix\logs\` |

### 6. Atomic File Operations

**Decision**: Write to temp file, then rename

**Rationale**:
- Rename is atomic on most file systems (POSIX, NTFS)
- Prevents partial writes on crash/interrupt
- Existing pattern in plix profile saving

**Implementation**:
```rust
// 1. Write to .tmp file
fs::write(path.with_extension("tmp"), content)?;
// 2. Atomic rename
fs::rename(path.with_extension("tmp"), path)?;
```

### 7. Progress Reporting

**Decision**: Console output with inline updates

**Rationale**:
- Spec requires minimal UI (console acceptable)
- No GUI framework needed
- Cross-platform terminal output

**Format**:
```
[INFO] Checking for updates...
[INFO] Update available: 1.2.3 -> 1.3.0
[INFO] Downloading plix-client (45.2 MB)...
[=============================>      ] 75% (33.9 MB / 45.2 MB)
[INFO] Verifying integrity...
[INFO] Installing version 1.3.0...
[INFO] Launching game...
```

### 8. Error Handling Strategy

**Decision**: Custom error enum with `thiserror`

**Rationale**:
- Follows existing plix patterns
- Clear error messages for users
- Proper error propagation

**Error Categories**:
- `NetworkError`: HTTP failures, timeouts
- `ManifestError`: Parse failures, validation errors
- `ChecksumError`: Integrity verification failures
- `InstallError`: File system operations
- `LaunchError`: Game process spawning

### 9. Single Instance Enforcement

**Decision**: File-based lock in data directory

**Rationale**:
- Simple, cross-platform
- No external dependencies
- Lock file removed on normal exit

**Implementation**:
```rust
// Create lock file with PID
let lock_path = data_dir().join("launcher.lock");
if lock_path.exists() {
    // Check if PID is still running
    // If not, remove stale lock
}
fs::write(&lock_path, std::process::id().to_string())?;
// On exit: remove lock file
```

### 10. Offline Mode Detection

**Decision**: Timeout-based with fallback

**Rationale**:
- Try to fetch manifest with short timeout (5s)
- If fails, check for valid local installation
- Launch with existing version if available

**Flow**:
```
1. Try fetch manifest (5s timeout)
2. If success → normal update flow
3. If timeout/error:
   a. If local version valid → launch (offline mode)
   b. If no local version → error message
```

## Dependency Analysis

### New Dependencies Required

| Crate | Version | Purpose | Size Impact |
|-------|---------|---------|-------------|
| `sha2` | 0.10 | SHA256 checksums | ~50KB |
| `semver` | 1.0 | Version comparison | ~20KB |

### Existing Workspace Dependencies Used

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client (blocking) |
| `serde` | Serialization |
| `toml` | Config format |
| `clap` | CLI parsing |
| `tracing` | Logging |
| `tracing-subscriber` | Log output |
| `thiserror` | Error handling |
| `dirs-next` | Platform paths |

### Binary Size Estimate

- Base Rust binary: ~2MB
- reqwest + TLS: ~5MB
- Other deps: ~1MB
- **Total estimate**: ~8MB (under 10MB requirement)

## Platform-Specific Considerations

### Linux

- Binary execution: `chmod +x` then spawn
- Symlinks: Use for `current/` → `versions/{version}/`
- Permissions: No elevation needed

### Windows

- Binary execution: Direct spawn of `.exe`
- Symlinks: Use directory junction or copy (symlinks require admin)
- Permissions: No elevation needed

**Windows Symlink Decision**: Use directory copy instead of symlink to avoid UAC prompt. Copy is fast for small file count.

## Integration Points

### With plix-client

- Launcher spawns `plix-client` binary
- Passes through CLI arguments
- No direct code dependency

### With Feature 028 (Dedicated Server Packaging)

- Manifest can be hosted alongside server releases
- Same HTTP/CDN infrastructure
- Compatible versioning scheme

### With Future Features

- Manifest format extensible for:
  - Protocol version compatibility
  - Asset packs
  - Optional components

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Network failures | High | Medium | Retry logic, offline mode |
| Corrupted downloads | Low | High | SHA256 verification |
| Partial installs | Medium | High | Atomic operations |
| Platform differences | Medium | Medium | Platform-specific code paths |
| Binary size bloat | Low | Low | Monitor with `cargo bloat` |

## Open Questions (Resolved)

1. **Q**: Should manifest include delta patches?
   **A**: No - full file replacement for v1 (per spec)

2. **Q**: Should launcher self-update?
   **A**: No - out of scope for v1

3. **Q**: How to handle concurrent file access?
   **A**: Single instance lock prevents conflicts

4. **Q**: What if user manually modifies game files?
   **A**: Checksum will detect, prompt re-download

## References

- Existing HTTP pattern: `crates/plix-client/src/server_browser/fetch.rs`
- Config pattern: `crates/plix-client/src/profile/player_profile.rs`
- Atomic write pattern: `crates/plix-client/src/config.rs`
