# Data Model: Sandboxed Mod Runtime (WASM)

**Feature**: 035-sandboxed-mod-runtime
**Date**: 2025-12-18

## Core Entities

### WasmRuntime

The top-level runtime managing all WASM mod instances.

```
WasmRuntime
├── engine: WasmEngine           # Shared Wasmtime engine
├── config: RuntimeConfig        # Global configuration
├── instances: Map<ModId, ModInstance>  # Active mod instances
└── fuel_calibration: u64        # Fuel units per millisecond
```

**Responsibilities**:
- Initialize Wasmtime engine with security settings
- Load/unload mod instances
- Dispatch events to appropriate instances
- Manage fuel calibration

### WasmEngine

Wrapper around Wasmtime Engine with plix-specific configuration.

```
WasmEngine
├── inner: wasmtime::Engine      # Underlying engine
├── linker: wasmtime::Linker     # Host function bindings
└── epoch_deadline: Duration     # Epoch-based timeout
```

**Configuration**:
- Fuel consumption: enabled
- Epoch interruption: enabled
- WASI: disabled
- Debug info: enabled in debug builds only

### ModInstance

A loaded and instantiated WASM module for a single mod.

```
ModInstance
├── mod_id: String               # Unique identifier
├── store: wasmtime::Store<ModState>  # WASM store with state
├── instance: wasmtime::Instance # Instantiated module
├── memory: wasmtime::Memory     # Linear memory reference
├── exports: ModExports          # Cached function references
├── metrics: ModMetrics          # Performance counters
└── response_buffer: [u8; 8192]  # Host response buffer
```

**Lifecycle**:
1. Created: WASM loaded and validated
2. Initialized: mod_init() called successfully
3. Active: Receiving events, calling host functions
4. Disabled: Too many errors, no longer receiving events
5. Shutdown: mod_shutdown() called, resources released

### ModState

Per-mod state stored in Wasmtime Store.

```
ModState
├── mod_id: String               # For logging/metrics
├── capabilities: Capability     # Effective capabilities (from 034)
├── event_context: EventContext  # Current event being processed
├── fuel_budget: u64             # Fuel units for current call
├── memory_limit: usize          # Max memory in bytes
├── response_ptr: usize          # Response buffer start
├── response_len: usize          # Response data length
└── metrics: ModMetrics          # Accumulated metrics
```

### ModExports

Cached references to required WASM exports.

```
ModExports
├── mod_init: TypedFunc<(), i32>
├── mod_on_event: TypedFunc<(i32, i32, i32), i32>
├── mod_shutdown: TypedFunc<(), i32>
└── memory: Memory
```

### RuntimeConfig

Configuration for the WASM runtime.

```
RuntimeConfig
├── handler_cpu_budget_ms: u64   # Default: 5
├── mod_tick_budget_ms: u64      # Default: 10
├── max_memory_bytes: usize      # Default: 32 * 1024 * 1024 (32 MiB)
├── violation_threshold: u32     # Default: 5 (from 034)
├── debug_mode: bool             # Default: false
└── fuel_per_ms: u64             # Calibrated at startup
```

### ModMetrics

Performance metrics per mod instance (extends 034's ModMetrics).

```
ModMetrics
├── cpu_time_ms: f64             # Total CPU time consumed
├── trap_count: u32              # Traps caught
├── host_call_count: u64         # Total host function calls
├── permission_denied_count: u32 # EMOD002 returns
├── rate_limited_count: u32      # EMOD005 returns
├── last_call_fuel: u64          # Fuel used in last call
└── peak_memory_bytes: usize     # Peak memory usage
```

## ABI Types

### AbiVersion

```
AbiVersion = 1 (constant)
```

### HostCallOp

Operation codes for host function calls.

```
enum HostCallOp: u8 {
    // Logging
    Log = 0x01,

    // Capabilities
    HasCapability = 0x10,
    GetApiVersion = 0x11,
    GetAbiVersion = 0x12,

    // Events
    SubscribeEvent = 0x20,
    CancelEvent = 0x21,

    // World API
    WorldGetBlock = 0x30,
    WorldSetBlock = 0x31,
    WorldRaycast = 0x32,
    WorldQueryAabb = 0x33,

    // Entity API
    EntityGetTransform = 0x40,
    EntityGetHealth = 0x41,
    EntityApplyDamage = 0x42,
    EntityApplyImpulse = 0x43,

    // Net API
    NetSend = 0x50,
    NetBroadcast = 0x51,

    // Timer API
    TimerSetTimeout = 0x60,
    TimerSetInterval = 0x61,
    TimerClear = 0x62,
}
```

### AbiRequest

Request structure for API calls.

```
AbiRequest
├── op: HostCallOp               # Operation code
└── payload: bytes               # bincode-serialized arguments
```

### AbiResponse

Response structure from host calls.

```
AbiResponse
├── success: bool                # true if operation succeeded
├── error_code: Option<u16>      # EMOD001-007 if failed
├── error_message: Option<String> # Human-readable error
└── payload: bytes               # bincode-serialized result
```

### LogLevel

```
enum LogLevel: i32 {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}
```

### EventPayload

Serialized event data passed to mod_on_event.

```
EventPayload
├── event_type: u8               # EventType enum value
├── tick: u64                    # Server tick when event occurred
├── data: bytes                  # Event-specific data (bincode)
└── cancellable: bool            # Whether event can be cancelled
```

## Relationships

```
WasmRuntime --owns--> WasmEngine
WasmRuntime --contains--> ModInstance (1:N)

ModInstance --has--> ModState
ModInstance --has--> ModExports
ModInstance --has--> ModMetrics

ModState --references--> Capability (from 034)
ModState --references--> EventContext (from 034)

ModInstance --loads-from--> ModManifest (from 034)
ModInstance --registered-in--> ModRegistry (from 034)
```

## State Transitions

### ModInstance Lifecycle

```
[Created] --validate_exports()--> [Validated]
[Validated] --mod_init()--> [Active]
[Active] --error_count >= 5--> [Disabled]
[Active] --unload_request--> [Shutting Down]
[Shutting Down] --mod_shutdown()--> [Terminated]
[Disabled] --admin_enable()--> [Active]
```

### Event Dispatch Flow

```
1. EventBus.dispatch() triggers for subscribed event
2. WasmRuntime.dispatch_event(mod_id, event)
3. ModInstance.prepare_event(payload)
   - Serialize EventPayload to mod memory
   - Set fuel budget
4. ModInstance.call_on_event(ptr, len)
   - Execute mod_on_event()
   - May trap on fuel/memory exhaustion
5. ModInstance.handle_result(result)
   - Success: reset error count
   - Trap: increment error count
   - Check disable threshold
```

## Validation Rules

### WASM Module Validation

1. Binary format must be valid WebAssembly
2. All imports must be in "plix" namespace
3. Required exports must exist with correct signatures
4. Memory must be exported as "memory"

### Runtime Invariants

1. Fuel is always set before calling mod functions
2. Response buffer is cleared before each host call
3. Memory bounds are checked before all pointer access
4. Capabilities are checked before all API operations
5. Metrics are updated after every host call
