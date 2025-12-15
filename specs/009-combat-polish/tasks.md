# Tasks: Combat Polish

**Input**: Design documents from `/specs/009-combat-polish/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included per constitution ("Unit tests required for all combat logic")

**Scope**: Cooldown, range tuning, knockback, respawn invulnerability, hitreg under latency (tolerance-based, no rewind).

**Constraints**: Server-authoritative only, deterministic @60Hz, must not break movement/block interaction/match flow, headless + load tests must keep working.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace crates**: `crates/plix-common/src/`, `crates/plix-server/src/`, `crates/plix-client/src/`
- **Tests**: `crates/plix-server/tests/`

---

## Phase 1: Setup (Config & Data Model)

**Purpose**: Create CombatConfig struct and extend ServerPlayer state for all user stories

- [x] T001 [P] Create CombatConfig struct with all combat parameters in crates/plix-common/src/combat.rs
  - `attack_cooldown_ticks: u32 = 30`
  - `attack_range: f32 = 1.8`
  - `attack_range_epsilon: f32 = 0.15`
  - `knockback_strength: f32 = 4.0`
  - `respawn_invuln_ticks: u32 = 120`
- [x] T002 [P] Add Default impl and effective_range() method to CombatConfig in crates/plix-common/src/combat.rs
- [x] T003 [P] Export combat module and CombatConfig in crates/plix-common/src/lib.rs
- [x] T004 Verify server boots with default CombatConfig values

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Extend ServerPlayer state - MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Add invulnerable_until_tick: Option<Tick> field to ServerPlayer in crates/plix-server/src/session.rs
- [x] T006 Initialize invulnerable_until_tick to None in ServerPlayer::new in crates/plix-server/src/session.rs
- [x] T007 Add is_invulnerable(&self, current_tick: Tick) method to ServerPlayer in crates/plix-server/src/session.rs
- [x] T008 Add knockback_dir: Vec3 field to HitResult in crates/plix-server/src/sim/combat.rs
- [x] T009 Verify workspace compiles with `cargo build --workspace`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Cooldown-Based Attacks (Priority: P1) 🎯 MVP

**Goal**: Enforce server-side attack cooldown (30 ticks / 0.5s) to prevent spam

**Independent Test**: Attempt N attacks within cooldown window - only 1 hit allowed. Attacks after cooldown tick are allowed.

### Tests for User Story 1

- [x] T010 [P] [US1] Unit test: attack rejected during cooldown (ticks_since < 30) in crates/plix-server/tests/combat_cooldown_test.rs
- [x] T011 [P] [US1] Unit test: attack accepted after cooldown expires (ticks_since >= 30) in crates/plix-server/tests/combat_cooldown_test.rs
- [x] T012 [P] [US1] Unit test: rapid attack spam only registers first hit in crates/plix-server/tests/combat_cooldown_test.rs

### Implementation for User Story 1

- [x] T013 [US1] Update try_attack signature to accept &CombatConfig parameter in crates/plix-server/src/sim/combat.rs
- [x] T014 [US1] Replace ATTACK_COOLDOWN_TICKS constant with config.attack_cooldown_ticks in try_attack in crates/plix-server/src/sim/combat.rs
- [x] T015 [US1] Update last_attack_tick on successful attack in crates/plix-server/src/sim/combat.rs
- [x] T016 [US1] Update all try_attack call sites to pass &CombatConfig in crates/plix-server/src/
- [x] T017 [US1] Mark ATTACK_COOLDOWN_TICKS as #[deprecated] in crates/plix-server/src/validation.rs

**Checkpoint**: Cooldown enforced - attack spam impossible

---

## Phase 4: User Story 2 - Tuned Attack Range (Priority: P1) 🎯 MVP

**Goal**: Attack range is 1.8 blocks with 0.15 epsilon tolerance for clear spatial rules

**Independent Test**: Just inside range (1.8) hits, just outside range (2.0) misses, within epsilon (1.9) hits.

### Tests for User Story 2

- [x] T018 [P] [US2] Unit test: attack hits at exactly 1.8 blocks in crates/plix-server/tests/combat_range_test.rs
- [x] T019 [P] [US2] Unit test: attack misses at 2.0 blocks (beyond effective range) in crates/plix-server/tests/combat_range_test.rs
- [x] T020 [P] [US2] Unit test: attack hits at 1.9 blocks (within epsilon) in crates/plix-server/tests/combat_range_test.rs

### Implementation for User Story 2

- [x] T021 [US2] Update ATTACK_RANGE from 2.0 to 1.8 in crates/plix-server/src/validation.rs
- [x] T022 [US2] Add ATTACK_RANGE_EPSILON constant (0.15) in crates/plix-server/src/validation.rs
- [x] T023 [US2] Replace range check with config.effective_range() (attack_range + epsilon) in crates/plix-server/src/sim/combat.rs
- [x] T024 [US2] Ensure range check uses authoritative post-collision positions in crates/plix-server/src/sim/combat.rs
- [x] T025 [US2] Update existing range tests for new 1.8 + 0.15 = 1.95 effective range in crates/plix-server/tests/

**Checkpoint**: Range tuning works - clear spatial rules at 1.8 blocks + 0.15 tolerance

---

## Phase 5: User Story 3 - Knockback Feedback (Priority: P2)

**Goal**: Valid hits apply 4.0 m/s velocity impulse that respects collision

**Independent Test**: Knockback changes victim velocity in correct direction. Victim against wall doesn't clip.

### Tests for User Story 3

- [x] T026 [P] [US3] Unit test: knockback direction is normalize(victim_pos - attacker_pos) in crates/plix-server/tests/combat_knockback_test.rs
- [x] T027 [P] [US3] Unit test: knockback magnitude is config.knockback_strength in crates/plix-server/tests/combat_knockback_test.rs
- [x] T028 [P] [US3] Integration test: knockback against wall stops at surface (no clipping) in crates/plix-server/tests/combat_knockback_test.rs

### Implementation for User Story 3

- [x] T029 [US3] Calculate knockback_dir = normalize(victim_pos - attacker_pos) in try_attack in crates/plix-server/src/sim/combat.rs
- [x] T030 [US3] Populate knockback_dir field in HitResult on successful hit in crates/plix-server/src/sim/combat.rs
- [x] T031 [US3] Apply velocity impulse (knockback_dir * config.knockback_strength) to victim on hit in crates/plix-server/src/lib.rs
- [x] T032 [US3] Verify knockback respects existing collision system from Feature 008

**Checkpoint**: Knockback works - hits feel impactful, walls block movement correctly

---

## Phase 6: User Story 4 - Respawn Invulnerability (Priority: P2)

**Goal**: Respawned players invulnerable for 2 seconds (120 ticks) to prevent spawn-killing

**Independent Test**: After respawn, hits do nothing during 120-tick window. After expiry, hits work normally.

### Tests for User Story 4

- [x] T033 [P] [US4] Unit test: attack blocked on invulnerable target (no damage) in crates/plix-server/tests/combat_invuln_test.rs
- [x] T034 [P] [US4] Unit test: no knockback applied to invulnerable target in crates/plix-server/tests/combat_invuln_test.rs
- [x] T035 [P] [US4] Unit test: attack succeeds after invulnerability expires in crates/plix-server/tests/combat_invuln_test.rs
- [x] T036 [P] [US4] Unit test: invulnerable_until_tick set correctly on spawn in crates/plix-server/tests/combat_invuln_test.rs

### Implementation for User Story 4

- [x] T037 [US4] Update spawn() signature to accept current_tick: Tick and invuln_ticks: Option<u32> in crates/plix-server/src/session.rs
- [x] T038 [US4] Set invulnerable_until_tick = Some(Tick(current_tick.0 + invuln_ticks)) on spawn in crates/plix-server/src/session.rs
- [x] T039 [US4] Update all spawn() call sites to pass current_tick and invuln_ticks in crates/plix-server/src/
- [x] T040 [US4] Add invulnerability check before damage application (filter invulnerable targets from target list) in crates/plix-server/src/lib.rs
- [x] T041 [US4] Skip knockback if target.is_invulnerable(current_tick) - handled by filtering in crates/plix-server/src/lib.rs

**Checkpoint**: Invulnerability works - spawn protection prevents instant deaths

---

## Phase 7: User Story 5 - Latency-Tolerant Hit Registration (Priority: P3)

**Goal**: Hits feel fair at 30-80ms latency using tolerance-only approach (no server rewind)

**Independent Test**: Combat uses server's authoritative state at processing tick, not client-reported positions. Epsilon is only latency assistance.

### Tests for User Story 5

- [x] T042 [P] [US5] Unit test: hit uses server authoritative positions not client-reported in crates/plix-server/tests/combat_hitreg_test.rs
- [x] T043 [P] [US5] Determinism test: same inputs produce same outcomes across runs in crates/plix-server/tests/combat_hitreg_test.rs

### Implementation for User Story 5

- [x] T044 [US5] Verify try_attack uses last validated server positions (not client input) in crates/plix-server/src/sim/combat.rs
- [x] T045 [US5] Confirm epsilon is only latency assistance (no history buffer) in crates/plix-server/src/sim/combat.rs
- [x] T046 [US5] Mark ATTACK_RANGE as #[deprecated] with migration note in crates/plix-server/src/validation.rs

**Checkpoint**: Latency tolerance works - combat feels fair, deterministic, server-authoritative

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Validation, cleanup, and non-regression testing across all user stories

### Automated Validation

- [x] T047 Run cargo test --workspace and verify all tests pass (247 tests passing)
- [x] T048 Run cargo clippy --all-targets (warnings are pre-existing, not from this feature)
- [x] T049 Run cargo fmt --all -- --check and fix formatting issues

### Manual Validation

- [ ] T050 Manual test: 2 clients - try attack spam, verify only cooldown-respecting hits register
- [ ] T051 Manual test: 2 clients - borderline range attacks, verify consistent hits/misses
- [ ] T052 Manual test: knockback feels correct, no wall clipping
- [ ] T053 Manual test: respawn invuln prevents instant death at spawn
- [ ] T054 Manual test: moderate latency (simulated or real) still feels fair

### Non-Regression

- [ ] T055 Run load test: `./scripts/run_load_test.sh 8 30 127.0.0.1:7777` - verify stable, no perf regression
- [ ] T056 Run quickstart.md verification checklist

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 (Cooldown) and US2 (Range) are P1 - can proceed in parallel
  - US3 (Knockback) and US4 (Invulnerability) are P2 - can proceed in parallel after P1
  - US5 (Latency) is P3 - proceeds after P2
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1 Cooldown)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P1 Range)**: Can start after Foundational - No dependencies on other stories
- **User Story 3 (P2 Knockback)**: Can start after Foundational - Benefits from US1/US2 for hit validation
- **User Story 4 (P2 Invuln)**: Can start after Foundational - Independent of other stories
- **User Story 5 (P3 Latency)**: Can start after Foundational - Uses epsilon from US2

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Data model changes before service logic
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- T001, T002, T003 can run in parallel (different aspects of same file)
- T010, T011, T012 can run in parallel (different test functions)
- T018, T019, T020 can run in parallel (different test functions)
- T026, T027, T028 can run in parallel (different test functions)
- T033, T034, T035, T036 can run in parallel (different test functions)
- T042, T043 can run in parallel (different test functions)

---

## Parallel Example: Phase 1 Setup

```bash
# Launch all setup tasks together:
Task: "Create CombatConfig struct in crates/plix-common/src/combat.rs"
Task: "Add Default impl and effective_range() method"
Task: "Export combat module in crates/plix-common/src/lib.rs"
```

## Parallel Example: User Story 1 Tests

```bash
# Launch all US1 tests together:
Task: "Unit test: attack rejected during cooldown"
Task: "Unit test: attack accepted after cooldown expires"
Task: "Unit test: rapid attack spam only registers first hit"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup (CombatConfig)
2. Complete Phase 2: Foundational (ServerPlayer fields)
3. Complete Phase 3: User Story 1 (Cooldown)
4. Complete Phase 4: User Story 2 (Range)
5. **STOP and VALIDATE**: Test cooldown and range independently
6. Deploy/demo if ready - core combat fairness achieved

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 + US2 → Test → Deploy (MVP - cooldown + range = fair combat)
3. US3 + US4 → Test → Deploy (P2 - knockback + spawn protection)
4. US5 → Test → Deploy (P3 - latency polish)
5. Each increment adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Cooldown)
   - Developer B: User Story 2 (Range)
3. After P1 complete:
   - Developer A: User Story 3 (Knockback)
   - Developer B: User Story 4 (Invulnerability)
4. P3 can follow from either developer

---

## Definition of Done

- [ ] Cooldown enforced server-side; spam impossible
- [ ] Range uses tuned distance (1.8) + small epsilon (0.15)
- [ ] Knockback applied on valid hits and respects collision
- [ ] Respawn invulnerability blocks damage + knockback for 2s
- [ ] Hitreg remains server-authoritative; tolerance only (no rewind)
- [ ] All tests pass; manual check OK; load test OK
- [ ] cargo clippy and cargo fmt clean

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All config values come from CombatConfig::default() for consistency
- Knockback integrates with Feature 008 collision system
