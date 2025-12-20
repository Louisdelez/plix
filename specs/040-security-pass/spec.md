# Feature Specification: Security Pass

**Feature Branch**: `040-security-pass`
**Created**: 2025-12-19
**Status**: Draft
**Input**: Production-grade security pass focusing on protocol fuzzing, parser hardening, and abuse case protections

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Runs Fuzz Tests (Priority: P1)

As a developer, I can launch fuzz testing on protocol decode functions and guarantee "no panic, no OOM" on any input, ensuring robust handling of malformed data.

**Why this priority**: Core security foundation - if decoders crash on malformed input, all other protections are moot. This directly prevents server crashes from malicious clients.

**Independent Test**: Can be fully tested by running fuzz harness for 5 minutes and observing zero panics/crashes across 1000+ fuzzing iterations.

**Acceptance Scenarios**:

1. **Given** a fuzz harness for ClientMessage decode, **When** I run the fuzzer for 5 minutes, **Then** zero panics or OOM errors occur and the process completes gracefully
2. **Given** random byte sequences of various lengths (0 to 64KB), **When** passed to message decode functions, **Then** all return typed errors without crashing
3. **Given** a corpus of valid messages, **When** bits are flipped randomly, **Then** decode either succeeds with valid data or returns a structured error

---

### User Story 2 - Server Admin Protected from Abuse (Priority: P1)

As a server administrator, I am protected against spam attacks on handshake, payload sync, and network messages, with automatic enforcement of rate limits and connection cleanup.

**Why this priority**: Equal priority with fuzzing - prevents denial of service attacks that could make servers unusable for legitimate players.

**Independent Test**: Can be tested by simulating abusive client patterns (rapid reconnects, spam messages) and verifying automatic disconnection occurs.

**Acceptance Scenarios**:

1. **Given** a client attempting rapid reconnection (>10 connects in 5 seconds), **When** the server processes these attempts, **Then** the client is rate-limited and eventually blocked for a cooldown period
2. **Given** a client in pending handshake state, **When** 10 seconds elapse without completion, **Then** the connection is automatically cleaned up
3. **Given** a client sending >200 messages/second, **When** the server receives this traffic, **Then** excess messages are dropped and strikes are accumulated leading to disconnect

---

### User Story 3 - Player Experience Unaffected by Malicious Clients (Priority: P2)

As a player, I do not experience server crashes, lag spikes, or disconnections caused by malicious clients sending crafted packets.

**Why this priority**: Relies on P1 protections being in place; represents the end-user outcome of security measures.

**Independent Test**: Can be tested by connecting legitimate clients while injecting malicious traffic from another client, measuring no impact on legitimate client experience.

**Acceptance Scenarios**:

1. **Given** legitimate players connected to a server, **When** a malicious client sends oversized packets (>64KB), **Then** the malicious client is disconnected and legitimate players experience no disruption
2. **Given** a player downloading mod payloads, **When** a malicious client attempts payload sync abuse (invalid chunks, endless transfers), **Then** only the malicious client's transfer is aborted
3. **Given** normal gameplay, **When** parser abuse is attempted (huge manifests, zip bombs), **Then** the abusive content is rejected without affecting server performance

---

### User Story 4 - Maintainer Prevents Regressions (Priority: P2)

As a maintainer, I can prevent security regressions through automated abuse tests in CI and documented fuzzing procedures.

**Why this priority**: Ensures P1/P2 protections remain effective over time; builds on existing implementations.

**Independent Test**: Can be tested by running the abuse test suite and verifying all tests pass with expected behavior documented.

**Acceptance Scenarios**:

1. **Given** the abuse test suite, **When** run in CI on every pull request, **Then** all tests pass and any new regression is caught before merge
2. **Given** documented fuzzing procedures, **When** a new developer follows the guide, **Then** they can successfully run fuzz targets within 15 minutes
3. **Given** security counters and logs, **When** reviewing server operation, **Then** I can identify attack patterns and tune protections

---

### Edge Cases

- What happens when a client sends a message exactly at the size limit (64KB)?
- How does the system handle messages with valid headers but corrupted payloads?
- What happens when a client disconnects mid-handshake timeout countdown?
- How are strikes handled when a client reconnects after being disconnected for violations?
- What happens when decompressed content exactly matches the zip ratio limit?
- How does the system handle unicode edge cases (overlong encodings, unpaired surrogates)?

## Requirements *(mandatory)*

### Functional Requirements

#### Threat Model & Limits

- **FR-001**: System MUST maintain a centralized limits module defining all security boundaries (max packet bytes, max string length, max list length, etc.)
- **FR-002**: System MUST document all untrusted inputs with their expected limits, error codes, and risk categories
- **FR-003**: System MUST define default limits: max_packet_bytes (64KB), max_string_bytes (1KB), max_list_len (256), max_cached_payload_hashes (256)

#### Protocol Hardening

- **FR-004**: System MUST validate all incoming messages against size limits before decode
- **FR-005**: System MUST never panic on decode of any input, returning typed errors instead
- **FR-006**: System MUST enforce per-message-type size limits for encoded messages
- **FR-007**: System MUST validate numeric bounds (positions, chunk indices) within reasonable ranges
- **FR-008**: System MUST implement a strike system: invalid message triggers strike, accumulating strikes leads to disconnect
- **FR-009**: System MUST disconnect clients after 3-5 accumulated strikes (configurable)

#### Fuzzing Infrastructure

- **FR-010**: System MUST provide at least 3 fuzz targets covering critical decode paths
- **FR-011**: System MUST include fuzz targets for: ClientMessage decode, ServerMessage decode, and at least one of (ModSetDescriptor, PayloadChunk, registry index, or mod manifest)
- **FR-012**: System MUST maintain an initial corpus of valid messages for seeding fuzz tests
- **FR-013**: Fuzz infrastructure MUST be feature-gated to avoid production impact

#### Handshake Abuse Protection

- **FR-014**: System MUST timeout pending handshakes after 10 seconds (configurable)
- **FR-015**: System MUST limit maximum pending connections per source
- **FR-016**: System MUST limit cached_payload_hashes list to 256 entries
- **FR-017**: System MUST clean up resources when handshake timeout triggers

#### Payload Sync Abuse Protection

- **FR-018**: System MUST reject payload chunks with invalid indices or out-of-order delivery beyond tolerance
- **FR-019**: System MUST limit resend requests per transfer window
- **FR-020**: System MUST timeout payload transfers after 30 seconds (configurable)
- **FR-021**: System MUST abort transfer and add strike on hash mismatch

#### Network Message Rate Limiting

- **FR-022**: System MUST enforce global per-client message rate limit (default: 200 msg/s)
- **FR-023**: System MUST apply token bucket rate limiting to mod channel messages
- **FR-024**: System MUST apply per-type rate limits for expensive message types

#### Parser Abuse Protection

- **FR-025**: System MUST limit registry index.json to 5MB maximum
- **FR-026**: System MUST limit mod.toml manifests to 256KB maximum
- **FR-027**: System MUST limit zip extraction to 10,000 files maximum
- **FR-028**: System MUST limit zip decompression ratio to 20:1 (or absolute byte cap)
- **FR-029**: System MUST reject zip entries with path traversal patterns (../, absolute paths)

#### Negative Tests

- **FR-030**: System MUST include deterministic tests for: truncated packets, random bytes, over-limit strings, over-limit lists
- **FR-031**: System MUST include tests for: pending handshake timeout cleanup, invalid payload chunk rejection, zip bomb blocking
- **FR-032**: System MUST include tests for: registry index size limits triggering appropriate error codes

#### Observability

- **FR-033**: System MUST expose counter metrics for: invalid_messages_total, disconnects_strikes_total, payload_sync_aborts_total, registry_parse_failures_total
- **FR-034**: System MUST implement rate-limited logging to prevent log-based DoS
- **FR-035**: System MUST support a debug mode for detailed security logging in development

### Key Entities

- **SecurityLimits**: Centralized configuration holding all size/count/rate limits with defaults and optional overrides
- **StrikeTracker**: Per-connection state tracking accumulated violations and determining disconnect threshold
- **RateLimiter**: Token bucket implementation for message rate limiting, supporting per-type and global limits
- **FuzzCorpus**: Collection of valid message samples used to seed fuzzing for better coverage

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero panics occur across 10,000+ fuzz iterations on all fuzz targets
- **SC-002**: All message decode functions return structured errors (never crash) for any malformed input
- **SC-003**: Clients sending >200 messages/second are automatically rate-limited within 1 second
- **SC-004**: Pending handshakes are cleaned up within 11 seconds of initiation (10s timeout + 1s grace)
- **SC-005**: Zip bombs (>20:1 compression ratio) are detected and rejected before full decompression
- **SC-006**: Path traversal attempts in zip files are rejected 100% of the time
- **SC-007**: Abuse test suite achieves 100% pass rate in CI with no flaky tests
- **SC-008**: New developers can run fuzz targets within 15 minutes following documentation
- **SC-009**: Security counters are queryable and accurately reflect blocked attacks
- **SC-010**: Server maintains stable performance (no p95 tick degradation >5%) when processing malformed traffic from malicious clients

## Scope

### In Scope

- Threat model and untrusted input inventory
- Centralized limits module
- Protocol decode hardening (no panics)
- Fuzz harness setup with 3+ targets
- Handshake abuse protections (timeouts, limits)
- Payload sync abuse protections (validation, timeouts)
- Network message rate limiting
- Parser abuse protections (size limits, zip bomb, path traversal)
- Deterministic negative test suite
- Security observability (counters, rate-limited logs)
- Documentation (threat model, limits, fuzzing guide, abuse cases)

### Out of Scope

- Advanced cryptography / TUF implementation
- IDS / anti-cheat systems
- Network-level WAF
- OS-level sandboxing (beyond existing WASM sandbox)
- Authentication/authorization changes
- New encryption protocols

## Assumptions

- Existing token bucket rate limiter from Feature 034 can be extended for broader use
- Bincode is used for binary protocol serialization
- WASM sandbox from Feature 035 already provides mod execution isolation
- Feature 036/037/038 structures (registry index, mod manifests, payload sync) exist and need hardening
- Performance validation through Feature 039 perf harness is available

## Dependencies

- **Feature 034**: Mod API Core (existing token bucket rate limiter)
- **Feature 035**: Sandboxed Mod Runtime (WASM isolation)
- **Feature 036**: Mod Distribution (registry, bundles, manifests)
- **Feature 037**: Handshake mods + payload sync
- **Feature 039**: Performance profiling (for validation)
