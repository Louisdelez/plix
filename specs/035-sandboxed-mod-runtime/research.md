# Research: Sandboxed Mod Runtime (WASM)

**Feature**: 035-sandboxed-mod-runtime
**Date**: 2025-12-18
**Status**: Complete

## Research Topics

### 1. WASM Runtime Selection

**Decision**: Wasmtime 27.x

**Rationale**:
- Bytecode Alliance project with strong security focus
- Robust fuel-based execution metering for CPU budgets
- Epoch-based interruption for cooperative multitasking
- First-class Rust support with typed API
- Active development with regular security updates
- Used by production systems (Fastly, Cloudflare Workers)

**Alternatives Considered**:

| Runtime | Pros | Cons | Rejected Because |
|---------|------|------|------------------|
| Wasmer | Multiple backends, WASI support | Less focus on fuel metering | Fuel interruption is critical for CPU budgets |
| wasm3 | Lightweight interpreter | No JIT, slower execution | Performance concern with 10 mods |
| lunatic | Erlang-style processes | Overkill for our use case | Too complex for MVP |

**Integration Notes**:
- Use `Engine` with fuel consumption enabled
- Configure `Store` with fuel limits per call
- Use `Linker` for host function registration
- Memory limits via `ResourceLimiter` trait

### 2. CPU Budget Enforcement Mechanism

**Decision**: Fuel-based metering with epoch interruption fallback

**Rationale**:
- Fuel counting provides instruction-level granularity
- Epoch interruption handles async/cooperative scenarios
- Both mechanisms work together in Wasmtime
- 5ms budget translates to approximately 50,000 fuel units (calibrated per-CPU)

**Implementation Pattern**:
```
1. Before mod_on_event(): store.set_fuel(FUEL_BUDGET)
2. Execute WASM function
3. Check fuel consumed vs elapsed time
4. If fuel exhausted -> OutOfFuel trap -> increment error count
5. Calibrate fuel-to-time ratio during startup
```

**Calibration Strategy**:
- Run calibration loop on startup (1ms of empty ops)
- Calculate fuel units per millisecond for current CPU
- Default: 10,000 fuel units ≈ 1ms on typical server CPU

### 3. Memory Limit Enforcement

**Decision**: Wasmtime ResourceLimiter + max_memory config

**Rationale**:
- Wasmtime's `ResourceLimiter` trait allows custom memory limits
- Memory.grow calls check against limiter before allocation
- Failed growth returns -1 (WASM spec) or traps if configured

**Implementation**:
- Implement `ResourceLimiter` for `ModInstance`
- Set `memory_growing` to return `false` when limit exceeded
- Default limit: 32 MiB (512 WASM pages of 64KB each)
- Configurable per-mod via RuntimeConfig

### 4. Host Function Binding Pattern

**Decision**: Wasmtime Linker with typed functions

**Rationale**:
- Type-safe function signatures at compile time
- Automatic argument/return value marshaling
- Access to Store state via Caller parameter

**Pattern**:
```rust
linker.func_wrap("plix", "log", |caller: Caller<ModState>, level: i32, ptr: i32, len: i32| -> i32 {
    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
    let data = read_memory(&caller, &mem, ptr as usize, len as usize)?;
    // ... process log message
    0 // success
})?;
```

### 5. ABI Payload Serialization

**Decision**: bincode with explicit versioning

**Rationale**:
- Already in workspace dependencies (Cargo.toml line 26)
- Compact binary format (smaller than JSON)
- Fast serialization/deserialization
- Works well with serde derive

**Format**:
```rust
#[derive(Serialize, Deserialize)]
struct AbiRequest {
    abi_version: u8,  // Always 1 for MVP
    op: u8,           // Operation code
    payload: Vec<u8>, // bincode-serialized operation data
}

#[derive(Serialize, Deserialize)]
struct AbiResponse {
    success: bool,
    error_code: Option<u16>,  // EMOD001-007
    payload: Vec<u8>,         // Result data or error message
}
```

### 6. Response Buffer Strategy

**Decision**: Host-owned buffer with fixed size

**Rationale**:
- Simpler than mod-allocated out-buffers
- Host controls memory, no risk of mod providing bad pointers
- Fixed 8KB buffer matches MAX_PAYLOAD_SIZE from plix-mod-core

**Implementation**:
- Host allocates response buffer in Store state
- `plix_response_ptr()` returns pointer to buffer start
- `plix_response_len()` returns actual response length after call
- Buffer cleared before each host call

### 7. Sandbox Policy (No WASI)

**Decision**: Pure sandbox with no WASI imports

**Rationale**:
- WASI provides filesystem, network, clock access
- Constitution requires complete isolation
- Mods use only plix-* host functions

**Enforcement**:
- Linker does not include any WASI functions
- If mod imports WASI functions, load fails with EMOD007 (Unsupported)
- Only "plix" namespace imports are recognized

### 8. Mod Exports Validation

**Decision**: Strict export validation at load time

**Required Exports**:
- `mod_init() -> i32`
- `mod_on_event(event_id: i32, ptr: i32, len: i32) -> i32`
- `mod_shutdown() -> i32`
- `memory` (linear memory, exported for host access)

**Optional Exports**:
- `plix_alloc(size: i32) -> i32` (for mod-side allocation, future use)
- `plix_free(ptr: i32, size: i32)` (for mod-side deallocation, future use)

**Validation**:
```rust
fn validate_exports(instance: &Instance) -> Result<(), RuntimeError> {
    instance.get_func("mod_init").ok_or(missing("mod_init"))?;
    instance.get_func("mod_on_event").ok_or(missing("mod_on_event"))?;
    instance.get_func("mod_shutdown").ok_or(missing("mod_shutdown"))?;
    instance.get_memory("memory").ok_or(missing("memory"))?;
    Ok(())
}
```

### 9. Error Handling Strategy

**Decision**: Non-panicking error propagation with auto-disable

**Flow**:
1. WASM trap occurs (OOB, fuel exhausted, etc.)
2. Trap caught by Wasmtime, converted to `anyhow::Error`
3. Runtime maps to appropriate EMOD error code
4. Error serialized to response buffer
5. Consecutive error count incremented in ModContext
6. If count >= 5, mod is disabled via ModRegistry

**Mapping Table** (from plan.md):
| Runtime Error | EMOD Code | Action |
|---------------|-----------|--------|
| Invalid WASM binary | EMOD001 | Reject load |
| Memory out of bounds | EMOD004 | Increment errors |
| Fuel exhausted | EMOD005 | Interrupt + increment |
| Memory limit exceeded | EMOD005 | Trap + increment |
| Missing required import | EMOD007 | Reject load |
| Missing required export | EMOD007 | Reject load |

### 10. Metrics Collection

**Decision**: Per-mod counters using existing ModMetrics from plix-mod-core

**Metrics**:
- `cpu_time_ms`: Total CPU time consumed (from fuel conversion)
- `trap_count`: Number of traps caught
- `host_call_count`: Total host function invocations
- `permission_denied_count`: EMOD002 returns
- `rate_limited_count`: EMOD005 returns (from net/timer limits)

**Collection Point**:
- After each host call: increment counters
- After each mod_on_event: accumulate CPU time
- On trap: increment trap_count

## Dependencies

### Wasmtime Crate Configuration

```toml
[dependencies]
wasmtime = { version = "27", default-features = false, features = [
    "cranelift",      # JIT compiler
    "runtime",        # Core runtime
    "parallel-compilation",  # Faster module compilation
] }
```

Note: Disable WASI features explicitly (not included = not linked).

### Test Mod Compilation

Test mods will be Rust libraries compiled to wasm32-unknown-unknown:

```toml
# tests/fixtures/chat_filter_mod/Cargo.toml
[package]
name = "chat_filter_mod"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
lto = true
opt-level = "s"
```

Compile with:
```bash
cargo build --target wasm32-unknown-unknown --release
```

## Open Questions (Resolved)

All research questions have been resolved. No outstanding unknowns.

## References

- [Wasmtime Book](https://docs.wasmtime.dev/)
- [Wasmtime Fuel Documentation](https://docs.wasmtime.dev/examples-rust-fuel.html)
- [Wasmtime ResourceLimiter](https://docs.rs/wasmtime/latest/wasmtime/trait.ResourceLimiter.html)
- [plix-mod-core (Feature 034)](../034-mod-api-core/)
