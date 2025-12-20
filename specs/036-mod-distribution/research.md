# Research: Mod Distribution

**Feature Branch**: `036-mod-distribution`
**Date**: 2025-12-18
**Status**: Complete

## Overview

This document evaluates technical options for implementing the mod distribution system, including crate selection, format decisions, and algorithm choices.

## Crate Evaluations

### 1. Version Parsing: `semver`

**Selected**: `semver` 1.0.x

| Criteria | Score | Notes |
|----------|-------|-------|
| Maturity | ★★★★★ | Rust ecosystem standard, used by Cargo |
| SemVer Compliance | ★★★★★ | Full spec compliance including pre-release |
| API Quality | ★★★★★ | Clean `Version`, `VersionReq` types |
| Performance | ★★★★★ | Zero-copy parsing, efficient comparisons |
| Maintenance | ★★★★★ | Actively maintained by dtolnay |

**Key Features**:
- `Version`: Parses `1.2.3`, `1.2.3-alpha.1+build`
- `VersionReq`: Parses `^1.0`, `~1.2`, `>=1.0, <2.0`
- `matches()`: Check if version satisfies requirement

**Example**:
```rust
use semver::{Version, VersionReq};

let req = VersionReq::parse("^1.2.0")?;
let v1 = Version::parse("1.5.0")?;
let v2 = Version::parse("2.0.0")?;

assert!(req.matches(&v1));  // true
assert!(!req.matches(&v2)); // false
```

**Alternatives Considered**:
- `lenient_semver`: More permissive parsing, but less strict compliance
- Custom parser: No benefit, higher maintenance

**Decision**: Use `semver` - industry standard with perfect fit.

---

### 2. Archive Format: `zip`

**Selected**: `zip` 2.x

| Criteria | Score | Notes |
|----------|-------|-------|
| Maturity | ★★★★★ | Widely used, stable API |
| Determinism | ★★★★☆ | Achievable with sorted entries + fixed timestamps |
| Compression | ★★★★★ | Supports Deflate, Zstd, Store |
| Streaming | ★★★★☆ | Read streaming, write requires seek |
| Maintenance | ★★★★★ | Active development |

**Key Features**:
- `ZipArchive::new()`: Read existing archives
- `ZipWriter::new()`: Create new archives
- Entry iteration with metadata access
- Compression level control

**Determinism Strategy**:
```rust
use zip::{ZipWriter, write::FileOptions, DateTime};
use std::io::Write;

fn create_deterministic_bundle(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));

    // Sort entries for determinism
    let mut sorted: Vec<_> = files.iter().collect();
    sorted.sort_by_key(|(name, _)| name.as_str());

    // Fixed timestamp: 2020-01-01 00:00:00
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(DateTime::from_date_and_time(2020, 1, 1, 0, 0, 0).unwrap());

    for (name, data) in sorted {
        zip.start_file(name, options)?;
        zip.write_all(data)?;
    }

    zip.finish()?;
    buf
}
```

**Alternatives Considered**:
- `tar` + `flate2`: Less tooling support, determinism harder
- `async-zip`: Async but less mature
- Custom format: No benefit

**Decision**: Use `zip` with deterministic creation strategy.

---

### 3. Signature Algorithm: `ed25519-dalek`

**Selected**: `ed25519-dalek` 2.x (optional feature)

| Criteria | Score | Notes |
|----------|-------|-------|
| Security | ★★★★★ | Modern EdDSA, no known weaknesses |
| Performance | ★★★★★ | ~15,000 sign/verify per second |
| Key Size | ★★★★★ | 32-byte keys, 64-byte signatures |
| Pure Rust | ★★★★★ | No C dependencies |
| Maintenance | ★★★★★ | dalek-cryptography team |

**Key Features**:
- `SigningKey`: Private key for signing
- `VerifyingKey`: Public key for verification
- `Signature`: Detached signature type

**Example**:
```rust
use ed25519_dalek::{SigningKey, Signature, VerifyingKey};
use ed25519_dalek::Signer;

// Signing (mod author)
let signing_key = SigningKey::generate(&mut rand::thread_rng());
let signature: Signature = signing_key.sign(bundle_bytes);

// Verification (server)
let verifying_key: VerifyingKey = signing_key.verifying_key();
let sig_bytes: [u8; 64] = signature.to_bytes();

// Later, with only public key
let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes)?;
let signature = Signature::from_bytes(&sig_bytes);
verifying_key.verify(bundle_bytes, &signature)?;
```

**Alternatives Considered**:
- `ring`: Faster but C dependencies, larger binary
- RSA: Larger signatures (256+ bytes), slower
- `p256` (ECDSA): More complex, less suitable for signing

**Decision**: Use `ed25519-dalek` as optional feature - pure Rust, fast, small signatures.

---

### 4. HTTP Client: `reqwest` (existing)

Already in workspace. Key features for this use case:
- Timeout configuration (connect + read)
- Streaming response body
- Retry logic (manual, via wrapper)

**Usage Pattern**:
```rust
use reqwest::Client;
use std::time::Duration;

let client = Client::builder()
    .connect_timeout(Duration::from_secs(30))
    .read_timeout(Duration::from_secs(120))
    .build()?;

// Streaming download
let mut response = client.get(url).send().await?;
let mut file = tokio::fs::File::create(path).await?;

while let Some(chunk) = response.chunk().await? {
    file.write_all(&chunk).await?;
}
```

---

### 5. Hashing: `sha2` (existing)

Already in workspace. Usage for streaming hash:

```rust
use sha2::{Sha256, Digest};
use tokio::io::AsyncReadExt;

async fn hash_file(path: &Path) -> [u8; 32] {
    let mut hasher = Sha256::new();
    let mut file = tokio::fs::File::open(path).await?;
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    hasher.finalize().into()
}
```

---

## Algorithm Decisions

### Dependency Resolution: Greedy Latest-Compatible

**Selected**: Simple greedy algorithm selecting newest compatible version

**Algorithm**:
```
Input: required_mods: Map<ModId, VersionReq>
Output: resolved: Map<ModId, Version> or Error

1. Initialize resolved = {}
2. Initialize queue = required_mods.keys()
3. While queue not empty:
   a. Pop mod_id from queue
   b. If mod_id in resolved, skip (already resolved)
   c. Get all available versions from registries
   d. Filter to versions matching all requirements for mod_id
   e. Select newest compatible version
   f. If none found, return EMREG006 (conflict)
   g. Add to resolved: mod_id -> selected_version
   h. For each dependency of selected_version:
      - Add dependency constraint to requirements
      - Add dependency mod_id to queue
4. Check for cycles (DFS on dependency graph)
5. Return resolved
```

**Cycle Detection**:
```rust
fn detect_cycles(deps: &HashMap<ModId, Vec<ModId>>) -> Option<Vec<ModId>> {
    enum State { Unvisited, InProgress, Done }
    let mut states: HashMap<_, _> = deps.keys().map(|k| (k, State::Unvisited)).collect();
    let mut path = Vec::new();

    fn dfs(node: &ModId, deps: &HashMap<ModId, Vec<ModId>>,
           states: &mut HashMap<&ModId, State>, path: &mut Vec<ModId>) -> Option<Vec<ModId>> {
        match states.get(&node) {
            Some(State::InProgress) => return Some(path.clone()),
            Some(State::Done) => return None,
            _ => {}
        }

        states.insert(node, State::InProgress);
        path.push(node.clone());

        if let Some(children) = deps.get(node) {
            for child in children {
                if let Some(cycle) = dfs(child, deps, states, path) {
                    return Some(cycle);
                }
            }
        }

        path.pop();
        states.insert(node, State::Done);
        None
    }

    for node in deps.keys() {
        if let Some(cycle) = dfs(node, deps, &mut states, &mut path) {
            return Some(cycle);
        }
    }
    None
}
```

**Complexity**: O(V + E) where V = mod count, E = dependency edges

**Why Not SAT Solver (pubgrub)?**:
- Overkill for MVP - most mod graphs are simple
- pubgrub adds ~5KB to binary size
- Greedy is predictable and debuggable
- Can upgrade later if needed

---

### Registry Priority

**Strategy**: First-match wins

When multiple registries have the same mod:
1. Iterate registries in config order
2. First registry with matching version wins
3. No version merging across registries

**Rationale**: Simple, predictable, allows private registries to shadow public.

---

### Cache Strategy

**Content-Addressed Bundles**:
- Store downloaded bundles as `bundles/<sha256>.plixmod`
- Prevents duplicate downloads
- Enables integrity verification without re-downloading

**Version-Keyed Installations**:
- Extract to `installed/<mod_id>/<version>/`
- Allows multiple versions to coexist
- Enables quick switching (lockfile change)

---

## Risk Mitigations

| Risk | Mitigation |
|------|------------|
| Large dependency graphs | Depth limit: 50 levels max |
| Slow resolution | Cache index locally, timeout: 5s |
| Registry unavailable | Graceful fallback to cache |
| Bundle corruption | SHA-256 verification, delete on mismatch |
| Disk space exhaustion | Pre-check available space, size limits |

---

## Test Strategy

### Unit Tests
- `semver` constraint parsing and matching
- Cycle detection algorithm
- Hash verification
- Lockfile serialization/deserialization

### Integration Tests
- Full resolution with mock registry
- Download + verify + extract flow
- Cache hit/miss scenarios
- Error code validation (EMREG001-008)

### Fixtures
- `mock_registry/` with sample mods and dependencies
- Pre-computed hashes and signatures
- Conflict scenarios for testing error paths

---

## Conclusion

All technical decisions align with:
- Constitution requirements (stable Rust, no proprietary deps)
- Performance targets (<5s resolution, <60s installation)
- Security requirements (SHA-256 mandatory, Ed25519 optional)

Ready to proceed with Phase 1: data model and contracts.
