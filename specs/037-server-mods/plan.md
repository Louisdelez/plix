# Implementation Plan: Server Mods + Client Sync

**Branch**: `037-server-mods` | **Date**: 2025-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/037-server-mods/spec.md`

## Summary

Enable server-only mod execution where WASM mods run exclusively on the server, with an optional synchronization mechanism to send data/configuration payloads to clients. The implementation extends the existing mod distribution system (Feature 036) with a handshake protocol, join policies, chunked payload transfer with SHA-256 verification, client-side caching, and mod network channels.

## Technical Context

**Language/Version**: Rust 1.83+ (stable channel only per constitution)
**Primary Dependencies**: plix-mod-distribution (036), plix-mod-runtime-wasm (035), plix-mod-core (034), bincode, serde, sha2, tokio
**Storage**: File system - `~/.local/share/plix/mods/payloads/` for client payload cache
**Testing**: cargo test, integration tests with mock client-server
**Target Platform**: Linux server (primary), Windows/macOS clients
**Project Type**: Multi-crate workspace (existing plix-server, plix-client, plix-common)
**Performance Goals**: Join <5s for server-only, sync <30s for 25MB payloads, 100ms message delivery
**Constraints**: <1MB memory per client during sync, 8 concurrent inflight chunks max, 25MB payload limit
**Scale/Scope**: 100 concurrent payload syncs, 50 mods validated in <2s

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | Server is source of truth, clients never execute mod code, SHA-256 verification |
| II. Performance (Low Latency) | PASS | Chunked transfers, configurable limits, no blocking operations |
| III. Architecture (Engine-First) | PASS | Extends existing mod system layers, no new fundamental primitives |
| IV. Modding (First-Class) | PASS | Automatic sync (IV.6), documented API, mod network channels |
| V. Code Quality | PASS | Integration tests required, structured logging, no panics |
| VI. Technical Standards | PASS | Stable Rust, explicit serialization, versioned protocol |
| VII. Player Experience | PASS | Zero prerequisites for server-only mods, clear error messages |
| VIII. Open Source | PASS | All code public, documented protocol |
| IX. Scoping & Realism | PASS | MVP scope is minimal, focused on core sync + policy |
| X. Long-Term Vision | PASS | Versioned protocol supports evolution |

**Gate Status**: PASSED - No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/037-server-mods/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       └── protocol/
│           └── messages.rs    # Extended with mod handshake messages
│
├── plix-server/
│   └── src/
│       └── mods/
│           ├── mod.rs              # Existing mod manager
│           ├── distribution.rs     # Existing (Feature 036)
│           ├── modset.rs           # NEW: ModSetDescriptor construction
│           ├── join_policy.rs      # NEW: Join decision logic
│           ├── client_payload.rs   # NEW: Payload packaging + hashing
│           └── payload_transfer.rs # NEW: Chunked payload streaming
│
├── plix-client/
│   └── src/
│       ├── net.rs                  # Extended with handshake handling
│       └── mods/                   # NEW directory
│           ├── mod.rs              # Module exports
│           ├── handshake.rs        # ModSetResponse, JoinDecision handling
│           ├── payload_cache.rs    # SHA-256 indexed cache
│           └── payload_receiver.rs # Chunk reassembly + verification
│
└── plix-mod-distribution/
    └── src/
        └── lockfile.rs             # Extended to expose data for ModSetDescriptor

tests/
├── fixtures/
│   └── mock_registry/              # Existing (Feature 036)
└── integration/
    └── mod_sync_test.rs            # NEW: Client-server sync tests
```

**Structure Decision**: Extends existing multi-crate structure. Server mod logic in `plix-server/src/mods/`, client mod handling in new `plix-client/src/mods/` directory.

## Architecture Overview

### Component Diagram

```text
┌─────────────────────────────────────────────────────────────────┐
│                         SERVER                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐   ┌────────────────┐   ┌──────────────────┐  │
│  │ ModManager   │──▶│ ModSetBuilder  │──▶│ ModSetDescriptor │  │
│  │ (035/036)    │   │ (from lockfile)│   │                  │  │
│  └──────────────┘   └────────────────┘   └────────┬─────────┘  │
│                                                    │            │
│  ┌──────────────┐   ┌────────────────┐            │            │
│  │ JoinPolicy   │◀──│ JoinDecision   │◀───────────┘            │
│  │ (config)     │   │ (evaluate)     │                          │
│  └──────────────┘   └────────┬───────┘                          │
│                              │                                   │
│  ┌──────────────────────────▼───────────────────────┐          │
│  │ PayloadTransfer                                   │          │
│  │ - build_payload_archive()                         │          │
│  │ - stream_chunks() → S2C_PayloadChunk             │          │
│  └───────────────────────────────────────────────────┘          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Network
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         CLIENT                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐   ┌────────────────┐                      │
│  │ HandshakeHandler │──▶│ PayloadCache   │                      │
│  │ - process modset │   │ (by SHA-256)   │                      │
│  │ - build response │   └────────────────┘                      │
│  └─────────┬────────┘                                            │
│            │                                                     │
│  ┌─────────▼────────┐   ┌────────────────┐                      │
│  │ PayloadReceiver  │──▶│ Integrity      │                      │
│  │ - receive chunks │   │ Verification   │                      │
│  │ - reassemble     │   │ (SHA-256)      │                      │
│  └──────────────────┘   └────────────────┘                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Protocol Flow

```text
Client                                          Server
  │                                               │
  │─── Connect { ... } ──────────────────────────▶│
  │                                               │
  │◀── S2C_ModSetDescriptor { mods[], ... } ─────│
  │                                               │
  │─── C2S_ModSetResponse { caps, hashes } ──────▶│
  │                                               │
  │◀── S2C_JoinDecision { OK | REFUSED | SYNC } ─│
  │                                               │
  │  [If SYNC_REQUIRED]                           │
  │◀── S2C_PayloadBegin { hash, size, chunks } ──│
  │◀── S2C_PayloadChunk { hash, idx, data } ─────│
  │◀── ... (more chunks) ────────────────────────│
  │◀── S2C_PayloadEnd { hash } ──────────────────│
  │─── C2S_PayloadAck { hash } ──────────────────▶│
  │                                               │
  │◀── S2C_JoinDecision { OK } ──────────────────│
  │                                               │
```

## Implementation Phases

### Phase 1: Data Model & Protocol (Foundation)

- Extend mod manifest with `runtime` and `client_payload` fields
- Define protocol messages in `plix-common/src/protocol/messages.rs`
- Extend `server_mods.toml` config with join_policy and sync sections
- Define `ModSetDescriptor`, `ModEntry`, `ClientCapabilities`, `JoinDecision`

### Phase 2: Server Handshake & Join Policy

- Build `ModSetDescriptor` from lockfile at server startup
- Implement join policy evaluation logic
- Send `S2C_ModSetDescriptor` after `Connect`
- Process `C2S_ModSetResponse` and make join decision
- Log handshake events (FR-025)

### Phase 3: Payload Packaging & Transfer (Server)

- Build deterministic client payload archive from mod bundles
- Calculate SHA-256 hash and enforce size limits
- Implement chunked streaming (256KB chunks, 8 max inflight)
- Handle `C2S_PayloadAck` and `C2S_PayloadResendRequest`
- Log sync events (FR-026)

### Phase 4: Client Payload Handling

- Implement `PayloadCache` with SHA-256 indexing
- Implement `PayloadReceiver` for chunk reassembly
- Verify integrity on completion
- Store to cache on success, purge on failure
- Support cache hit detection in handshake response

### Phase 5: Mod Network Channels

- Extend channel allowlist for `mod:<id>:*` pattern
- Implement client-to-server channel gating
- Verify rate limits via existing Feature 034 infrastructure
- Implement spoof protection (reject `mod:<other_id>:*`)

### Phase 6: Metrics, Testing & Documentation

- Expose metrics: joins_refused, payload_bytes, cache_hits (FR-027)
- Unit tests: handshake parsing, decision matrix, chunking, sha256
- Integration tests: server-only join, payload sync, cache hit, refused
- Documentation: `docs/feature-037.md`

## Complexity Tracking

No constitution violations requiring justification. All components follow established patterns from Features 034-036.

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Payload sync blocks game join | High | Timeout + abort with clear message |
| Memory exhaustion during sync | Medium | Chunk limits (8 inflight), size cap (25MB) |
| Hash collision (theoretical) | Low | SHA-256 is collision-resistant; no action needed |
| Protocol version mismatch | Medium | Version in ModSetDescriptor, refuse outdated clients |

## Dependencies

- **Feature 034**: Mod API Core - network channel infrastructure, rate limiting
- **Feature 035**: WASM Runtime - server-side mod execution
- **Feature 036**: Mod Distribution - lockfile, mod bundles, SHA-256 verification

## Definition of Done

- [ ] Client can join server with server-only mods without installation
- [ ] Payload sync works: chunked transfer, SHA-256 verified, cached
- [ ] Cache hit skips redundant download
- [ ] Client-required missing → join refused with clear message
- [ ] Mod channels safe: spoof blocked, allowlist respected, rate limited
- [ ] Logs/metrics present for observability
- [ ] Unit + integration tests pass
- [ ] Documentation complete at `docs/feature-037.md`
