# Tasks: Matchmaking v1 (Quick Join)

**Input**: Design documents from `/specs/027-matchmaking-v1/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md structure:
- `crates/plix-client/src/matchmaking/` - New matchmaking module
- `crates/plix-client/src/profile/` - Existing profile module (Feature 025)
- `crates/plix-client/src/server_browser/` - Existing server browser (Feature 026)
- `crates/plix-client/src/console.rs` - Console commands
- `crates/plix-client/src/main.rs` - Game loop integration

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create matchmaking module structure and types

- [x] T001 Create matchmaking module directory structure in crates/plix-client/src/matchmaking/mod.rs
- [x] T002 [P] Define QuickJoinRequest struct with mode/region fields in crates/plix-client/src/matchmaking/request.rs
- [x] T003 [P] Define MatchmakingPreferences struct in crates/plix-client/src/matchmaking/preferences.rs
- [x] T004 [P] Define ServerScore and ScoreBreakdown structs in crates/plix-client/src/matchmaking/scoring.rs
- [x] T005 [P] Define QuickJoinResult struct in crates/plix-client/src/matchmaking/request.rs
- [x] T006 Add serde serialization tests for MatchmakingPreferences in crates/plix-client/src/matchmaking/preferences.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Profile extension and server filtering infrastructure

**CRITICAL**: Must complete before any user story implementation

- [x] T007 Extend PlayerProfile with MatchmakingPreferences field in crates/plix-client/src/profile/player_profile.rs
- [x] T008 Add save/load roundtrip test for profile with matchmaking section in crates/plix-client/src/profile/player_profile.rs
- [x] T009 [P] Implement mandatory filters (protocol version, full servers) in crates/plix-client/src/matchmaking/filtering.rs
- [x] T010 Add unit tests for mandatory filtering (version mismatch, full server) in crates/plix-client/src/matchmaking/filtering.rs
- [x] T011 [P] Implement mode filter with "any" support in crates/plix-client/src/matchmaking/filtering.rs
- [x] T012 Add unit tests for mode filtering (exact match, any mode) in crates/plix-client/src/matchmaking/filtering.rs
- [x] T013 [P] Implement region filter with "any" support in crates/plix-client/src/matchmaking/filtering.rs
- [x] T014 Add unit tests for region filtering (exact match, any region) in crates/plix-client/src/matchmaking/filtering.rs
- [x] T015 Export matchmaking module from crates/plix-client/src/lib.rs

**Checkpoint**: Filtering infrastructure ready - user story implementation can begin

---

## Phase 3: User Story 1 - Quick Join with Mode and Region (Priority: P1) - MVP

**Goal**: Player can issue `/quickjoin <mode> <region>` and connect to a matching server

**Independent Test**: Issue `/quickjoin tdm eu` with multiple servers, verify connection to EU TDM server

### Implementation for User Story 1

- [x] T016 [US1] Implement /quickjoin command parsing in crates/plix-client/src/console.rs
- [x] T017 [US1] Implement /play alias command in crates/plix-client/src/console.rs
- [x] T018 [US1] Add command validation (valid mode/region values) in crates/plix-client/src/console.rs
- [x] T019 [US1] Add test for command parsing with various inputs in crates/plix-client/src/console.rs
- [x] T020 [US1] Implement filter_servers function combining all filters in crates/plix-client/src/matchmaking/filtering.rs
- [x] T021 [US1] Add logging for quick join request (mode, region) in crates/plix-client/src/matchmaking/request.rs
- [x] T022 [US1] Add logging for candidate server count after filtering in crates/plix-client/src/matchmaking/filtering.rs
- [ ] T023 [US1] Integrate quick join trigger in game loop in crates/plix-client/src/main.rs
- [ ] T024 [US1] Preserve display_name when connecting via quick join in crates/plix-client/src/matchmaking/request.rs

**Checkpoint**: Basic quick join works - player can connect to servers by mode/region

---

## Phase 4: User Story 2 - Intelligent Server Selection (Priority: P1) - MVP

**Goal**: System selects best server using scoring algorithm

**Independent Test**: Register servers with varying player counts and verify scoring prefers partially-filled servers

### Implementation for User Story 2

- [x] T025 [P] [US2] Implement calculate_score function in crates/plix-client/src/matchmaking/scoring.rs
- [x] T026 [US2] Implement region bonus (+50 points) in scoring in crates/plix-client/src/matchmaking/scoring.rs
- [x] T027 [US2] Implement capacity bonus (+30 for 1-80% full) in scoring in crates/plix-client/src/matchmaking/scoring.rs
- [x] T028 [US2] Implement freshness bonus (+20 if last_seen < 30s) in scoring in crates/plix-client/src/matchmaking/scoring.rs
- [x] T029 [US2] Implement player bonus (+1 per player up to 80% cap) in scoring in crates/plix-client/src/matchmaking/scoring.rs
- [x] T030 [US2] Implement optional ping bonus (+40/<50ms, +20/<100ms) in scoring in crates/plix-client/src/matchmaking/scoring.rs
- [x] T031 [US2] Add unit tests for each scoring component in crates/plix-client/src/matchmaking/scoring.rs
- [x] T032 [US2] Implement score_servers function (returns sorted list) in crates/plix-client/src/matchmaking/scoring.rs
- [x] T033 [US2] Add test for score sorting (descending order) in crates/plix-client/src/matchmaking/scoring.rs
- [x] T034 [P] [US2] Implement select_best_server with random tie-breaking in crates/plix-client/src/matchmaking/selection.rs
- [x] T035 [US2] Add test for tie-breaking (multiple servers with same score) in crates/plix-client/src/matchmaking/selection.rs
- [x] T036 [US2] Add logging for selected server (name, host:port, score) in crates/plix-client/src/matchmaking/selection.rs

**Checkpoint**: Intelligent selection works - best server chosen by scoring algorithm

---

## Phase 5: User Story 3 - Fallback When No Exact Match (Priority: P2)

**Goal**: System finds server via fallback when no exact match exists

**Independent Test**: Request mode/region with no exact match, verify fallback to any region then any mode

### Implementation for User Story 3

- [x] T037 [US3] Implement select_server with fallback cascade in crates/plix-client/src/matchmaking/request.rs
- [x] T038 [US3] Implement fallback step 1: expand region to "any" in crates/plix-client/src/matchmaking/request.rs
- [x] T039 [US3] Implement fallback step 2: expand mode to "any" in crates/plix-client/src/matchmaking/request.rs
- [x] T040 [US3] Set fallback_used and fallback_reason in QuickJoinResult in crates/plix-client/src/matchmaking/request.rs
- [x] T041 [US3] Display "No servers available" when all fallbacks fail in crates/plix-client/src/matchmaking/request.rs
- [ ] T042 [US3] Add feedback message when fallback was used in crates/plix-client/src/main.rs
- [x] T043 [US3] Add unit tests for fallback cascade (region, then mode) in crates/plix-client/src/matchmaking/request.rs
- [x] T044 [US3] Add test for empty server list handling in crates/plix-client/src/matchmaking/request.rs

**Checkpoint**: Fallback works - players always find a server if any exist

---

## Phase 6: User Story 4 - Preferences Persistence (Priority: P2)

**Goal**: Player preferences persist across sessions

**Independent Test**: Set preference, restart client, verify preference loaded

### Implementation for User Story 4

- [x] T045 [US4] Implement load_preferences helper in crates/plix-client/src/matchmaking/preferences.rs
- [x] T046 [US4] Implement save_preferences helper in crates/plix-client/src/matchmaking/preferences.rs
- [ ] T047 [US4] Load preferences on client startup in crates/plix-client/src/main.rs
- [ ] T048 [US4] Save preferences after successful quick join in crates/plix-client/src/main.rs
- [x] T049 [US4] Implement /quickjoin-prefs command (view) in crates/plix-client/src/console.rs
- [x] T050 [US4] Implement /quickjoin-prefs mode <value> command in crates/plix-client/src/console.rs
- [x] T051 [US4] Implement /quickjoin-prefs region <value> command in crates/plix-client/src/console.rs
- [ ] T052 [US4] Apply saved preferences as defaults in /quickjoin in crates/plix-client/src/console.rs
- [x] T053 [US4] Add test for preference persistence roundtrip in crates/plix-client/src/matchmaking/preferences.rs

**Checkpoint**: Preferences persist - returning players have saved defaults

---

## Phase 7: User Story 5 - Connection Error Handling (Priority: P2)

**Goal**: Clear error feedback and auto-retry on connection failure

**Independent Test**: Simulate connection failure, verify retry attempts and error messages

### Implementation for User Story 5

- [x] T054 [P] [US5] Define RetryState struct with failed_servers HashSet in crates/plix-client/src/matchmaking/retry.rs
- [x] T055 [US5] Implement mark_failed and is_failed methods in crates/plix-client/src/matchmaking/retry.rs
- [x] T056 [US5] Implement can_retry (attempts < 3) check in crates/plix-client/src/matchmaking/retry.rs
- [ ] T057 [US5] Integrate retry loop in quick join flow in crates/plix-client/src/main.rs
- [x] T058 [US5] Exclude failed servers from next selection in crates/plix-client/src/matchmaking/selection.rs
- [ ] T059 [US5] Display "Connection timed out" error message in crates/plix-client/src/main.rs
- [ ] T060 [US5] Display "Server is full" error message in crates/plix-client/src/main.rs
- [ ] T061 [US5] Display "Incompatible version" error message in crates/plix-client/src/main.rs
- [ ] T062 [US5] Display retry attempt count (X/3) in crates/plix-client/src/main.rs
- [ ] T063 [US5] Display final error after 3 failed attempts in crates/plix-client/src/main.rs
- [x] T064 [US5] Add unit test for retry state (mark failed, can retry) in crates/plix-client/src/matchmaking/retry.rs
- [x] T065 [US5] Add test for retry limit (stops after 3) in crates/plix-client/src/matchmaking/retry.rs

**Checkpoint**: Error handling works - players get clear feedback and auto-retry

---

## Phase 8: User Story 6 - Quick Play Menu Option (Priority: P3)

**Goal**: "Quick Play" option in pause menu

**Independent Test**: Open pause menu, select Quick Play, verify quick join triggers

### Implementation for User Story 6

- [ ] T066 [US6] Add QuickPlay variant to PauseMenuItem enum in crates/plix-client/src/ui/menu.rs
- [ ] T067 [US6] Insert "Quick Play" menu item after "Servers" in crates/plix-client/src/ui/menu.rs
- [ ] T068 [US6] Handle QuickPlay selection in menu input handler in crates/plix-client/src/main.rs
- [ ] T069 [US6] Trigger quick join with saved preferences from menu in crates/plix-client/src/main.rs
- [ ] T070 [US6] Add UiState::QuickJoining for loading display in crates/plix-client/src/ui/state.rs
- [ ] T071 [US6] Update test_pause_menu_navigation for new item in crates/plix-client/src/ui/menu.rs

**Checkpoint**: Menu integration works - players can quick join from UI

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Security, robustness, non-regression, documentation

### Security & Robustness

- [ ] T072 [P] Implement 5-second connection timeout in crates/plix-client/src/matchmaking/request.rs
- [ ] T073 [P] Implement 2-second debounce between quick join requests in crates/plix-client/src/main.rs
- [ ] T074 Handle malformed server response (no panic) in crates/plix-client/src/matchmaking/request.rs
- [ ] T075 Add test for malformed server data handling in crates/plix-client/src/matchmaking/request.rs

### Non-Regression Tests

- [ ] T076 [P] Verify /servers command still works in crates/plix-client/tests/server_browser_regression_test.rs
- [ ] T077 [P] Verify /connect <n> still works in crates/plix-client/tests/server_browser_regression_test.rs
- [ ] T078 [P] Verify direct connect without master still works in crates/plix-client/tests/direct_connect_test.rs

### Integration Tests

- [ ] T079 E2E test: quick join with mock master in crates/plix-client/tests/matchmaking_test.rs
- [ ] T080 Test: quick join selection correctness in crates/plix-client/tests/matchmaking_test.rs
- [ ] T081 Test: retry on connection failure in crates/plix-client/tests/matchmaking_test.rs
- [ ] T082 Test: fallback cascade in crates/plix-client/tests/matchmaking_test.rs

### Documentation

- [ ] T083 Update /help with quick join commands in crates/plix-client/src/console.rs
- [ ] T084 Run quickstart.md validation scenarios

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phases 3-4 (US1, US2)**: Depend on Phase 2 - P1 priority, MVP
- **Phases 5-7 (US3, US4, US5)**: Depend on Phase 2 - P2 priority
- **Phase 8 (US6)**: Depends on Phase 2 - P3 priority (optional)
- **Phase 9 (Polish)**: Depends on all desired user stories

### User Story Dependencies

- **US1 (Quick Join)**: Independent - needs only foundational phase
- **US2 (Scoring)**: Independent - can parallel with US1, both are MVP
- **US3 (Fallback)**: Builds on US1/US2 filtering/scoring
- **US4 (Preferences)**: Independent - profile extension
- **US5 (Error Handling)**: Builds on US1 connection flow
- **US6 (Menu)**: Independent - UI extension

### Parallel Opportunities

Within Phase 1:
- T002, T003, T004, T005 can run in parallel (different files)

Within Phase 2:
- T009, T011, T013 can run in parallel (different filter functions)

Within Phase 3 (US1):
- T016, T017 can run in parallel (different commands)

Within Phase 4 (US2):
- T025, T034 can run in parallel (different files: scoring.rs vs selection.rs)

Within Phase 9:
- T072, T073, T076, T077, T078 can run in parallel

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (filtering infrastructure)
3. Complete Phase 3: US1 (basic quick join command)
4. Complete Phase 4: US2 (scoring algorithm)
5. **STOP and VALIDATE**: Test quick join with scoring
6. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. Add US1 + US2 → **MVP Complete!** (basic quick join works)
3. Add US3 (Fallback) → Better "no match" handling
4. Add US4 (Preferences) → Convenience for returning players
5. Add US5 (Error Handling) → Robust error feedback
6. Add US6 (Menu) → UI polish
7. Polish phase → Production ready

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- US1 and US2 are both P1 (MVP) - can be done in parallel
- US3, US4, US5 are P2 - add after MVP validation
- US6 is P3 (optional) - menu convenience
- rand crate already in workspace dependencies (for tie-breaking)
- Feature builds on Feature 025 (profile) and Feature 026 (server browser)
