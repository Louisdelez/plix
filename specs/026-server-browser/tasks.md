# Tasks: Server Browser v1

**Input**: Design documents from `/specs/026-server-browser/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md structure:
- `crates/plix-master/src/` - New master server crate
- `crates/plix-common/src/server_browser/` - Shared types
- `crates/plix-server/src/master_announce/` - Game server heartbeat
- `crates/plix-client/src/server_browser/` - Client browser logic
- `crates/plix-client/src/console.rs` - Console commands

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create new crate and shared types

- [x] T001 Create plix-master crate with Cargo.toml in crates/plix-master/Cargo.toml
- [x] T002 Add workspace dependencies (axum, reqwest, tower) to Cargo.toml
- [x] T003 [P] Create server_browser module structure in crates/plix-common/src/server_browser/mod.rs
- [x] T004 [P] Define ServerEntry and ServerListResponse types in crates/plix-common/src/server_browser/types.rs
- [x] T005 [P] Add serde JSON roundtrip tests for ServerEntry in crates/plix-common/src/server_browser/types.rs
- [x] T006 Export server_browser module from crates/plix-common/src/lib.rs

---

## Phase 2: Foundational (Master Server Core)

**Purpose**: Core master server infrastructure that enables all user stories

**CRITICAL**: Master server must be operational before client/server integration

- [x] T007 Create main.rs with CLI args (bind_addr, ttl) in crates/plix-master/src/main.rs
- [x] T008 [P] Create lib.rs with core exports in crates/plix-master/src/lib.rs
- [x] T009 [P] Implement ServerRegistry state (HashMap + RwLock) in crates/plix-master/src/state.rs
- [x] T010 [P] Implement server_id generation (hash of host:port) in crates/plix-master/src/state.rs
- [x] T011 Add tests for server_id generation (same host:port => same id) in crates/plix-master/src/state.rs
- [x] T012 [P] Implement field validation (size/charset) in crates/plix-master/src/validation.rs
- [x] T013 Add validation tests (name > 64 chars rejected) in crates/plix-master/src/validation.rs
- [x] T014 [P] Implement rate limiting per IP in crates/plix-master/src/rate_limit.rs
- [x] T015 Add rate limit tests (11th request returns 429) in crates/plix-master/src/rate_limit.rs
- [x] T016 Define HeartbeatRequest and HeartbeatResponse types in crates/plix-master/src/types.rs
- [x] T017 Implement POST /heartbeat route with validation in crates/plix-master/src/api.rs
- [x] T018 Implement GET /servers route with TTL filtering in crates/plix-master/src/api.rs
- [x] T019 [P] Implement GET /health endpoint in crates/plix-master/src/api.rs
- [x] T020 Implement TTL expiration (lazy cleanup on GET) in crates/plix-master/src/state.rs
- [x] T021 Add TTL expiration test (expired entry not returned) in crates/plix-master/src/state.rs
- [x] T022 Add structured logging for heartbeat/list requests in crates/plix-master/src/api.rs
- [x] T023 Integration test: boot server and query /health in crates/plix-master/tests/integration_test.rs

**Checkpoint**: Master server operational - can receive heartbeats and list servers

---

## Phase 3: User Story 2 - Server Registration and Heartbeat (Priority: P1) 🎯 MVP

**Goal**: Game server announces itself to master server via periodic heartbeats

**Independent Test**: Start game server with master_url configured, verify it appears in GET /servers

### Implementation for User Story 2

- [x] T024 [P] [US2] Create master_announce module structure in crates/plix-server/src/master_announce/mod.rs
- [x] T025 [P] [US2] Define MasterConfig (url, name, region, tags, enabled) in crates/plix-server/src/master_announce/config.rs
- [x] T026 [US2] Add reqwest dependency to plix-server Cargo.toml
- [x] T027 [US2] Implement HeartbeatRequest payload builder in crates/plix-server/src/master_announce/heartbeat.rs
- [x] T028 [US2] Add test for heartbeat payload construction in crates/plix-server/src/master_announce/heartbeat.rs
- [x] T029 [US2] Implement async heartbeat task (20s interval) in crates/plix-server/src/master_announce/heartbeat.rs
- [x] T030 [US2] Handle HTTP errors gracefully (log warning, no crash) in crates/plix-server/src/master_announce/heartbeat.rs
- [x] T031 [US2] Add test: master down => server continues in crates/plix-server/tests/heartbeat_test.rs
- [x] T032 [US2] Integrate heartbeat start on server boot in crates/plix-server/src/main.rs
- [x] T033 [US2] Add CLI args for master config (--master-url, --server-name, --region) in crates/plix-server/src/main.rs
- [x] T034 [US2] Add logging for heartbeat success/failure in crates/plix-server/src/master_announce/heartbeat.rs

**Checkpoint**: Game server sends heartbeats; appears in master's /servers list

---

## Phase 4: User Story 1 - Browse and Connect to Server (Priority: P1) 🎯 MVP

**Goal**: Player can view server list and connect to a selected server

**Independent Test**: Run /servers, see list, /connect 1 connects to server

### Implementation for User Story 1

- [x] T035 [P] [US1] Create server_browser module structure in crates/plix-client/src/server_browser/mod.rs
- [x] T036 [P] [US1] Add reqwest dependency to plix-client Cargo.toml
- [x] T037 [US1] Implement HTTP client for GET /servers in crates/plix-client/src/server_browser/fetch.rs
- [x] T038 [US1] Add 5s timeout and error handling in crates/plix-client/src/server_browser/fetch.rs
- [x] T039 [US1] Add test: parse valid/invalid JSON response in crates/plix-client/src/server_browser/fetch.rs
- [x] T040 [US1] Implement BrowserState (cached server list, indices) in crates/plix-client/src/server_browser/mod.rs
- [x] T041 [US1] Add /servers command parsing in crates/plix-client/src/console.rs
- [x] T042 [US1] Implement server list display formatting in crates/plix-client/src/server_browser/mod.rs
- [x] T043 [US1] Add /connect <index> command parsing in crates/plix-client/src/console.rs
- [x] T044 [US1] Implement connection from ServerEntry (host:port) in crates/plix-client/src/server_browser/mod.rs
- [x] T045 [US1] Preserve display_name (Feature 025) on connect in crates/plix-client/src/server_browser/mod.rs
- [x] T046 [US1] Add error handling (timeout, offline, incompatible version) in crates/plix-client/src/server_browser/mod.rs
- [x] T047 [US1] Add test: /connect 1 uses correct host:port in crates/plix-client/src/server_browser/mod.rs
- [x] T048 [US1] Add logging for refresh/connect attempts in crates/plix-client/src/server_browser/mod.rs
- [x] T049 [US1] Integrate server browser with client main loop in crates/plix-client/src/main.rs

**Checkpoint**: Player can browse servers and connect via pause menu (ESC > Servers)

---

## Phase 5: User Story 3 - Search and Filter Servers (Priority: P2)

**Goal**: Player can search and filter the server list

**Independent Test**: With multiple servers, /servers ctf shows only matching servers

### Implementation for User Story 3

- [ ] T050 [P] [US3] Implement substring search (name, tags, region) in crates/plix-client/src/server_browser/filter.rs
- [ ] T051 [P] [US3] Implement has_players filter (player_count > 0) in crates/plix-client/src/server_browser/filter.rs
- [ ] T052 [P] [US3] Implement compatible_version filter in crates/plix-client/src/server_browser/filter.rs
- [ ] T053 [US3] Add /servers <search> command variant in crates/plix-client/src/console.rs
- [ ] T054 [US3] Add --players and --compatible flags to /servers in crates/plix-client/src/console.rs
- [ ] T055 [US3] Add tests for each filter type in crates/plix-client/src/server_browser/filter.rs

**Checkpoint**: Search and filter functionality works on server list

---

## Phase 6: User Story 4 - Manage Favorite Servers (Priority: P2)

**Goal**: Player can save and view favorite servers

**Independent Test**: /favorite 1, exit client, restart, /favorites shows the server

### Implementation for User Story 4

- [ ] T056 [P] [US4] Define FavoriteServer and FavoritesConfig types in crates/plix-client/src/server_browser/favorites.rs
- [ ] T057 [US4] Implement TOML save/load for ~/.config/plix/servers.toml in crates/plix-client/src/server_browser/favorites.rs
- [ ] T058 [US4] Add test: save/load roundtrip in crates/plix-client/src/server_browser/favorites.rs
- [ ] T059 [US4] Add /favorite <index> command in crates/plix-client/src/console.rs
- [ ] T060 [US4] Add /unfavorite <index> command in crates/plix-client/src/console.rs
- [ ] T061 [US4] Add /favorites command (show list with online status) in crates/plix-client/src/console.rs
- [ ] T062 [US4] Handle corrupted favorites file (fallback to empty) in crates/plix-client/src/server_browser/favorites.rs

**Checkpoint**: Favorites persist across client restarts

---

## Phase 7: User Story 5 - Sort Server List (Priority: P3)

**Goal**: Player can sort servers by player count or recency

**Independent Test**: /servers --sort=players shows busiest servers first

### Implementation for User Story 5

- [ ] T063 [P] [US5] Implement sort by player_count desc in crates/plix-client/src/server_browser/filter.rs
- [ ] T064 [P] [US5] Implement sort by last_seen desc in crates/plix-client/src/server_browser/filter.rs
- [ ] T065 [US5] Add --sort=players|recent flag to /servers in crates/plix-client/src/console.rs
- [ ] T066 [US5] Add sorting tests in crates/plix-client/src/server_browser/filter.rs

**Checkpoint**: Sorting options work correctly

---

## Phase 8: User Story 6 - Server Ping Display (Priority: P3) - Optional

**Goal**: Display ping to each server (or "?" if not implemented)

**Independent Test**: /servers shows ping column with ms values or "?"

### Implementation for User Story 6

- [ ] T067 [US6] Display "ping: ?" placeholder for all servers in crates/plix-client/src/server_browser/mod.rs
- [ ] T068 [US6] (Optional) Implement async ping measurement in crates/plix-client/src/server_browser/ping.rs
- [ ] T069 [US6] (Optional) Add ping budget (max 10 concurrent) in crates/plix-client/src/server_browser/ping.rs

**Checkpoint**: Ping column displays (even if just "?")

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Security, testing, documentation

### Security & Sanitization

- [ ] T070 [P] Implement string sanitization (truncate, safe chars) in crates/plix-client/src/server_browser/mod.rs
- [ ] T071 Add test: oversized fields truncated in crates/plix-client/src/server_browser/mod.rs

### Integration Tests

- [ ] T072 E2E test: master + server + client flow in crates/plix-master/tests/e2e_test.rs
- [ ] T073 Test: server heartbeat -> master lists -> client sees in crates/plix-master/tests/e2e_test.rs
- [ ] T074 Test: server stops -> TTL expires -> client doesn't see in crates/plix-master/tests/e2e_test.rs

### Non-Regression

- [ ] T075 [P] Verify direct connect (without master) still works in crates/plix-client/tests/direct_connect_test.rs
- [ ] T076 [P] Verify server runs without advertise_enabled in crates/plix-server/tests/no_advertise_test.rs

### Documentation

- [ ] T077 [P] Update /help command with server browser commands in crates/plix-client/src/console.rs
- [ ] T078 Run quickstart.md validation scenarios

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3 (US2 - Heartbeat)**: Depends on Phase 2 (needs master server)
- **Phase 4 (US1 - Browse)**: Depends on Phase 2 (needs master server) and benefits from Phase 3
- **Phase 5-8 (US3-6)**: Depend on Phase 4 (need basic browser working)
- **Phase 9 (Polish)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US2 (Heartbeat)**: Independent - only needs master server
- **US1 (Browse/Connect)**: Independent - only needs master server; US2 provides data
- **US3 (Search/Filter)**: Depends on US1 (needs browser state)
- **US4 (Favorites)**: Depends on US1 (needs server list)
- **US5 (Sort)**: Depends on US1 (needs server list)
- **US6 (Ping)**: Depends on US1 (needs server entries)

### Parallel Opportunities

Within Phase 1:
- T003, T004, T005 can run in parallel (different files)

Within Phase 2:
- T009, T010, T012, T014, T019 can run in parallel (different files)

Within Phase 3 (US2):
- T024, T025 can run in parallel

Within Phase 4 (US1):
- T035, T036 can run in parallel

Within Phase 5 (US3):
- T050, T051, T052 can run in parallel (different functions in same file)

---

## Parallel Example: Phase 2 (Foundational)

```bash
# Launch parallel tasks for master server core:
Task: "Implement ServerRegistry state in crates/plix-master/src/state.rs"
Task: "Implement field validation in crates/plix-master/src/validation.rs"
Task: "Implement rate limiting in crates/plix-master/src/rate_limit.rs"
Task: "Implement GET /health endpoint in crates/plix-master/src/api.rs"
```

## Parallel Example: User Story 1 (Browse)

```bash
# Launch parallel model/structure tasks:
Task: "Create server_browser module in crates/plix-client/src/server_browser/mod.rs"
Task: "Add reqwest dependency to plix-client Cargo.toml"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (master server core)
3. Complete Phase 3: US2 (server registration/heartbeat)
4. Complete Phase 4: US1 (browse and connect)
5. **STOP and VALIDATE**: Test E2E flow
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Master server operational
2. Add US2 (Heartbeat) → Servers can register
3. Add US1 (Browse/Connect) → **MVP Complete!**
4. Add US3 (Search/Filter) → Better discoverability
5. Add US4 (Favorites) → Repeat player experience
6. Add US5 (Sort) → UX polish
7. Add US6 (Ping) → Optional QoL

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- US2 (Heartbeat) is listed before US1 (Browse) because it provides the data, but both are P1
- Ping (US6) is optional - "?" placeholder is acceptable for v1
- All master server work is in Phase 2 because it's foundational
- Direct connect (without master) must continue to work
