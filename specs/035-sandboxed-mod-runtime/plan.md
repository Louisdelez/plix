# Implementation Plan: Sandboxed Mod Runtime (WASM)

**Branch**: `035-sandboxed-mod-runtime` | **Date**: 2025-12-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/035-sandboxed-mod-runtime/spec.md`

## Summary

Implement a WebAssembly-based sandboxed mod runtime that executes server-side mods safely with:
- Complete sandbox isolation (no filesystem/network/OS access)
- CPU budget enforcement via fuel/epoch interruption (default 5ms per handler)
- Memory limits (default 32 MiB per mod)
- Host ABI v1 exposing plix-mod-core APIs with capability enforcement
- Integration with existing ModRegistry/EventBus from Feature 034

The runtime uses Wasmtime as the WASM backend for its robust fuel-based interruption and security focus.

## Technical Context

**Language/Version**: Rust 1.83+ (stable channel only per constitution)
**Primary Dependencies**: wasmtime (WASM runtime), plix-mod-core (API/capabilities/events), bincode (ABI serialization)
**Storage**: N/A (in-memory only, mods loaded from filesystem at startup)
**Testing**: cargo test (unit + integration), test WASM modules compiled with wasm32-unknown-unknown
**Target Platform**: Linux server (wasm32-unknown-unknown for mod compilation)
**Project Type**: Single project (new crate plix-mod-runtime-wasm)
**Performance Goals**: 60 tick/s with 10 active mods, mod load <100ms, host calls <1ms
**Constraints**: 5ms CPU budget per handler, 32 MiB memory per mod, no WASI
**Scale/Scope**: Up to 10 concurrent mods, each with isolated memory and CPU budgets

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | WASM sandbox isolates mods from OS/FS/network. Capability enforcement on all host calls. |
| I. Mod Sandboxing | ✅ PASS | Wasmtime provides memory isolation, no WASI = no system access |
| I. Resource Limits | ✅ PASS | CPU fuel budgets (5ms) + memory limits (32 MiB) with auto-disable |
| II. Performance (Low Latency) | ✅ PASS | Epoch interruption allows non-blocking budget enforcement |
| II. Tick Stability | ✅ PASS | Mods exceeding budget are interrupted mid-execution |
| III. Architecture (Engine-First) | ✅ PASS | Runtime wraps plix-mod-core primitives, no new simulation logic |
| III. API Versioning | ✅ PASS | abi_version=1 separate from api_version=1 |
| IV. Modding (First-Class) | ✅ PASS | This feature implements the core mod runtime infrastructure |
| IV. Engine Performance Control | ✅ PASS | Auto-disable after 5 consecutive errors |
| V. Code Quality | ✅ PASS | No panics, structured errors (EMOD001-007), mandatory testing |
| VI. Technical Standards | ✅ PASS | Stable Rust, clippy/fmt compliance, documented ABI |
| IX. Scoping (Minimal MVP) | ✅ PASS | MVP excludes hot reload, JIT caching, multi-threading |

**Gate Result**: PASS - No violations requiring justification

## Project Structure

### Documentation (this feature)

```text
specs/035-sandboxed-mod-runtime/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (ABI specification)
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/plix-mod-runtime-wasm/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # Public API, WasmRuntime struct
│   ├── engine.rs              # Wasmtime engine configuration
│   ├── module_loader.rs       # WASM validation and compilation
│   ├── instance.rs            # Per-mod instance management
│   ├── memory.rs              # Linear memory access helpers
│   ├── budgets.rs             # CPU/memory budget tracking
│   ├── errors.rs              # Runtime-specific errors
│   ├── metrics.rs             # Per-mod metrics collection
│   ├── abi/
│   │   ├── mod.rs             # ABI constants, opcodes, codec
│   │   ├── types.rs           # Serializable ABI types
│   │   └── response.rs        # Response buffer management
│   └── host/
│       ├── mod.rs             # Host function linker
│       ├── log.rs             # plix_log implementation
│       ├── caps.rs            # plix_has_capability, version queries
│       ├── events.rs          # plix_subscribe_event, plix_cancel_event
│       ├── world.rs           # plix_world_call implementation
│       ├── entities.rs        # plix_entity_call implementation
│       ├── net.rs             # plix_net_call implementation
│       └── timers.rs          # plix_timer_call implementation
└── tests/
    ├── fixtures/
    │   ├── chat_filter_mod/   # Example mod source + compiled wasm
    │   ├── infinite_loop_mod/ # Malicious test mod
    │   └── memory_bomb_mod/   # Memory exhaustion test mod
    ├── unit/
    │   ├── memory_tests.rs
    │   ├── budget_tests.rs
    │   └── abi_tests.rs
    └── integration/
        ├── mod_lifecycle_test.rs
        ├── event_dispatch_test.rs
        └── malicious_mod_test.rs

crates/plix-server/src/mods/
├── mod.rs                     # (existing) Extended with WASM integration
└── wasm_bridge.rs             # (new) Bridge between ModManager and WasmRuntime
```

**Structure Decision**: New crate `plix-mod-runtime-wasm` following existing workspace pattern. Server integration via `wasm_bridge.rs` in plix-server's existing mods module.

## Complexity Tracking

> No violations requiring justification. All complexity is essential for security and performance guarantees.

## Key Design Decisions

### 1. WASM Runtime Choice: Wasmtime

**Decision**: Use Wasmtime over Wasmer
**Rationale**:
- Industry-standard security focus (Bytecode Alliance)
- Robust fuel-based CPU interruption
- Better epoch interruption for cooperative multitasking
- More active Rust ecosystem integration

### 2. ABI Serialization: bincode

**Decision**: Use bincode for ABI payload serialization
**Rationale**:
- Already in workspace dependencies
- Fast binary format with zero-copy support
- Simple integration with serde

### 3. Response Buffer Strategy: Host-Owned

**Decision**: Host provides response buffer via `plix_response_ptr/len`
**Rationale**:
- Simpler than mod-provided out-buffers
- Host controls memory layout
- Single allocation per call

### 4. Error Mapping Strategy

| Runtime Error | Maps To | Action |
|---------------|---------|--------|
| Invalid WASM | EMOD001 | Reject load |
| Trap (OOB/div0) | EMOD004 | Increment error count |
| Fuel exhausted | EMOD005 | Interrupt + increment |
| Memory limit | EMOD005 | Trap + increment |
| Missing import | EMOD007 | Reject load |

## Integration Points

### With plix-mod-core (Feature 034)

- `ModRegistry`: Load manifest, get effective capabilities
- `EventBus`: Subscribe to events, dispatch to mods
- `Capability`: Check permissions in host functions
- `ModApiError`: Serialize errors in ABI responses
- `ModMetrics`: Track per-mod statistics

### With plix-server

- `ModManager`: Orchestrate load/unload/dispatch
- Server tick loop: Call timer tick + event dispatch
- Shutdown: Call mod_shutdown for cleanup
