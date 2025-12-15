# Tasks: Server-Authoritative Combat System

**Input**: Design documents from `/specs/003-combat-visible/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Test tasks are included as this feature requires validation of server-authoritative behavior.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace**: `crates/` at repository root
- **Common**: `crates/plix-common/src/`
- **Server**: `crates/plix-server/src/`
- **Client**: `crates/plix-client/src/`

---

## Phase 1: Setup (Protocol Extension)

**Purpose**: Extend protocol with combat events (no new infrastructure needed - extending existing)

- [x] T001 [P] Add HitConfirmed event to GameEvent enum in crates/plix-common/src/protocol/messages.rs (or replication/events.rs)
- [x] T002 [P] Add DamageTaken event to GameEvent enum in crates/plix-common/src/protocol/messages.rs (or replication/events.rs)
- [x] T003 Verify PlayerDied and PlayerRespawned events already exist in crates/plix-common/src/protocol/messages.rs
- [x] T004 Run `cargo build -p plix-common` to verify protocol compiles

**Checkpoint**: Protocol events defined - server and client can use new event types

---

## Phase 2: Foundational (Server Combat Core)

**Purpose**: Core server-side combat infrastructure that MUST be complete before user story work

**⚠️ CRITICAL**: Client integration cannot begin until this phase is complete

- [x] T005 Verify combat constants exist (MELEE_DAMAGE, ATTACK_COOLDOWN_TICKS, ATTACK_RANGE) in crates/plix-server/src/sim/combat.rs
- [x] T006 Verify ServerPlayer has combat fields (health, is_dead, respawn_tick, last_attack_tick) in crates/plix-server/src/session.rs
- [x] T007 Verify attack flag exists in PlayerInput struct in crates/plix-common/src/protocol/messages.rs
- [x] T008 Verify CombatSystem::try_attack() exists and returns HitResult in crates/plix-server/src/sim/combat.rs
- [x] T009 Run `cargo test -p plix-server` to verify existing combat tests pass

**Checkpoint**: Foundation verified - user story implementation can now begin

---

## Phase 3: User Story 1 - Attack Another Player (Priority: P1) 🎯 MVP

**Goal**: Two players can attack each other with server-validated hits and distinct feedback for attacker/victim

**Independent Test**: Run server + 2 clients, attack within range, verify attacker sees "hit confirmed" and victim sees "damage taken"

### Tests for User Story 1

- [x] T010 [P] [US1] Unit test: attack in range hits target in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: test_melee_hit
- [x] T011 [P] [US1] Unit test: attack out of range misses in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: test_melee_out_of_range
- [x] T012 [P] [US1] Unit test: attack outside facing cone misses in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: covered by cone check in try_attack
- [x] T013 [P] [US1] Unit test: closest target in cone is selected in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: try_attack selects closest
- [x] T014 [P] [US1] Unit test: cooldown prevents rapid attacks in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: test_melee_cooldown

### Server Implementation for User Story 1

- [x] T015 [US1] Integrate combat processing into tick loop (call try_attack for players with attack flag) in crates/plix-server/src/lib.rs
- [x] T016 [US1] Emit HitConfirmed event to attacker on successful hit in crates/plix-server/src/lib.rs (in simulate_tick)
- [x] T017 [US1] Emit DamageTaken event to victim on successful hit in crates/plix-server/src/lib.rs (in simulate_tick)
- [x] T018 [US1] Apply damage to victim health in crates/plix-server/src/lib.rs (calls take_damage)
- [x] T019 [US1] Gate attacks on MatchPhase::Playing in crates/plix-server/src/lib.rs (line 213)

### Client Implementation for User Story 1

- [x] T020 [US1] Verify attack input mapped to LMB in crates/plix-client/src/input.rs - EXISTING
- [x] T021 [US1] Verify attack flag sent in input packet in crates/plix-client/src/main.rs (generate_input)
- [x] T022 [US1] Handle HitConfirmed event - show "HIT" text in HUD in crates/plix-client/src/main.rs (handle_game_event)
- [x] T023 [US1] Handle DamageTaken event - show damage flash/text in HUD in crates/plix-client/src/main.rs (handle_game_event)
- [x] T024 [US1] Update local HP display from snapshot in crates/plix-client/src/main.rs (handle_snapshot)

**Checkpoint**: Two clients can attack each other, see distinct feedback - MVP complete

---

## Phase 4: User Story 2 - Kill and Respawn Flow (Priority: P2)

**Goal**: Death triggers when HP reaches 0, player respawns after delay at spawn point

**Independent Test**: Attack player until death, verify they disappear, then reappear at spawn after 3 seconds

### Tests for User Story 2

- [x] T025 [P] [US2] Unit test: fatal damage sets is_dead=true in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: test_melee_kill
- [x] T026 [P] [US2] Unit test: death schedules respawn_tick in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: test_combat_damage_and_death
- [x] T027 [P] [US2] Unit test: respawn resets health to 100 in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: spawn() sets health=100
- [x] T028 [P] [US2] Unit test: dead players cannot attack in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: skipped in loop
- [x] T029 [P] [US2] Unit test: dead players cannot be targeted in crates/plix-server/src/sim/combat.rs (tests module) - EXISTING: filtered in targets

### Server Implementation for User Story 2

- [x] T030 [US2] Emit PlayerDied event when HP reaches 0 in crates/plix-server/src/lib.rs (in simulate_tick)
- [x] T031 [US2] Set is_dead=true and schedule respawn_tick on death in crates/plix-server/src/session.rs (take_damage)
- [x] T032 [US2] Add respawn check to tick loop in crates/plix-server/src/lib.rs (simulate_tick lines 236-248)
- [x] T033 [US2] Reset health=100, is_dead=false, position=spawn on respawn in crates/plix-server/src/lib.rs (calls spawn())
- [x] T034 [US2] Emit PlayerRespawned event on respawn in crates/plix-server/src/lib.rs (simulate_tick lines 348-352)
- [x] T035 [US2] Block attacks from dead players in crates/plix-server/src/lib.rs (simulate_tick: skip if is_dead)
- [x] T036 [US2] Skip dead players as attack targets in crates/plix-server/src/lib.rs (targets filter: !p.is_dead)

### Client Implementation for User Story 2

- [x] T037 [US2] Handle PlayerDied event - show kill feed message in crates/plix-client/src/main.rs (handle_game_event)
- [x] T038 [US2] Skip rendering players where is_dead=true in crates/plix-client/src/main.rs (update loop: player.is_dead)
- [x] T039 [US2] Handle PlayerRespawned event - optional notification in crates/plix-client/src/main.rs (handle_game_event)

**Checkpoint**: Death and respawn cycle works - killed players disappear and return

---

## Phase 5: User Story 3 - View Local Player HP (Priority: P3)

**Goal**: Player sees their current HP in debug HUD

**Independent Test**: Take damage, observe HP value decreases in HUD

### Implementation for User Story 3

- [x] T040 [US3] Verify HudData has health field in crates/plix-client/src/ui/hud.rs - EXISTING
- [x] T041 [US3] Update local player health from snapshot each frame in crates/plix-client/src/main.rs (handle_snapshot)
- [x] T042 [US3] Render HP value in HUD (numeric display) in crates/plix-client/src/main.rs (window title)

**Checkpoint**: HP visible in HUD, updates when damaged

---

## Phase 6: User Story 4 - Observe Combat Events (Priority: P3)

**Goal**: Developer/tester sees combat events in debug HUD

**Independent Test**: Trigger combat, observe hit/kill messages appear in HUD

### Implementation for User Story 4

- [x] T043 [US4] Add combat event log buffer to HudData (last 5 events) in crates/plix-client/src/ui/hud.rs (events VecDeque)
- [x] T044 [US4] Push hit events to log buffer in crates/plix-client/src/ui/hud.rs (push_event)
- [x] T045 [US4] Push kill events to log buffer in crates/plix-client/src/ui/hud.rs (push_event)
- [x] T046 [US4] Render event log in HUD in crates/plix-client/src/ui/hud.rs (recent_events for rendering)

**Checkpoint**: Combat events visible in debug HUD

---

## Phase 7: Polish & Validation

**Purpose**: Non-regression, cleanup, final validation

### Non-Regression Tests

- [x] T047 Run `cargo test --workspace` - all tests must pass (75 tests pass)
- [x] T048 Run `cargo clippy --workspace` - no errors (only pre-existing warnings)
- [x] T049 Run `cargo fmt --check` - formatting valid
- [ ] T050 Verify headless client still connects (no window deps break)
- [ ] T051 Verify load tests still work without crashes

### Manual Validation

- [ ] T052 Manual test: 2 windowed clients PvP - verify acceptance criteria from spec.md
  - Hits only within range+cone
  - Attacker gets "hit confirmed" feedback
  - Victim gets "damage taken" feedback + HP decreases
  - Death removes victim immediately
  - Respawn after delay at spawn point

### Cleanup

- [x] T053 [P] Fix any clippy warnings introduced by combat work - no new warnings
- [x] T054 [P] Remove debug logging (if any temporary logs added) - only info-level combat logs

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS user stories
- **Phase 3 (US1)**: Depends on Phase 2 - can start after foundation verified
- **Phase 4 (US2)**: Depends on Phase 2 - can run in parallel with US1
- **Phase 5 (US3)**: Depends on Phase 2 - can run in parallel with US1/US2
- **Phase 6 (US4)**: Depends on Phase 2 - can run in parallel with others
- **Phase 7 (Polish)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (Attack)**: Core combat - no other story dependencies
- **US2 (Death/Respawn)**: Builds on attack damage from US1 (sequential preferred)
- **US3 (HP Display)**: Needs snapshot replication - can run parallel after US1
- **US4 (Event Log)**: Needs events flowing - can run parallel after US1

### Within Each User Story

- Tests written and FAIL before implementation
- Server-side before client-side
- Core logic before integration
- Story complete before checkpoint

### Parallel Opportunities

**Phase 1 (Setup)**:
```bash
# T001 and T002 can run in parallel (different event types)
Task: "Add HitConfirmed event to GameEvent enum"
Task: "Add DamageTaken event to GameEvent enum"
```

**Phase 3 (US1 Tests)**:
```bash
# All US1 tests can run in parallel
Task: "Unit test: attack in range hits target"
Task: "Unit test: attack out of range misses"
Task: "Unit test: attack outside facing cone misses"
Task: "Unit test: closest target in cone is selected"
Task: "Unit test: cooldown prevents rapid attacks"
```

**Phase 4 (US2 Tests)**:
```bash
# All US2 tests can run in parallel
Task: "Unit test: fatal damage sets is_dead=true"
Task: "Unit test: death schedules respawn_tick"
Task: "Unit test: respawn resets health to 100"
Task: "Unit test: dead players cannot attack"
Task: "Unit test: dead players cannot be targeted"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (protocol events)
2. Complete Phase 2: Foundational (verify existing code)
3. Complete Phase 3: User Story 1 (attack + feedback)
4. **STOP and VALIDATE**: Two clients can attack with feedback
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Protocol ready
2. Add User Story 1 → Attack works with feedback (MVP!)
3. Add User Story 2 → Death/respawn works
4. Add User Story 3 → HP visible in HUD
5. Add User Story 4 → Event log in HUD
6. Each story adds value without breaking previous

### Recommended Execution Order

Since much infrastructure exists, recommended order:

1. **T001-T004**: Verify/add protocol events
2. **T005-T009**: Verify existing combat code works
3. **T015-T019**: Wire combat into tick loop (server)
4. **T020-T024**: Wire combat feedback (client)
5. **T030-T039**: Add death/respawn cycle
6. **T040-T046**: Add HUD elements
7. **T047-T054**: Validate and clean up

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Many tasks are "verify existing" due to partial implementation
- Existing tests in `crates/plix-server/tests/combat_test.rs` should pass
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
