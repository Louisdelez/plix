# Implementation Plan: Anti-Cheat Hardening

**Branch**: `007-anti-cheat-hardening` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-anti-cheat-hardening/spec.md`

## Summary

Harden the existing server-authoritative architecture against common cheats and abuse through:
1. **Strict Input Validation** - Reject NaN/INF, out-of-bounds values, and invalid sequences
2. **Per-Action Rate Limiting** - Fixed-window rate limiters for inputs, attacks, block edits, and ready toggles
3. **Movement Sanity Checks** - Detect speed hacks and teleportation via delta position/velocity validation
4. **Progressive Sanctions** - Warning → Kick → Temp Ban escalation with configurable thresholds
5. **Ban Persistence** - In-memory ban list for MVP (clears on restart per spec)

The server already has an `InputValidator` struct and some validation constants; this feature integrates and expands them into a comprehensive anti-cheat subsystem.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: tokio (async), bincode (serialization), glam (math)
**Storage**: In-memory only for MVP (ban list clears on restart per spec)
**Testing**: cargo test (unit + integration), existing load tests (plix-tools)
**Target Platform**: Linux/Windows server (headless), client-agnostic
**Project Type**: Multi-crate workspace (plix-server, plix-common, plix-client, plix-tools)
**Performance Goals**: <1µs per check per player per tick, zero heap allocations in hot path
**Constraints**: No false positives at 60Hz tick rate, must not break headless/bot tests
**Scale/Scope**: 16 concurrent players max, 60Hz tick rate, ~120 inputs/sec per player max

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I. Security (Server Authority & Isolation) ✅
- **Server Authoritative Architecture**: ✅ All validation happens server-side; client inputs are treated as untrusted suggestions
- **Anti-Cheat Baseline**: ✅ This feature directly implements the constitution requirement to "detect and reject basic cheats (speedhack, fly, state falsification) at the protocol level"

### Principle II. Performance (Low Latency & Stability) ✅
- **Tick Stability**: ✅ Design targets <1µs per check with zero heap allocations; will not affect 60Hz tick rate
- **Event-Driven Updates**: ✅ Validation integrates into existing tick loop without new polling/iteration

### Principle V. Code Quality (Explicit & Tested) ✅
- **Mandatory Testing**: ✅ Spec requires unit tests for validation, rate limiting, sanctions, and integration tests for no false positives
- **Structured Logging**: ✅ All violations will be logged with structured key-value format (player, reason, metric, value)
- **No Panics in Production**: ✅ Core goal is to never panic on any input; all error paths handled explicitly

### Principle VI. Technical Standards (Rust & Reproducibility) ✅
- **Stable Rust Only**: ✅ No nightly features required
- **Tooling Compliance**: ✅ Must pass cargo clippy and cargo fmt
- **Deterministic APIs**: ✅ Sanction escalation is fully deterministic (strikes → warning → kick → ban)

### Principle IX. Scoping & Realism (Minimal Viable Scope) ✅
- **Minimal MVP**: ✅ Spec explicitly lists non-goals (no ML, no HWID, no external services)
- **Simple Over Complex**: ✅ Using fixed-window rate limiting instead of token bucket; in-memory bans instead of persistence

### Gate Status: **PASSED** - All relevant principles satisfied, no violations to justify

## Project Structure

### Documentation (this feature)

```text
specs/007-anti-cheat-hardening/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal API contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-server/
│   ├── src/
│   │   ├── lib.rs                    # Server struct, tick loop (modify)
│   │   ├── session.rs                # ServerPlayer, SessionManager (modify)
│   │   ├── validation.rs             # InputValidator (expand significantly)
│   │   ├── anti_cheat/               # NEW module
│   │   │   ├── mod.rs                # AntiCheatSubsystem, integration
│   │   │   ├── config.rs             # AntiCheatConfig struct
│   │   │   ├── rate_limiter.rs       # FixedWindowLimiter per action
│   │   │   ├── sanctions.rs          # SanctionManager, escalation logic
│   │   │   └── ban_list.rs           # BanList (in-memory)
│   │   └── sim/
│   │       └── movement.rs           # Movement system (read for constants)
│   └── tests/
│       ├── anti_cheat_test.rs        # NEW: unit tests for anti-cheat
│       ├── combat_test.rs            # Existing (verify no regression)
│       ├── movement_test.rs          # Existing (verify no regression)
│       └── block_edit_test.rs        # Existing (verify no regression)
│
├── plix-common/
│   └── src/
│       └── protocol/
│           └── messages.rs           # Add Warning message variant (if needed)
│
└── plix-tools/
    └── tests/
        └── load_test.rs              # Verify no false positives under load
```

**Structure Decision**: Multi-crate Rust workspace. Anti-cheat logic goes in `plix-server` as a new `anti_cheat` module, with minimal changes to `plix-common` for any new message variants.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations to justify - all design decisions align with constitutional principles.

## Post-Design Constitution Re-Check

*Re-evaluated after Phase 1 design artifacts completed.*

### Design Decisions Verified Against Constitution

| Decision | Principle | Status |
|----------|-----------|--------|
| Fixed-window rate limiting (not token bucket) | IX. Simple Over Complex | ✅ Simpler, meets requirements |
| In-memory ban list (not persisted) | IX. Minimal MVP | ✅ Per spec, persistence deferred |
| Per-player inline state (not HashMap) | II. Performance | ✅ O(1) lookup, cache-friendly |
| NaN/INF rejection (fail-closed) | I. Server Authority | ✅ Never trust client data |
| Deterministic escalation thresholds | VI. Deterministic APIs | ✅ Same input = same sanction |
| Structured logging for violations | V. Structured Logging | ✅ Key-value format per constitution |

### No New Violations Introduced

The Phase 1 design:
- Adds ~6 new files, all in `plix-server` crate
- No new external dependencies
- No nightly Rust features
- No breaking changes to existing APIs
- All new code will have unit tests

### Gate Status: **PASSED** - Design phase complete, ready for task generation

## Generated Artifacts

| Artifact | Location | Status |
|----------|----------|--------|
| Implementation Plan | `specs/007-anti-cheat-hardening/plan.md` | ✅ Complete |
| Research | `specs/007-anti-cheat-hardening/research.md` | ✅ Complete |
| Data Model | `specs/007-anti-cheat-hardening/data-model.md` | ✅ Complete |
| API Contracts | `specs/007-anti-cheat-hardening/contracts/anti_cheat_api.md` | ✅ Complete |
| Quickstart Guide | `specs/007-anti-cheat-hardening/quickstart.md` | ✅ Complete |
| Tasks | `specs/007-anti-cheat-hardening/tasks.md` | ⏳ Next: `/speckit.tasks` |

## Next Steps

Run `/speckit.tasks` to generate the implementation task list from these design artifacts.
