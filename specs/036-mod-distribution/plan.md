# Implementation Plan: Mod Distribution

**Branch**: `036-mod-distribution` | **Date**: 2025-12-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/036-mod-distribution/spec.md`

## Summary

Implement a production-ready mod distribution system enabling servers to:
- Read mod configuration (registries + required mods with version constraints)
- Resolve dependencies using SemVer constraints and generate reproducible lockfiles
- Download mod bundles from remote/local registries with caching
- Verify integrity (SHA-256 mandatory) and signatures (optional)
- Install/extract mods into local cache
- Load mods via `plix-mod-core` (034) and `plix-mod-runtime-wasm` (035)

## Technical Context

**Language/Version**: Rust 1.83+ (stable channel only per constitution)
**Primary Dependencies**:
- `semver` (version parsing and constraint resolution)
- `reqwest` (HTTP client, already in use)
- `sha2` (SHA-256 hashing)
- `zip` (bundle extraction)
- `serde` + `serde_json` (index/lockfile serialization)
- `ed25519-dalek` (optional signature verification)
- `tracing` (logging, already in use)
- `plix-mod-core` (manifest parsing, mod registry)
- `plix-mod-runtime-wasm` (WASM mod execution)

**Storage**: File system - `~/.local/share/plix/mods/` for cache, `mods.lock` in server directory
**Testing**: `cargo test` for unit/integration tests
**Target Platform**: Linux server (primary), cross-platform (secondary)
**Project Type**: Rust crate (monorepo workspace member)
**Performance Goals**:
- Dependency resolution: <5s for 50+ mods
- Installation: <60s for typical 5-mod setup
- Hash verification: streaming (no full file in memory)

**Constraints**:
- Bundle max size: 50 MB default (configurable)
- Download timeouts: 30s connect, 120s read
- Retries: 3 attempts
- Offline-first: local registry must work without network

**Scale/Scope**: 50+ mods with complex dependency graphs, 1000+ versions per registry

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | SHA-256 integrity mandatory, optional signatures, no code execution during distribution |
| I. Mod Sandboxing | ✅ PASS | Distribution only; execution handled by 034/035 with sandboxing |
| I. Resource Limits | ✅ PASS | Bundle size limits, download timeouts enforced |
| II. Performance | ✅ PASS | Streaming hash verification, lazy downloading, caching |
| III. Architecture (Modularity) | ✅ PASS | New crate `plix-mod-distribution` with clear boundaries |
| III. API Versioning | ✅ PASS | Registry index versioned (`registry_version`), lockfile versioned |
| IV. Modding (First-Class) | ✅ PASS | Automatic mod sync via lockfile, no manual installation required |
| V. Code Quality | ✅ PASS | Structured errors (EMREG001-008), mandatory testing |
| VI. Stable Rust | ✅ PASS | Using Rust 1.83 stable |
| VI. Deterministic APIs | ✅ PASS | Same inputs → same lockfile output |
| VI. Explicit Serialization | ✅ PASS | JSON schemas documented for index and lockfile |
| VIII. Open Source | ✅ PASS | No proprietary dependencies |
| IX. Minimal MVP | ✅ PASS | Signatures optional, no UI, server-only |

**Gate Status**: ✅ PASS - No violations requiring justification

## Project Structure

### Documentation (this feature)

```text
specs/036-mod-distribution/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (index schema, lockfile schema)
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/
├── plix-mod-distribution/     # NEW CRATE
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # Public API, re-exports
│       ├── config.rs          # server_mods.toml parsing
│       ├── index.rs           # Registry index schema + validation
│       ├── registry.rs        # Local + HTTP registry sources
│       ├── resolver.rs        # SemVer dependency resolution
│       ├── lockfile.rs        # mods.lock read/write
│       ├── downloader.rs      # HTTP fetch with retries/timeouts
│       ├── integrity.rs       # SHA-256 verification
│       ├── signatures.rs      # Optional Ed25519 signature verification
│       ├── installer.rs       # Bundle extraction + cache management
│       ├── errors.rs          # EMREG001-008 error codes
│       └── bundle.rs          # .plixmod format handling
│
├── plix-server/
│   └── src/
│       └── mods/
│           └── distribution.rs  # Integration: resolve_install_load()
│
└── plix-mod-core/              # EXISTING - manifest parsing reused

tests/
├── fixtures/
│   └── mock_registry/         # Test registry with sample mods
│       ├── index.json
│       └── mods/
│           ├── test-mod-a-1.0.0.plixmod
│           ├── test-mod-b-1.0.0.plixmod
│           └── test-mod-c-2.0.0.plixmod
└── integration/
    └── mod_distribution_test.rs
```

**Structure Decision**: Single new crate `plix-mod-distribution` with clear module boundaries. Integration point in `plix-server/src/mods/distribution.rs` to wire into server startup.

## Architecture Overview

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `config.rs` | Parse `server_mods.toml`: registries, required mods, trust policy |
| `index.rs` | Registry index schema (v1), parsing, validation |
| `registry.rs` | Unified interface for local/HTTP registries, priority ordering |
| `resolver.rs` | SemVer constraint parsing, dependency graph, cycle/conflict detection |
| `lockfile.rs` | `mods.lock` serialization, deterministic output |
| `downloader.rs` | HTTP GET with timeouts, retries, streaming to disk |
| `integrity.rs` | SHA-256 hash calculation and verification |
| `signatures.rs` | Ed25519 signature verification (optional feature) |
| `installer.rs` | Zip extraction, cache directory management |
| `bundle.rs` | `.plixmod` format: deterministic zip creation/reading |
| `errors.rs` | Error types: EMREG001-008 with structured context |

### Data Flow

```
server_mods.toml
       │
       ▼
   [Config]
       │
       ▼
   [Registry] ◄──── index.json (HTTP/local)
       │
       ▼
   [Resolver] ──► mods.lock
       │
       ▼
  [Downloader] ◄── .plixmod bundles
       │
       ▼
  [Integrity] ──► SHA-256 verify
       │
       ▼
  [Signatures] ──► (optional) Ed25519 verify
       │
       ▼
  [Installer] ──► extracted mods in cache
       │
       ▼
  plix-mod-core ──► manifest loaded
       │
       ▼
  plix-mod-runtime-wasm ──► WASM mods initialized
```

## Key Design Decisions

### 1. Bundle Format: ZIP

**Decision**: Use ZIP for `.plixmod` bundles
**Rationale**:
- `zip` crate is well-maintained and pure Rust
- Deterministic output achievable with sorted entries + fixed timestamps
- Wide tooling support for inspection/debugging
**Alternatives Rejected**:
- tar.gz: Less deterministic without extra work, less tooling
- Custom format: No benefit, higher maintenance

### 2. Signature Algorithm: Ed25519

**Decision**: Use Ed25519 via `ed25519-dalek` (optional feature)
**Rationale**:
- Fast, secure, small signatures (64 bytes)
- Pure Rust implementation
- Widely adopted (SSH, TUF, etc.)
**Alternatives Rejected**:
- RSA: Larger keys/signatures, slower
- ECDSA/secp256k1: More complex, less suitable for signing

### 3. Resolver Algorithm: Greedy Latest-Compatible

**Decision**: Simple greedy resolver selecting newest compatible version
**Rationale**:
- Sufficient for MVP (most mods have simple deps)
- Predictable behavior
- Fast O(n) for typical graphs
**Alternatives Rejected**:
- SAT solver (pubgrub): More complex, overkill for MVP
- Backtracking: Harder to debug/understand

### 4. Cache Layout

**Decision**: Content-addressed bundles, version-keyed installations
```
mods_cache/
├── bundles/<sha256>.plixmod        # Downloaded bundles (content-addressed)
├── installed/<mod_id>/<version>/   # Extracted contents
└── indexes/<registry_hash>/        # Cached registry indexes
```
**Rationale**:
- Content-addressing prevents duplicate downloads
- Version-keyed extraction allows multiple versions coexisting
- Separate index cache for offline operation

## Complexity Tracking

> No violations requiring justification - constitution check passed.

## Dependencies

### Required Crates (new)

| Crate | Version | Purpose |
|-------|---------|---------|
| `semver` | 1.0 | SemVer parsing and requirements |
| `zip` | 2.x | Bundle creation/extraction |
| `ed25519-dalek` | 2.x | Signature verification (optional feature) |

### Already in Workspace

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client |
| `sha2` | SHA-256 hashing |
| `serde`, `serde_json` | Serialization |
| `tracing` | Structured logging |
| `tokio` | Async runtime |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| SemVer edge cases | Medium | Low | Use well-tested `semver` crate |
| Large dependency graphs | Low | Medium | Implement depth limit (50 levels) |
| Registry unavailability | Medium | High | Graceful fallback to cache + clear errors |
| Signature key compromise | Low | High | Key rotation documentation, allowlist in config |

## Next Steps

1. **Phase 0**: Complete `research.md` with crate evaluations
2. **Phase 1**: Generate `data-model.md`, `contracts/`, `quickstart.md`
3. **Phase 2**: Generate `tasks.md` via `/speckit.tasks`
