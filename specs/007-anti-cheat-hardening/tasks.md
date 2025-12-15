# Tasks: Anti-Cheat Hardening

**Feature**: 007-anti-cheat-hardening
**Branch**: `007-anti-cheat-hardening`
**Input**: Design documents from `/specs/007-anti-cheat-hardening/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/anti_cheat_api.md

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Exact file paths included in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create anti-cheat module structure and configuration

- [x] T001 Create anti_cheat module directory structure in crates/plix-server/src/anti_cheat/
- [x] T002 [P] Create mod.rs with module exports in crates/plix-server/src/anti_cheat/mod.rs
- [x] T003 [P] Add `pub mod anti_cheat;` to crates/plix-server/src/lib.rs
- [x] T004 [P] Create AntiCheatConfig struct with defaults in crates/plix-server/src/anti_cheat/config.rs
- [x] T005 [P] Create InfractionType enum in crates/plix-server/src/anti_cheat/mod.rs
- [x] T006 [P] Create ActionType enum in crates/plix-server/src/anti_cheat/mod.rs
- [x] T007 Add AntiCheatConfig field to Server struct in crates/plix-server/src/lib.rs

**Checkpoint**: Module structure ready, server initializes with default config

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core state and types that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T008 Create AntiCheatState struct (strikes, counters, window_start) in crates/plix-server/src/anti_cheat/state.rs
- [x] T009 Implement AntiCheatState::new(tick) constructor in crates/plix-server/src/anti_cheat/state.rs
- [x] T010 Implement AntiCheatState::record_infraction() in crates/plix-server/src/anti_cheat/state.rs
- [x] T011 Add anti_cheat: AntiCheatState field to ServerPlayer in crates/plix-server/src/session.rs
- [x] T012 Initialize AntiCheatState when player connects in crates/plix-server/src/lib.rs (handle_connect)
- [x] T013 Verify workspace compiles with `cargo build --workspace`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Strict Input Validation (Priority: P1)

**Goal**: Reject NaN/INF, out-of-bounds, and invalid sequence inputs to prevent game state corruption

**Independent Test**: Send malformed inputs from test client, verify rejection without crash

### Tests for User Story 1

- [x] T014 [P] [US1] Create anti_cheat_test.rs with test_validate_input_rejects_nan() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T015 [P] [US1] Add test_validate_input_rejects_inf() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T016 [P] [US1] Add test_validate_input_rejects_out_of_bounds() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T017 [P] [US1] Add test_sequence_rejects_duplicate() in crates/plix-server/tests/anti_cheat_test.rs

### Implementation for User Story 1

- [x] T018 [P] [US1] Create validate.rs with floats_are_finite() helper in crates/plix-server/src/anti_cheat/validate.rs
- [x] T019 [US1] Implement validate_input() checking NaN/INF in crates/plix-server/src/anti_cheat/validate.rs
- [x] T020 [US1] Add bounds checking for move_forward, move_right (-1.0 to 1.0) in crates/plix-server/src/anti_cheat/validate.rs
- [x] T021 [US1] Add bounds checking for pitch (-PI/2 to PI/2) in crates/plix-server/src/anti_cheat/validate.rs
- [x] T022 [US1] Implement check_sequence() for duplicate/out-of-order detection in crates/plix-server/src/anti_cheat/state.rs
- [x] T023 [US1] Integrate validate_input() in handle_message() for ClientMessage::Input in crates/plix-server/src/lib.rs
- [x] T024 [US1] Add infraction recording on validation failure in crates/plix-server/src/lib.rs
- [x] T025 [US1] Add structured log for InputRejected events in crates/plix-server/src/lib.rs
- [x] T026 [US1] Run tests: `cargo test -p plix-server validate`

**Checkpoint**: US1 complete - all malformed inputs rejected, no server crashes

---

## Phase 4: User Story 2 - Rate Limiting (Priority: P1)

**Goal**: Limit action frequency to prevent spam/flood attacks

**Independent Test**: Send actions faster than allowed, verify excess rejected with infractions

### Tests for User Story 2

- [x] T027 [P] [US2] Add test_rate_limiter_allows_under_limit() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T028 [P] [US2] Add test_rate_limiter_rejects_over_limit() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T029 [P] [US2] Add test_rate_limiter_window_reset() in crates/plix-server/tests/anti_cheat_test.rs

### Implementation for User Story 2

- [x] T030 [P] [US2] Create rate_limiter.rs with FixedWindowLimiter struct in crates/plix-server/src/anti_cheat/rate_limiter.rs
- [x] T031 [US2] Implement check_rate_limit(action, tick, config) in crates/plix-server/src/anti_cheat/state.rs
- [x] T032 [US2] Implement maybe_reset_window(tick) in crates/plix-server/src/anti_cheat/state.rs
- [x] T033 [US2] Integrate input rate limit check in handle_message() for Input in crates/plix-server/src/lib.rs
- [x] T034 [US2] Integrate attack rate limit check for attack inputs in crates/plix-server/src/lib.rs
- [x] T035 [US2] Integrate block edit rate limit check for BlockEdit in crates/plix-server/src/lib.rs
- [x] T036 [US2] Integrate ready toggle rate limit check for ReadyToggle in crates/plix-server/src/lib.rs
- [x] T037 [US2] Add structured log for RateLimitExceeded events in crates/plix-server/src/lib.rs
- [x] T038 [US2] Run tests: `cargo test -p plix-server rate_limit`

**Checkpoint**: US2 complete - all action types rate-limited, excess actions rejected

---

## Phase 5: User Story 3 - Physics Sanity Checks (Priority: P1)

**Goal**: Detect speed hacks and teleportation via position/velocity validation

**Independent Test**: Claim impossible positions, verify server rejects and uses authoritative position

### Tests for User Story 3

- [x] T039 [P] [US3] Add test_movement_sanity_allows_normal_speed() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T040 [P] [US3] Add test_movement_sanity_rejects_speed_hack() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T041 [P] [US3] Add test_movement_sanity_rejects_teleport() in crates/plix-server/tests/anti_cheat_test.rs

### Implementation for User Story 3

- [x] T042 [P] [US3] Add last_valid_position field to AntiCheatState in crates/plix-server/src/anti_cheat/state.rs
- [x] T043 [US3] Implement validate_movement_delta(old_pos, new_pos, config) in crates/plix-server/src/anti_cheat/validate.rs
- [x] T044 [US3] Integrate movement sanity check in simulate_tick() after move_player() in crates/plix-server/src/lib.rs
- [x] T045 [US3] On sanity violation: keep authoritative position, record infraction in crates/plix-server/src/lib.rs
- [x] T046 [US3] Add structured log for SanityViolation events (throttled) in crates/plix-server/src/lib.rs
- [x] T047 [US3] Update last_valid_position after successful movement in crates/plix-server/src/lib.rs
- [x] T048 [US3] Run tests: `cargo test -p plix-server movement_sanity`

**Checkpoint**: US3 complete - speed hacks and teleports detected and blocked

---

## Phase 6: User Story 4 - Automatic Sanctions (Priority: P2)

**Goal**: Apply progressive warnings → kick → ban based on infraction accumulation

**Independent Test**: Accumulate infractions, verify warning at threshold, kick at threshold, ban at threshold

### Tests for User Story 4

- [x] T049 [P] [US4] Add test_sanction_warning_at_threshold() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T050 [P] [US4] Add test_sanction_kick_at_threshold() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T051 [P] [US4] Add test_sanction_ban_at_threshold() in crates/plix-server/tests/anti_cheat_test.rs
- [x] T052 [P] [US4] Add test_banned_ip_rejected_on_connect() in crates/plix-server/tests/anti_cheat_test.rs

### Implementation for User Story 4

- [x] T053 [P] [US4] Create SanctionType enum in crates/plix-server/src/anti_cheat/sanctions.rs
- [x] T054 [P] [US4] Create SanctionManager struct in crates/plix-server/src/anti_cheat/sanctions.rs
- [x] T055 [US4] Implement evaluate(strikes) -> Option<SanctionType> in crates/plix-server/src/anti_cheat/sanctions.rs
- [x] T056 [US4] Implement should_warn(state, tick) for rate-limited warnings in crates/plix-server/src/anti_cheat/sanctions.rs
- [x] T057 [P] [US4] Create BanList struct in crates/plix-server/src/anti_cheat/ban_list.rs
- [x] T058 [P] [US4] Create BanEntry struct in crates/plix-server/src/anti_cheat/ban_list.rs
- [x] T059 [US4] Implement BanList::is_banned(ip) in crates/plix-server/src/anti_cheat/ban_list.rs
- [x] T060 [US4] Implement BanList::add_ban(ip, reason, duration) in crates/plix-server/src/anti_cheat/ban_list.rs
- [x] T061 [US4] Implement BanList::cleanup_expired() in crates/plix-server/src/anti_cheat/ban_list.rs
- [x] T062 [US4] Add ban_list: BanList field to Server struct in crates/plix-server/src/lib.rs
- [x] T063 [US4] Add ServerMessage::Warning variant (if not exists) in crates/plix-common/src/protocol/messages.rs
- [x] T064 [US4] Implement apply_sanction() helper in Server in crates/plix-server/src/lib.rs
- [x] T065 [US4] Call sanction evaluation in tick() after processing inputs in crates/plix-server/src/lib.rs
- [x] T066 [US4] Check ban list in handle_connect() before accepting connection in crates/plix-server/src/lib.rs
- [x] T067 [US4] Send Rejected message with ban reason/duration for banned IPs in crates/plix-server/src/lib.rs
- [x] T068 [US4] Add structured log for SanctionApplied events in crates/plix-server/src/lib.rs
- [x] T069 [US4] Run tests: `cargo test -p plix-server sanction`

**Checkpoint**: US4 complete - progressive sanctions working, bans enforced

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Integration testing, regression checks, and validation

### Regression Tests

- [x] T070 [P] Run existing combat tests: `cargo test -p plix-server combat`
- [x] T071 [P] Run existing movement tests: `cargo test -p plix-server movement`
- [x] T072 [P] Run existing block edit tests: `cargo test -p plix-server block_edit`

### Integration Validation

- [x] T073 Run full workspace tests: `cargo test --workspace`
- [x] T074 Run clippy: `cargo clippy --workspace --all-targets`
- [x] T075 Run fmt check: `cargo fmt --all -- --check`
- [ ] T076 Manual test: two clients connect, play, no false positives
- [ ] T077 Load test: verify 8 bots for 30 seconds without rate-limit kicks

### Documentation

- [ ] T078 [P] Update quickstart.md with actual implementation details in specs/007-anti-cheat-hardening/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational
- **User Story 2 (Phase 4)**: Depends on Foundational (can run parallel to US1)
- **User Story 3 (Phase 5)**: Depends on Foundational (can run parallel to US1, US2)
- **User Story 4 (Phase 6)**: Depends on Foundational (can run parallel to US1-3)
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Independence

All user stories (US1-US4) depend only on the Foundational phase and can be implemented in parallel:

```
        ┌────────────┐
        │   Setup    │
        └─────┬──────┘
              │
        ┌─────▼──────┐
        │Foundational│
        └─────┬──────┘
              │
    ┌─────────┼─────────┬─────────┐
    │         │         │         │
┌───▼───┐ ┌───▼───┐ ┌───▼───┐ ┌───▼───┐
│  US1  │ │  US2  │ │  US3  │ │  US4  │
│ (P1)  │ │ (P1)  │ │ (P1)  │ │ (P2)  │
└───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘
    │         │         │         │
    └─────────┴─────────┴─────────┘
              │
        ┌─────▼──────┐
        │   Polish   │
        └────────────┘
```

### Within Each User Story

1. Tests written FIRST (verify they fail)
2. Implementation tasks in order
3. Run story-specific tests to verify
4. Story complete and independently testable

### Parallel Opportunities

**Setup (Phase 1)**:
```
T002, T003, T004, T005, T006 can run in parallel
```

**Each User Story** (tests before implementation):
```
# US1 Tests (parallel)
T014, T015, T016, T017

# US2 Tests (parallel)
T027, T028, T029

# US3 Tests (parallel)
T039, T040, T041

# US4 Tests (parallel)
T049, T050, T051, T052
```

**Polish (parallel regression)**:
```
T070, T071, T072, T078
```

---

## Implementation Strategy

### MVP First (P1 Stories Only)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational (T008-T013)
3. Complete Phase 3: US1 - Strict Input Validation (T014-T026)
4. **STOP and VALIDATE**: Test malformed inputs are rejected
5. Continue Phase 4: US2 - Rate Limiting (T027-T038)
6. Continue Phase 5: US3 - Physics Sanity Checks (T039-T048)
7. **DEPLOY**: P1 stories deliver core anti-cheat protection

### Full Feature (Add P2)

8. Complete Phase 6: US4 - Automatic Sanctions (T049-T069)
9. Complete Phase 7: Polish & Validation (T070-T078)
10. **DEPLOY**: Full anti-cheat with progressive sanctions

### Parallel Team Strategy

With 2+ developers:

1. Both complete Setup + Foundational together
2. Developer A: US1 (Validation) + US3 (Physics)
3. Developer B: US2 (Rate Limiting) + US4 (Sanctions)
4. Merge and run integration tests

---

## Summary

| Phase | Tasks | Parallel Opportunities |
|-------|-------|----------------------|
| Setup | T001-T007 (7) | 5 tasks |
| Foundational | T008-T013 (6) | 0 (sequential) |
| US1 Validation | T014-T026 (13) | 5 tasks |
| US2 Rate Limiting | T027-T038 (12) | 4 tasks |
| US3 Physics | T039-T048 (10) | 4 tasks |
| US4 Sanctions | T049-T069 (21) | 8 tasks |
| Polish | T070-T078 (9) | 4 tasks |
| **Total** | **78 tasks** | **30 parallelizable** |

### Task Distribution by User Story

- **US1 (Strict Input Validation)**: 13 tasks
- **US2 (Rate Limiting)**: 12 tasks
- **US3 (Physics Sanity)**: 10 tasks
- **US4 (Automatic Sanctions)**: 21 tasks

### Suggested MVP Scope

**MVP = Phase 1 + Phase 2 + US1 + US2 + US3** (48 tasks)
- Delivers core anti-cheat protection
- All P1 stories complete
- Server protected from malformed inputs, floods, and speed hacks
- Sanctions (US4) can be added later without blocking core protection
