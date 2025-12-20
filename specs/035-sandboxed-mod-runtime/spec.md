# Feature Specification: Sandboxed Mod Runtime (WASM)

**Feature Branch**: `035-sandboxed-mod-runtime`
**Created**: 2025-12-18
**Status**: Draft
**Input**: Implement a sandboxed mod runtime based on WebAssembly (WASM) for executing mods safely, performantly, and controllably with full capability enforcement and integration with the Mod API Core (Feature 034).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Safe Mod Loading (Priority: P1)

As a server administrator, I can load WASM mod files knowing they are completely sandboxed with no access to filesystem, network, or OS resources, ensuring my server remains secure regardless of what mods are loaded.

**Why this priority**: Security is the foundational requirement - without proper sandboxing, the entire mod system poses unacceptable risks to server operators.

**Independent Test**: Load a test mod that attempts to access filesystem/network and verify it cannot. Load a valid mod and verify `mod_init` is called successfully.

**Acceptance Scenarios**:

1. **Given** a valid mod package (mod.toml + mod.wasm), **When** the server loads the mod, **Then** the WASM module is validated, instantiated in a sandbox, and `mod_init()` is called
2. **Given** a corrupted or invalid WASM file, **When** loading is attempted, **Then** loading fails with a clear error message and no server crash
3. **Given** a mod attempting filesystem access, **When** the mod executes, **Then** the access is denied with no capability to read/write any files
4. **Given** a mod attempting network access, **When** the mod executes, **Then** the access is denied with no capability for direct sockets/HTTP

---

### User Story 2 - Event Handling via Host ABI (Priority: P1)

As a mod developer, I can subscribe to game events and respond to them using the host ABI, calling World/Entity/Net/Timer APIs without needing unsafe code or direct engine access.

**Why this priority**: Core mod functionality depends on the ability to interact with game events and APIs. Without this, mods cannot provide any value.

**Independent Test**: Create a test mod that subscribes to PlayerChat event, reads the message via ABI, and optionally cancels it if the mod has the capability.

**Acceptance Scenarios**:

1. **Given** a mod that subscribes to PlayerChat, **When** a player sends a message, **Then** `mod_on_event(event_id, payload)` is called with the chat data
2. **Given** a mod with EVENT_CANCEL_CHAT capability, **When** it calls cancel on a chat event, **Then** the chat message is suppressed for other players
3. **Given** a mod calling `plix_world_call(get_block, ...)`, **When** it has WORLD_READ capability, **Then** it receives the block data at the specified position
4. **Given** a mod calling an API without the required capability, **When** the call is made, **Then** EMOD002 (PermissionDenied) is returned

---

### User Story 3 - CPU Budget Enforcement (Priority: P1)

As the game engine, I can automatically interrupt and penalize mods that exceed their CPU budget, preventing any single mod from degrading server performance.

**Why this priority**: DoS protection is critical for production servers. A single buggy or malicious mod with an infinite loop must not be able to crash or lag the server.

**Independent Test**: Load a test mod containing an infinite loop in its event handler and verify the server interrupts it within the budget limit.

**Acceptance Scenarios**:

1. **Given** a mod handler running for longer than the CPU budget (default 5ms), **When** the budget is exceeded, **Then** execution is interrupted (trap) and an error is recorded
2. **Given** a mod that exceeds budget 5 consecutive times, **When** the 5th violation occurs, **Then** the mod is automatically disabled
3. **Given** a mod that occasionally exceeds budget but succeeds in between, **When** it succeeds after a failure, **Then** the consecutive error counter resets

---

### User Story 4 - Memory Limits (Priority: P2)

As a server administrator, I can configure memory limits per mod to prevent any single mod from consuming excessive RAM.

**Why this priority**: Memory exhaustion is a critical DoS vector, but slightly less urgent than CPU since memory.grow is typically less common than compute loops.

**Independent Test**: Load a mod that aggressively calls memory.grow and verify it is trapped when exceeding the limit.

**Acceptance Scenarios**:

1. **Given** a mod configured with 32 MiB max memory, **When** it tries to grow beyond this limit, **Then** memory.grow fails and returns -1 (or traps)
2. **Given** memory exhaustion occurs, **When** the mod continues failing, **Then** it is disabled after consecutive error threshold

---

### User Story 5 - Observability and Logging (Priority: P2)

As a server administrator, I can see logs from mods with proper attribution and monitor mod performance metrics to identify problematic mods.

**Why this priority**: Debugging and performance monitoring are essential for operating a modded server, but not strictly required for basic functionality.

**Independent Test**: Have a mod call `plix_log` and verify the message appears in server logs with the mod ID.

**Acceptance Scenarios**:

1. **Given** a mod calling `plix_log(INFO, ptr, len)`, **When** the message is valid UTF-8, **Then** it appears in server logs prefixed with the mod ID
2. **Given** mod runtime metrics are enabled, **When** mods execute, **Then** CPU time, trap count, host call count, and permission denials are tracked per mod
3. **Given** debug mode is enabled, **When** mods run, **Then** verbose trace logs show all host function calls

---

### User Story 6 - Capability Discovery (Priority: P3)

As a mod developer, I can query at runtime which capabilities my mod has been granted, allowing me to gracefully degrade functionality when certain permissions are unavailable.

**Why this priority**: Nice-to-have for mod developer experience but not critical for basic mod operation.

**Independent Test**: Have a mod call `plix_has_capability(WORLD_WRITE)` and verify the return matches the manifest.

**Acceptance Scenarios**:

1. **Given** a mod granted WORLD_READ but not WORLD_WRITE, **When** it calls `plix_has_capability(WORLD_READ)`, **Then** it receives 1 (true)
2. **Given** the same mod, **When** it calls `plix_has_capability(WORLD_WRITE)`, **Then** it receives 0 (false)

---

### Edge Cases

- What happens when a mod's WASM imports functions the host doesn't provide?
  - Loading fails with a clear error listing missing imports
- What happens when a mod passes invalid pointers or lengths to host functions?
  - The host validates all pointer/length pairs and returns EMOD001 (InvalidArgument) if out of bounds
- What happens when mod_init() itself traps or times out?
  - The mod is not activated; loading fails with an error
- How does the system handle multiple mods with overlapping event subscriptions?
  - Events are dispatched to mods in FIFO order per Feature 034; each mod runs independently
- What happens if a mod is disabled mid-tick while other mods are processing?
  - The disabled mod's remaining handlers are skipped; already-processed handlers are not rolled back

## Requirements *(mandatory)*

### Functional Requirements

**WASM Loading & Validation**
- **FR-001**: System MUST load WASM modules from mod packages containing mod.toml and mod.wasm
- **FR-002**: System MUST validate WASM binary format before instantiation
- **FR-003**: System MUST reject mods that import functions not provided by the host
- **FR-004**: System MUST support a configurable entrypoint function name (default: `mod_main` or standard export convention)

**Sandbox Isolation**
- **FR-005**: System MUST NOT provide any WASI capabilities by default
- **FR-006**: System MUST NOT allow mods to access filesystem, network sockets, or OS processes
- **FR-007**: System MUST NOT allow mods to access uncontrolled time sources (no direct clock access)

**Host ABI (v1)**
- **FR-008**: System MUST expose `plix_get_api_version() -> i32` returning the API version
- **FR-009**: System MUST expose `plix_log(level: i32, ptr: i32, len: i32)` for mod logging
- **FR-010**: System MUST expose `plix_has_capability(cap_id: i32) -> i32` for capability discovery
- **FR-011**: System MUST expose `plix_subscribe_event(event_type: i32)` for event subscription
- **FR-012**: System MUST expose `plix_cancel_event() -> i32` to cancel the current cancellable event
- **FR-013**: System MUST expose `plix_world_call(op: i32, ptr: i32, len: i32) -> i32` for World API
- **FR-014**: System MUST expose `plix_entity_call(op: i32, ptr: i32, len: i32) -> i32` for Entity API
- **FR-015**: System MUST expose `plix_net_call(op: i32, ptr: i32, len: i32) -> i32` for Net API
- **FR-016**: System MUST expose `plix_timer_call(op: i32, ptr: i32, len: i32) -> i32` for Timer API
- **FR-017**: System MUST validate all pointer/length arguments against WASM linear memory bounds
- **FR-018**: System MUST check capability requirements before executing any host function
- **FR-019**: System MUST use a stable, versioned binary encoding for ABI payloads (abi_version = 1)

**Mod Lifecycle**
- **FR-020**: System MUST call `mod_init()` after successful WASM instantiation
- **FR-021**: System MUST call `mod_on_event(event_id: i32, ptr: i32, len: i32)` for dispatched events
- **FR-022**: System MUST call `mod_shutdown()` when unloading a mod
- **FR-023**: System MUST dispatch events to mods in FIFO order per Feature 034 specification

**CPU Budget Enforcement**
- **FR-024**: System MUST interrupt mod execution that exceeds the per-handler CPU budget (default: 5ms)
- **FR-025**: System MUST support configurable per-handler CPU budget
- **FR-026**: System MUST record budget violations as errors in the mod's consecutive error count
- **FR-027**: System MUST disable mods that exceed the violation threshold (default: 5 consecutive errors)

**Memory Limits**
- **FR-028**: System MUST limit each mod's linear memory to a configurable maximum (default: 32 MiB)
- **FR-029**: System MUST trap or fail memory.grow operations that would exceed the limit

**Error Handling**
- **FR-030**: System MUST serialize ModApiError (EMOD001-007) in ABI responses
- **FR-031**: System MUST NOT crash or panic due to mod traps or errors
- **FR-032**: System MUST increment consecutive error count on any handler failure
- **FR-033**: System MUST reset consecutive error count on successful handler completion

**Observability**
- **FR-034**: System MUST route `plix_log` calls to server logs with mod_id attribution
- **FR-035**: System MUST track per-mod metrics: CPU time, trap count, host calls, permission denials
- **FR-036**: System MUST support a debug mode with verbose host function tracing

### Key Entities

- **WasmModInstance**: Represents a loaded and instantiated WASM module with its memory, state, and capability grants
- **HostAbi**: The interface layer exposing engine functions to WASM mods with pointer validation and capability checks
- **RuntimeConfig**: Configuration for memory limits, CPU budgets, and violation thresholds
- **ModMetrics**: Aggregated performance and error metrics per mod instance

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Valid mods load and initialize within 100ms of load request
- **SC-002**: Mods with infinite loops are interrupted within 10ms of budget expiration
- **SC-003**: Server maintains 60 tick/s performance with up to 10 active mods under normal operation
- **SC-004**: Zero server crashes caused by mod failures during testing (traps are contained)
- **SC-005**: All host function calls complete with correct error codes within 1ms (excluding mod execution time)
- **SC-006**: Unauthorized API calls return EMOD002 100% of the time with no capability bypass
- **SC-007**: Mods exceeding memory limits are prevented from allocating additional memory 100% of the time
- **SC-008**: At least one example mod can be compiled to WASM and demonstrates event handling + API calls

## Assumptions

- Feature 034 (Mod API Core) is complete and provides the capability system, event types, and error codes
- The chosen WASM runtime supports fuel-based or epoch-based interruption for CPU budget enforcement
- WASM modules are compiled with wasm32-unknown-unknown target (no WASI)
- The server runs single-threaded mod dispatch (one mod handler at a time per tick)
- Binary encoding for ABI payloads will use bincode or postcard (decision at implementation time)

## Scope Boundaries

### In Scope
- WASM runtime integration with CPU/memory limits
- Host ABI v1 with capability enforcement
- Mod lifecycle (init, event, shutdown)
- Example mod with build instructions
- Basic observability (logging, metrics)

### Out of Scope
- Lua/JavaScript runtimes
- WASI filesystem/network access
- Hot reload of WASM modules
- JIT caching or AOT compilation
- Mod marketplace or signature verification
- Multi-threaded mod execution
- Client-side mod execution
