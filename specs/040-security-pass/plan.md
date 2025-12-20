# Implementation Plan: Security Pass

**Branch**: `040-security-pass` | **Date**: 2025-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/040-security-pass/spec.md`

## Summary

Production-grade security hardening for Plix covering protocol fuzzing, parser hardening, and abuse case protections. The implementation adds centralized limits, decode hardening with no panics, fuzz testing infrastructure, handshake/payload sync abuse protections, and security observability.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (protocol), plix-server (netloop, mods), plix-mod-distribution (registry), plix-mod-core (manifest), bincode (serialization), cargo-fuzz/libfuzzer (fuzzing)
**Storage**: N/A (in-memory security state, limits module is compile-time constants)
**Testing**: cargo test (unit/integration), cargo fuzz (fuzz targets)
**Target Platform**: Linux server (primary), cross-platform client
**Project Type**: Multi-crate workspace (existing structure)
**Performance Goals**: No p95 tick degradation >5% under malicious traffic (SC-010)
**Constraints**: Zero panics on decode/parse, O(n) bounded validation, rate-limited logging
**Scale/Scope**: 64KB max packets, 256 max list items, 200 msg/s per client

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Security (Server Authority) | **PASS** | This feature directly implements server-side input validation, attack surface reduction, and resource limits |
| II. Performance (Low Latency) | **PASS** | O(1)/O(n) bounded validation, no blocking operations, rate-limited logging prevents log DoS |
| III. Architecture (Engine Modularity) | **PASS** | Centralized limits module, clear separation between validation and business logic |
| IV. Modding (First-Class) | **PASS** | Hardening protects mod distribution without breaking mod functionality |
| V. Code Quality (Explicit & Tested) | **PASS** | Mandatory fuzz tests, abuse test suite, no panics policy |
| VI. Technical Standards (Rust) | **PASS** | Stable Rust, cargo clippy/fmt compliance, feature-gated fuzzing |
| VII. Player Experience | **PASS** | Protects legitimate players from malicious client disruption |
| VIII. Open Source | **PASS** | All security measures are auditable, no proprietary dependencies |
| IX. Scoping & Realism | **PASS** | MVP scope: 3 fuzz targets, essential abuse tests, core limits |
| X. Long-Term Vision | **PASS** | Centralized limits support evolution, fuzz tests prevent regressions |

**Gate Status**: PASSED - No violations

## Project Structure

### Documentation (this feature)

```text
specs/040-security-pass/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal APIs)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── limits.rs              # NEW: Centralized security limits module
│       └── protocol/
│           └── messages.rs        # MODIFY: Add bounded decode helpers
│
├── plix-server/
│   └── src/
│       ├── security/              # NEW: Security subsystem
│       │   ├── mod.rs
│       │   ├── limits.rs          # Re-export plix-common limits + server-specific
│       │   ├── strikes.rs         # Strike tracking per connection
│       │   ├── rate_limiter.rs    # Token bucket implementation
│       │   ├── observability.rs   # Counters + rate-limited logging
│       │   └── handshake.rs       # Handshake timeout + pending connection tracking
│       ├── mods/
│       │   └── sync_session.rs    # MODIFY: Add abuse protections
│       └── netloop.rs             # MODIFY: Integrate security checks
│
├── plix-mod-distribution/
│   └── src/
│       ├── registry.rs            # MODIFY: Add size limits
│       ├── bundle.rs              # MODIFY: Add zip safety (path traversal, bomb detection)
│       └── integrity.rs           # MODIFY: Integrate strike on hash mismatch
│
├── plix-mod-core/
│   └── src/
│       └── manifest.rs            # MODIFY: Add size limits
│
└── plix-fuzz/                     # NEW: Fuzz testing crate
    ├── Cargo.toml
    ├── fuzz_targets/
    │   ├── fuzz_decode_client_message.rs
    │   ├── fuzz_decode_server_message.rs
    │   └── fuzz_decode_modsync_chunk.rs
    └── corpus/
        └── (seed files)

tests/
├── security/                      # NEW: Abuse test suite
│   ├── mod.rs
│   ├── decode_abuse_test.rs
│   ├── handshake_abuse_test.rs
│   ├── payload_sync_abuse_test.rs
│   └── parser_abuse_test.rs

docs/
└── security/                      # NEW: Security documentation
    ├── threat-model.md
    ├── limits.md
    ├── fuzzing.md
    └── abuse-cases.md
```

**Structure Decision**: Extending existing multi-crate workspace. New `security/` module in plix-server contains all security subsystems. New `plix-fuzz/` crate for fuzzing (feature-gated, not in default build). Security documentation in `docs/security/`.

## Complexity Tracking

> No constitution violations requiring justification.

## Research Areas (Phase 0)

1. **Existing decode implementation**: How does bincode handle size limits? Can we add pre-decode size checks?
2. **Token bucket vs fixed-window**: Existing anti_cheat uses fixed-window; evaluate migration path to token bucket
3. **Fuzz infrastructure**: cargo-fuzz vs libfuzzer directly, corpus generation strategy
4. **Zip library safety**: Does the current zip library (if any) support streaming extraction with limits?
5. **Rate-limited logging**: Best practices for tracing crate with per-category rate limits

## Design Deliverables (Phase 1)

1. **data-model.md**: SecurityLimits, StrikeTracker, RateLimiter, FuzzCorpus entities
2. **contracts/**: Internal API contracts for security module
3. **quickstart.md**: How to run fuzz targets, understand limits, interpret security logs
