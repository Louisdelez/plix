# Tasks: Weapons & Items v1

**Input**: Design documents from `/specs/022-weapons-items-v1/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included as requested in user input task breakdown.

**Organization**: Tasks grouped by user story (8 stories from spec.md, P1-P2 priorities).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1-US8) this task belongs to
- Include exact file paths in descriptions

## Path Conventions

Rust workspace structure:
- `crates/plix-common/src/` - Shared types
- `crates/plix-server/src/` - Server logic
- `crates/plix-server/tests/` - Integration tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create weapons module structure and shared types

- [ ] T001 Create weapons module directory structure at `crates/plix-server/src/weapons/`
- [ ] T002 [P] Add `ProjectileId` type to `crates/plix-common/src/types.rs`
- [ ] T003 [P] Add `ItemId::BOW` constant to `crates/plix-common/src/types.rs`
- [ ] T004 [P] Add `WeaponType` enum (Melee/Ranged) to `crates/plix-common/src/inventory/item.rs`
- [ ] T005 Create `mod.rs` with module exports at `crates/plix-server/src/weapons/mod.rs`
- [ ] T006 Add `pub mod weapons;` to `crates/plix-server/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: WeaponDef registry and Bow item definition - required by ALL user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T007 Define `WeaponDef` struct with all weapon parameters at `crates/plix-server/src/weapons/defs.rs`
- [ ] T008 Add `SWORD_DEF` constant (damage=25, cooldown=36 ticks, cone=30°, radius=2.5) at `crates/plix-server/src/weapons/defs.rs`
- [ ] T009 Add `BOW_DEF` constant (damage=15, cooldown=48 ticks, speed=30, ttl=180, spread=2.0°) at `crates/plix-server/src/weapons/defs.rs`
- [ ] T010 Add `FIST_DEF` constant (damage=10, cooldown=36 ticks, cone=30°, radius=2.0) at `crates/plix-server/src/weapons/defs.rs`
- [ ] T011 [P] Add Bow to `ITEM_DEFS` registry at `crates/plix-server/src/inventory/item_registry.rs`
- [ ] T012 [P] Add test for Bow ItemDef serialization at `crates/plix-server/tests/item_use_test.rs`
- [ ] T013 Add test for WeaponDef parameter consistency at `crates/plix-server/src/weapons/defs.rs`

**Checkpoint**: WeaponDefs ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Melee Combat with Sword (Priority: P1) 🎯 MVP

**Goal**: Sword deals 25 damage with 60° cone hit detection at 2.5 block range

**Independent Test**: Player can swing sword, hit enemies in cone, deal damage

### Tests for User Story 1

- [ ] T014 [P] [US1] Create melee combat test file at `crates/plix-server/tests/melee_combat_test.rs`
- [ ] T015 [P] [US1] Add test: hit if target within 2.5 blocks AND within 60° cone
- [ ] T016 [P] [US1] Add test: miss if target outside range (>2.5 blocks)
- [ ] T017 [P] [US1] Add test: miss if target outside cone (>60°)
- [ ] T018 [P] [US1] Add test: sword deals exactly 25 damage

### Implementation for User Story 1

- [ ] T019 [US1] Implement `MeleeSystem` with cone hit detection at `crates/plix-server/src/weapons/melee.rs`
- [ ] T020 [US1] Add `find_targets_in_range()` function using radius check (2.5 blocks)
- [ ] T021 [US1] Add `filter_by_cone()` function using dot product (60° = cos(30°) threshold)
- [ ] T022 [US1] Add `select_best_target()` function (closest/most aligned)
- [ ] T023 [US1] Connect MeleeSystem to existing health/damage system in `crates/plix-server/src/sim/combat.rs`
- [ ] T024 [US1] Verify sword kill triggers existing death handling

**Checkpoint**: Sword melee combat fully functional and independently testable

---

## Phase 4: User Story 2 - Ranged Combat with Bow (Priority: P1)

**Goal**: Bow fires arrow projectiles that travel, hit targets, and deal 15 damage

**Independent Test**: Player fires bow, arrow travels, impacts target for damage

### Tests for User Story 2

- [ ] T025 [P] [US2] Create ranged combat test file at `crates/plix-server/tests/ranged_combat_test.rs`
- [ ] T026 [P] [US2] Add test: bow creates projectile with correct parameters
- [ ] T027 [P] [US2] Add test: projectile moves each tick (pos += vel)
- [ ] T028 [P] [US2] Add test: projectile despawns at TTL expiry (180 ticks)
- [ ] T029 [P] [US2] Add test: projectile impact on player deals 15 damage
- [ ] T030 [P] [US2] Add test: projectile impact on block despawns arrow

### Implementation for User Story 2

- [ ] T031 [US2] Define `Projectile` struct at `crates/plix-server/src/weapons/projectiles.rs`
- [ ] T032 [US2] Implement `ProjectileManager` with Vec<Option<Projectile>> storage
- [ ] T033 [US2] Implement `ProjectileManager::spawn()` with generation ID handling
- [ ] T034 [US2] Implement `ProjectileManager::tick()` - move projectiles, decrement TTL
- [ ] T035 [US2] Implement `ProjectileManager::despawn()` for explicit removal
- [ ] T036 [US2] Add player collision detection (sphere-vs-capsule) in tick()
- [ ] T037 [US2] Add block collision detection (discrete stepping) in tick()
- [ ] T038 [US2] Connect projectile damage to health system

**Checkpoint**: Bow ranged combat fully functional and independently testable

---

## Phase 5: User Story 3 - Weapon Cooldowns (Priority: P1)

**Goal**: Server enforces per-weapon cooldowns (sword 0.6s, bow 0.8s)

**Independent Test**: Rapid attacks rejected, attacks succeed after cooldown

### Tests for User Story 3

- [ ] T039 [P] [US3] Create cooldown test file at `crates/plix-server/tests/cooldown_test.rs`
- [ ] T040 [P] [US3] Add test: sword cooldown rejects attack before 36 ticks
- [ ] T041 [P] [US3] Add test: sword cooldown allows attack after 36 ticks
- [ ] T042 [P] [US3] Add test: bow cooldown rejects shot before 48 ticks
- [ ] T043 [P] [US3] Add test: bow cooldown allows shot after 48 ticks
- [ ] T044 [P] [US3] Add test: switching weapons does NOT inherit cooldown

### Implementation for User Story 3

- [ ] T045 [US3] Implement `CooldownState` struct at `crates/plix-server/src/weapons/cooldown.rs`
- [ ] T046 [US3] Add `is_ready(item_id, tick)` method
- [ ] T047 [US3] Add `trigger(item_id, tick, duration)` method
- [ ] T048 [US3] Add `remaining(item_id, tick)` method
- [ ] T049 [US3] Integrate CooldownState into MeleeSystem (sword)
- [ ] T050 [US3] Integrate CooldownState into RangedSystem (bow)

**Checkpoint**: Cooldowns enforced for both weapons, independently testable

---

## Phase 6: User Story 4 - Accuracy and Movement Spread (Priority: P2)

**Goal**: Ranged weapons have spread, increased when moving

**Independent Test**: Arrows spread from aim direction, more spread when moving

### Tests for User Story 4

- [ ] T051 [P] [US4] Create spread test file at `crates/plix-server/tests/spread_recoil_test.rs`
- [ ] T052 [P] [US4] Add test: stationary shot has base spread (±2°)
- [ ] T053 [P] [US4] Add test: moving shot has increased spread (+50%)
- [ ] T054 [P] [US4] Add test: spread is bounded (never exceeds max)

### Implementation for User Story 4

- [ ] T055 [US4] Implement `calculate_spread()` function at `crates/plix-server/src/weapons/ranged.rs`
- [ ] T056 [US4] Implement `apply_spread()` function with deterministic RNG
- [ ] T057 [US4] Add `is_moving` detection based on player velocity
- [ ] T058 [US4] Integrate spread into projectile spawn direction

**Checkpoint**: Accuracy/spread system functional

---

## Phase 7: User Story 5 - Recoil System (Priority: P2)

**Goal**: Rapid firing accumulates spread penalty, resets after recovery window

**Independent Test**: Consecutive shots have increasing spread, waiting resets

### Tests for User Story 5

- [ ] T059 [P] [US5] Add test: first shot has base spread only
- [ ] T060 [P] [US5] Add test: rapid shots accumulate spread (+1° per shot)
- [ ] T061 [P] [US5] Add test: recoil caps at maximum (+5°)
- [ ] T062 [P] [US5] Add test: spread resets after recovery window (30 ticks)

### Implementation for User Story 5

- [ ] T063 [US5] Implement `RecoilState` struct at `crates/plix-server/src/weapons/recoil.rs`
- [ ] T064 [US5] Add `add_shot(penalty, tick)` method
- [ ] T065 [US5] Add `get_spread(tick)` method with decay calculation
- [ ] T066 [US5] Integrate RecoilState into RangedSystem spread calculation

**Checkpoint**: Recoil system functional

---

## Phase 8: User Story 6 - Hotbar Integration (Priority: P1)

**Goal**: UseActiveItem routes to correct weapon system based on hotbar slot

**Independent Test**: Selecting sword triggers melee, bow triggers ranged, empty triggers fist

### Tests for User Story 6

- [ ] T067 [P] [US6] Add test: UseActiveItem with Sword → MeleeSystem
- [ ] T068 [P] [US6] Add test: UseActiveItem with Bow → RangedSystem
- [ ] T069 [P] [US6] Add test: UseActiveItem with empty slot → default melee (fist)
- [ ] T070 [P] [US6] Add test: UseActiveItem with non-weapon → NotAWeapon result

### Implementation for User Story 6

- [ ] T071 [US6] Implement `RangedSystem` struct at `crates/plix-server/src/weapons/ranged.rs`
- [ ] T072 [US6] Add `try_shoot()` method with cooldown + limit + spread checks
- [ ] T073 [US6] Create `WeaponUseResult` enum with all outcomes
- [ ] T074 [US6] Implement weapon routing in `use_system.rs` based on ItemId
- [ ] T075 [US6] Add `PlayerWeaponState` to player session at `crates/plix-server/src/session.rs`
- [ ] T076 [US6] Wire weapon tick into main game loop

**Checkpoint**: Full weapon system integrated with hotbar

---

## Phase 9: User Story 7 - Game Mode Compatibility (Priority: P2)

**Goal**: Weapons work in all modes with correct damage rules

**Independent Test**: Weapon damage respects mode rules (friendly fire, etc.)

### Tests for User Story 7

- [ ] T077 [P] [US7] Add test: Training mode sword damages bot
- [ ] T078 [P] [US7] Add test: TDM sword damages enemy team
- [ ] T079 [P] [US7] Add test: TDM sword blocked by friendly fire setting
- [ ] T080 [P] [US7] Add test: weapon kill attribution correct

### Implementation for User Story 7

- [ ] T081 [US7] Add game mode check before applying weapon damage
- [ ] T082 [US7] Update loadouts for all modes at `crates/plix-server/src/loot/spawner.rs`
- [ ] T083 [US7] Training/TDM/FFA/CTF: Sword + Bow starting loadout
- [ ] T084 [US7] BR Lite: Add Bow to loot spawn pool
- [ ] T085 [US7] Update `get_starting_loadout()` in mode loadout tests

**Checkpoint**: Weapons work correctly in all game modes

---

## Phase 10: User Story 8 - Projectile Replication (Priority: P2)

**Goal**: Clients receive projectile spawn/impact/despawn events

**Independent Test**: Client receives events, no per-tick position updates

### Tests for User Story 8

- [ ] T086 [P] [US8] Add test: spawn generates ProjectileSpawn event
- [ ] T087 [P] [US8] Add test: impact generates ProjectileImpact event
- [ ] T088 [P] [US8] Add test: TTL expiry generates ProjectileDespawn event
- [ ] T089 [P] [US8] Add test: no per-tick position events sent

### Implementation for User Story 8

- [ ] T090 [US8] Add `ProjectileSpawn` to `GameEvent` enum at `crates/plix-common/src/protocol/messages.rs`
- [ ] T091 [US8] Add `ProjectileImpact` to `GameEvent` enum
- [ ] T092 [US8] Add `ProjectileDespawn` to `GameEvent` enum
- [ ] T093 [US8] Add `ProjectileImpactType` enum (Player/Bot/Block)
- [ ] T094 [US8] Add `ProjectileDespawnReason` enum (Timeout/LimitPurge/OwnerLeft)
- [ ] T095 [US8] Emit ProjectileSpawn in `ProjectileManager::spawn()`
- [ ] T096 [US8] Emit ProjectileImpact in collision handling
- [ ] T097 [US8] Emit ProjectileDespawn in TTL expiry handling
- [ ] T098 [US8] Add client handler stubs for projectile events

**Checkpoint**: Projectile replication complete

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Metrics, rejection handling, non-regression, documentation

### Observability & Metrics

- [ ] T099 [P] Add `melee_attacks_total` counter at `crates/plix-server/src/weapons/melee.rs`
- [ ] T100 [P] Add `ranged_shots_total` counter at `crates/plix-server/src/weapons/ranged.rs`
- [ ] T101 [P] Add `projectiles_active` gauge at `crates/plix-server/src/weapons/projectiles.rs`
- [ ] T102 [P] Add `cooldown_rejections` counter
- [ ] T103 [P] Add `projectile_limit_rejections` counter

### Rejection Handling

- [ ] T104 [P] Create projectile limit test file at `crates/plix-server/tests/projectile_limit_test.rs`
- [ ] T105 Add test: 128 projectiles active → 129th rejected
- [ ] T106 Add test: rejection increments limit counter
- [ ] T107 [P] Add `WeaponCooldown` rejection event to protocol
- [ ] T108 [P] Add `ProjectileLimitReached` rejection event to protocol

### Non-Regression Tests

- [ ] T109 Run existing hotbar tests - verify no regressions
- [ ] T110 Run existing mode tests (Training/TDM/FFA/CTF/BR) - verify no regressions
- [ ] T111 Run `cargo clippy --all-targets` - fix any warnings
- [ ] T112 Run `cargo fmt --all` - ensure formatting

### Integration Test

- [ ] T113 Create full combat integration test at `crates/plix-server/tests/weapon_integration_test.rs`
- [ ] T114 Test: Player A sword attacks Player B → B loses 25 HP
- [ ] T115 Test: Player A bow shoots Player B → projectile spawns → impact → B loses 15 HP
- [ ] T116 Test: verify spawn/impact/despawn events generated

### Documentation

- [ ] T117 Add weapons system documentation to `specs/022-weapons-items-v1/` README or inline comments

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-10)**: All depend on Foundational phase completion
- **Polish (Phase 11)**: Depends on all user story phases complete

### User Story Dependencies

| Story | Priority | Dependencies | Can Parallelize With |
|-------|----------|--------------|----------------------|
| US1 (Melee) | P1 | Foundational | US2, US3 |
| US2 (Ranged) | P1 | Foundational | US1, US3 |
| US3 (Cooldowns) | P1 | US1 + US2 (integration) | - |
| US4 (Spread) | P2 | US2 | US5 |
| US5 (Recoil) | P2 | US2 | US4 |
| US6 (Hotbar) | P1 | US1 + US2 + US3 | - |
| US7 (Modes) | P2 | US6 | US8 |
| US8 (Replication) | P2 | US2 | US7 |

### Critical Path

```
Setup → Foundational → US1 (melee) ─┐
                    → US2 (ranged) ─┼→ US3 (cooldowns) → US6 (hotbar) → US7 (modes) → Polish
                                    │
                                    └→ US4 (spread) → US5 (recoil)
                                    └→ US8 (replication)
```

### Parallel Opportunities

**Phase 1 (Setup)**: T002, T003, T004 can run in parallel

**Phase 2 (Foundational)**: T011, T012 can run in parallel with T007-T010

**User Story Tests**: All test tasks within a story marked [P] can run in parallel

**Cross-Story**: US1 and US2 can be developed in parallel after Foundational

---

## Parallel Example: User Story 2 (Ranged)

```bash
# Launch all tests in parallel:
Task: T025-T030 (all [P] [US2] tests)

# Launch models:
Task: T031 "Define Projectile struct"

# After model complete, implementation sequentially:
Task: T032-T038 (ProjectileManager implementation)
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 + 6)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL)
3. Complete Phase 3: US1 (Melee) - sword works
4. Complete Phase 4: US2 (Ranged) - bow works
5. Complete Phase 5: US3 (Cooldowns) - rate limiting works
6. Complete Phase 8: US6 (Hotbar Integration) - weapons usable from inventory
7. **STOP and VALIDATE**: Core combat functional
8. Deploy/demo MVP

### Full Feature Delivery

1. Complete MVP above
2. Add Phase 6: US4 (Spread) - accuracy mechanics
3. Add Phase 7: US5 (Recoil) - paced shooting reward
4. Add Phase 9: US7 (Mode Compatibility) - all modes work
5. Add Phase 10: US8 (Replication) - network events
6. Complete Phase 11: Polish

### Parallel Team Strategy

With 2 developers:
- Dev A: US1 (Melee) → US3 (Cooldowns) → US6 (Hotbar)
- Dev B: US2 (Ranged) → US4 (Spread) → US5 (Recoil)
- Both: US7, US8, Polish

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Run `cargo test` after each phase to verify no regressions
