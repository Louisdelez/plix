# Feature Specification: Logging & Metrics

**Feature Branch**: `010-logging-metrics`
**Created**: 2025-12-15
**Status**: Draft
**Input**: User description: "Server-side metrics (tick time, RTT, jitter, packet loss) and client-side network debug overlay"

## Clarifications

### Session 2025-12-15

- Q: How should RTT be measured - dedicated ping/pong, extended input messages, or sequence acknowledgments? → A: Extend existing client input messages with echo timestamp
- Q: What rolling window size for network metrics (RTT, jitter, packet loss)? → A: Same 10-second window as tick metrics

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Server Tick-Time Metrics (Priority: P1)

As a server operator, I need to monitor tick processing time so I can detect performance degradation before it affects gameplay.

**Why this priority**: Tick time is the fundamental server health metric. If tick time exceeds 16.67ms (60Hz target), the game simulation degrades. This is the most critical metric for server operators.

**Independent Test**: Start server, run load test with 8 clients, verify tick time metrics are logged every 5 seconds showing avg, p95, and max over a 10-second rolling window.

**Acceptance Scenarios**:

1. **Given** a running server, **When** I check logs every 5 seconds, **Then** I see tick_time_ms_avg, tick_time_ms_p95, tick_time_ms_max values computed over the last 10 seconds
2. **Given** normal load, **When** tick time avg stays below 16.67ms, **Then** no warning is emitted
3. **Given** high load causing tick time to exceed threshold, **When** tick_time_ms_avg exceeds 16.67ms, **Then** a warning is logged

---

### User Story 2 - Per-Connection Network Metrics (Priority: P1)

As a server operator, I need to monitor per-connection network quality (RTT, jitter, packet loss) so I can identify players with connectivity issues.

**Why this priority**: Network quality directly affects player experience. Per-connection metrics allow operators to identify problematic connections and help players troubleshoot.

**Independent Test**: Connect 2 clients, verify each connection has its own RTT, jitter, and loss_pct metrics computed and logged.

**Acceptance Scenarios**:

1. **Given** an established connection, **When** I check server logs, **Then** I see rtt_ms, jitter_ms, loss_pct for each player
2. **Given** a stable connection, **When** RTT is measured, **Then** jitter is computed as the standard deviation of RTT samples over the window
3. **Given** packet sequence tracking, **When** gaps are detected in sequence numbers, **Then** loss_pct is computed as (gaps / expected) * 100
4. **Given** a player disconnects, **When** session ends, **Then** their metrics are cleaned up

---

### User Story 3 - Server Aggregate Metrics (Priority: P2)

As a server operator, I need aggregate metrics (PPS in/out, player count, session count) so I can understand overall server load.

**Why this priority**: Aggregate metrics provide the big picture view. While less critical than per-tick and per-connection health, they help with capacity planning and load monitoring.

**Independent Test**: Run server with multiple clients, verify aggregate PPS_in, PPS_out, players_connected, sessions_active are logged.

**Acceptance Scenarios**:

1. **Given** a running server, **When** I check logs, **Then** I see pps_in and pps_out (packets per second over window)
2. **Given** players connecting/disconnecting, **When** I check logs, **Then** players_connected reflects current count
3. **Given** active matches, **When** I check logs, **Then** sessions_active reflects current match count

---

### User Story 4 - Client Network Debug Overlay (Priority: P2)

As a player, I need to press F3 to see a network debug overlay so I can diagnose connection issues.

**Why this priority**: Client-side visibility helps players self-diagnose issues. This reduces support burden and improves player experience during network problems.

**Independent Test**: Connect client, press F3, verify overlay shows RTT, jitter, loss%, local FPS. Press F3 again to hide.

**Acceptance Scenarios**:

1. **Given** a connected client with overlay disabled, **When** I press F3, **Then** the overlay appears showing network stats
2. **Given** the overlay is visible, **When** I press F3 again, **Then** the overlay hides
3. **Given** the overlay is visible, **When** 500ms elapses, **Then** the displayed values update (2Hz refresh)
4. **Given** the overlay is visible, **When** I look at it, **Then** I see RTT (ms), Jitter (ms), Loss (%), FPS

---

### Edge Cases

- **What happens when no packets received for extended period?** RTT/jitter show last known value, loss_pct increases toward 100%
- **What happens when rolling window has insufficient samples?** Use available samples; if zero samples, display "N/A" or 0
- **What happens when client disconnects during overlay update?** Gracefully handle missing data, show "Disconnected" state
- **What happens in headless mode?** Overlay code path is skipped entirely, only server metrics are collected
- **What happens during high packet loss?** Sequence gap detection continues working; RTT uses only acknowledged packets

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Server MUST measure tick processing time using std::time::Instant at start and end of each tick
- **FR-002**: Server MUST maintain a 10-second rolling window of tick time samples (600 samples at 60Hz)
- **FR-003**: Server MUST log tick metrics (avg, p95, max) every 5 seconds
- **FR-004**: Server MUST compute RTT per connection by echoing a client timestamp field added to existing input messages
- **FR-005**: Server MUST compute jitter as standard deviation of RTT samples in a 10-second rolling window
- **FR-006**: Server MUST compute packet loss percentage using sequence number gaps over a 10-second rolling window
- **FR-007**: Server MUST track PPS (packets per second) for inbound and outbound traffic per connection
- **FR-008**: Server MUST aggregate PPS across all connections for server-wide metrics
- **FR-009**: Client MUST toggle debug overlay on F3 key press
- **FR-010**: Client MUST update overlay at 2Hz (every 500ms) when visible
- **FR-011**: Client MUST display RTT, jitter, loss%, and local FPS in overlay
- **FR-012**: Overlay MUST NOT interfere with gameplay (render on top, minimal CPU/GPU impact)
- **FR-013**: Metrics system MUST NOT allocate during tick processing (pre-allocate buffers)
- **FR-014**: Headless server mode MUST work without client overlay code
- **FR-015**: Load tests MUST continue working with metrics enabled

### Key Entities

- **ServerMetrics**: Aggregate server health - tick_time rolling window, player count, session count, aggregate PPS
- **SessionNetMetrics**: Per-connection network quality - RTT samples (10s window), jitter, loss tracking (10s window), per-connection PPS, bandwidth
- **ClientOverlayState**: Client-side overlay state - enabled flag, cached display values, last update timestamp

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Tick time measurement overhead < 1% of tick budget (< 0.167ms per tick)
- **SC-002**: Rolling window statistics computed in O(1) amortized time
- **SC-003**: No heap allocations during steady-state tick processing
- **SC-004**: Overlay renders in < 1ms (GPU time)
- **SC-005**: Overlay toggle responds within 1 frame (< 16.67ms)
- **SC-006**: All existing tests pass with metrics enabled
- **SC-007**: Load test with 8 clients for 30 seconds shows no performance regression
- **SC-008**: Headless server boots and runs without client dependencies
