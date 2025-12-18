# Tasks: Inventory Hotbar

**Input**: Design documents from `/specs/021-inventory-hotbar/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included per constitution requirement (V. Code Quality: Mandatory Testing)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md, this is a Rust workspace with:
- `crates/plix-common/src/` - Shared types
- `crates/plix-server/src/` - Server logic
- `crates/plix-client/src/` - Client UI
- `crates/plix-server/tests/` - Integration tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create module structure and basic type definitions

- [X] T001 Create inventory module structure in crates/plix-common/src/inventory/mod.rs
- [X] T002 [P] Add ItemId type to crates/plix-common/src/types.rs
- [X] T003 [P] Add LootEntityId type to crates/plix-common/src/types.rs
- [X] T004 Create inventory module structure in crates/plix-server/src/inventory/mod.rs
- [X] T005 [P] Create loot module structure in crates/plix-server/src/loot/mod.rs
- [X] T006 Export inventory module from crates/plix-common/src/lib.rs
- [X] T007 Export inventory and loot modules from crates/plix-server/src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and protocol that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T008 Implement ItemKind enum (Weapon/Consumable/Tool) in crates/plix-common/src/inventory/item.rs
- [X] T009 Implement ItemDef struct with static item definitions in crates/plix-common/src/inventory/item.rs
- [X] T010 [P] Implement ItemStack struct in crates/plix-common/src/inventory/item_stack.rs
- [X] T011 [P] Implement Hotbar struct in crates/plix-common/src/inventory/hotbar.rs
- [X] T012 Add unit tests for ItemStack in crates/plix-common/src/inventory/item_stack.rs
- [X] T013 Add unit tests for Hotbar in crates/plix-common/src/inventory/hotbar.rs
- [X] T014 Implement item registry with Sword/HealthPack/BlockPlacer in crates/plix-server/src/inventory/item_registry.rs
- [X] T015 Add protocol messages (SelectHotbarSlot, UseActiveItem) to crates/plix-common/src/protocol/messages.rs
- [X] T016 Add SlotUpdate and InventoryUpdate to ServerMessage in crates/plix-common/src/protocol/messages.rs
- [X] T017 Add inventory GameEvents (LootSpawned, LootRemoved, LootPickedUp, ItemUsed) to crates/plix-common/src/protocol/messages.rs
- [X] T018 Implement LootEntity struct in crates/plix-server/src/loot/entity.rs
- [X] T019 Implement LootManager in crates/plix-server/src/loot/mod.rs

**Checkpoint**: Foundation ready - user story implementation can begin ✅

---

## Phase 3: User Story 1 - Basic Hotbar Display and Slot Selection (Priority: P1) 🎯 MVP

**Goal**: Players can see hotbar and select slots via keyboard (1-9) or scroll wheel

**Independent Test**: Spawn player → render hotbar → press 1-9 keys → verify selection changes

### Tests for User Story 1

- [X] T020 [P] [US1] Create hotbar_slot_test.rs in crates/plix-server/tests/hotbar_slot_test.rs
- [X] T021 [US1] Add test: slot selection validates bounds (0-8 for 9 slots) in crates/plix-server/tests/hotbar_slot_test.rs
- [X] T022 [US1] Add test: slot selection syncs to client via InventoryUpdate in crates/plix-server/tests/hotbar_slot_test.rs
- [X] T023 [US1] Add test: active_slot always valid after any operation in crates/plix-server/tests/hotbar_slot_test.rs

### Implementation for User Story 1

- [X] T024 [US1] Add Hotbar field to ServerPlayer in crates/plix-server/src/session.rs
- [X] T025 [US1] Implement handle_select_slot() in crates/plix-server/src/lib.rs
- [X] T026 [US1] Handle ClientMessage::SelectHotbarSlot in message dispatch in crates/plix-server/src/lib.rs
- [X] T027 [US1] Implement hotbar replication in snapshot (HotbarSnapshot) in crates/plix-server/src/replication/snapshot.rs
- [X] T028 [US1] Add AntiCheat ActionType::SlotSelect rate limiting in crates/plix-server/src/anti_cheat/mod.rs
- [X] T029 [US1] Add AntiCheat rate limit check for slot selection in crates/plix-server/src/anti_cheat/state.rs
- [X] T030 [US1] Run tests and verify all pass: cargo test -p plix-server hotbar_slot

**Checkpoint**: US1 complete - players can select hotbar slots ✅

---

## Phase 4: User Story 2 - Item Pickup and Hotbar Population (Priority: P1)

**Goal**: Players automatically pick up items within 1.5 blocks, items stack for consumables

**Independent Test**: Place loot → player walks over → item appears in hotbar

### Tests for User Story 2

- [ ] T031 [P] [US2] Create pickup_test.rs in crates/plix-server/tests/pickup_test.rs
- [ ] T032 [US2] Add test: pickup within 1.5 blocks adds to first empty slot in crates/plix-server/tests/pickup_test.rs
- [ ] T033 [US2] Add test: pickup fails when hotbar full (non-stackable) in crates/plix-server/tests/pickup_test.rs
- [ ] T034 [US2] Add test: consumable stacks to max 16 in crates/plix-server/tests/pickup_test.rs
- [ ] T035 [US2] Add test: pickup sends InventoryUpdate and LootRemoved events in crates/plix-server/tests/pickup_test.rs

### Implementation for User Story 2

- [ ] T036 [US2] Implement InventoryConfig with pickup_range in crates/plix-server/src/inventory/config.rs
- [ ] T037 [US2] Implement pickup_system.rs with try_pickup() in crates/plix-server/src/inventory/pickup_system.rs
- [ ] T038 [US2] Add LootManager to GameServer state in crates/plix-server/src/lib.rs
- [ ] T039 [US2] Implement check_pickups() in game tick loop in crates/plix-server/src/lib.rs
- [ ] T040 [US2] Send LootSpawned/LootRemoved/LootPickedUp events in crates/plix-server/src/lib.rs
- [ ] T041 [US2] Run tests: cargo test -p plix-server pickup

**Checkpoint**: US2 complete - players can pick up items

---

## Phase 5: User Story 3 - Item Usage (Priority: P1)

**Goal**: Players can use equipped items (attack/consume/tool action)

**Independent Test**: Select weapon → attack → verify damage dealt with weapon value (25)

### Tests for User Story 3

- [ ] T042 [P] [US3] Create item_use_test.rs in crates/plix-server/tests/item_use_test.rs
- [ ] T043 [US3] Add test: Sword deals 25 damage (not default melee) in crates/plix-server/tests/item_use_test.rs
- [ ] T044 [US3] Add test: Health Pack heals 50 HP, decrements stack in crates/plix-server/tests/item_use_test.rs
- [ ] T045 [US3] Add test: consumable removed when quantity reaches 0 in crates/plix-server/tests/item_use_test.rs
- [ ] T046 [US3] Add test: Block Placer places Stone block in crates/plix-server/tests/item_use_test.rs
- [ ] T047 [US3] Add test: empty slot uses default melee in crates/plix-server/tests/item_use_test.rs

### Implementation for User Story 3

- [ ] T048 [US3] Implement use_system.rs with use_active_item() in crates/plix-server/src/inventory/use_system.rs
- [ ] T049 [US3] Implement weapon effect (integrate with existing combat) in crates/plix-server/src/inventory/use_system.rs
- [ ] T050 [US3] Implement consumable effect (heal player) in crates/plix-server/src/inventory/use_system.rs
- [ ] T051 [US3] Implement tool effect (place block via existing system) in crates/plix-server/src/inventory/use_system.rs
- [ ] T052 [US3] Handle ClientMessage::UseActiveItem in message dispatch in crates/plix-server/src/lib.rs
- [ ] T053 [US3] Add AntiCheat ActionType::InventoryUse rate limiting in crates/plix-server/src/anti_cheat/mod.rs
- [ ] T054 [US3] Send ItemUsed event on successful use in crates/plix-server/src/lib.rs
- [ ] T055 [US3] Run tests: cargo test -p plix-server item_use

**Checkpoint**: US3 complete - players can use items

---

## Phase 6: User Story 4 - Server-Authoritative Inventory Validation (Priority: P2)

**Goal**: Server validates all inventory operations, rejects invalid requests

**Independent Test**: Send invalid slot index → verify rejection and anti-cheat warning

### Tests for User Story 4

- [ ] T056 [P] [US4] Create inventory_validation_test.rs in crates/plix-server/tests/inventory_validation_test.rs
- [ ] T057 [US4] Add test: invalid slot index rejected in crates/plix-server/tests/inventory_validation_test.rs
- [ ] T058 [US4] Add test: use item not in hotbar rejected in crates/plix-server/tests/inventory_validation_test.rs
- [ ] T059 [US4] Add test: pickup non-existent loot rejected in crates/plix-server/tests/inventory_validation_test.rs
- [ ] T060 [US4] Add test: race condition pickup (first request wins) in crates/plix-server/tests/inventory_validation_test.rs

### Implementation for User Story 4

- [ ] T061 [US4] Add validation checks to handle_select_slot() in crates/plix-server/src/lib.rs
- [ ] T062 [US4] Add validation checks to use_active_item() in crates/plix-server/src/inventory/use_system.rs
- [ ] T063 [US4] Add validation checks to try_pickup() in crates/plix-server/src/inventory/pickup_system.rs
- [ ] T064 [US4] Log anti-cheat warnings for invalid operations in crates/plix-server/src/lib.rs
- [ ] T065 [US4] Run tests: cargo test -p plix-server inventory_validation

**Checkpoint**: US4 complete - server rejects all invalid inventory operations

---

## Phase 7: User Story 5 - Item Drops on Death (Priority: P2)

**Goal**: Items drop on death (mode-dependent: FFA/BR drop, Training/TDM retain)

**Independent Test**: Kill player in FFA → verify loot entities spawn at death location

### Tests for User Story 5

- [ ] T066 [P] [US5] Create death_drop_test.rs in crates/plix-server/tests/death_drop_test.rs
- [ ] T067 [US5] Add test: FFA mode drops all items on death in crates/plix-server/tests/death_drop_test.rs
- [ ] T068 [US5] Add test: BR Lite drops all items on death in crates/plix-server/tests/death_drop_test.rs
- [ ] T069 [US5] Add test: Training mode does NOT drop items in crates/plix-server/tests/death_drop_test.rs
- [ ] T070 [US5] Add test: TDM mode does NOT drop items in crates/plix-server/tests/death_drop_test.rs
- [ ] T071 [US5] Add test: dropped loot spread 0.5-1.0 blocks from death position in crates/plix-server/tests/death_drop_test.rs

### Implementation for User Story 5

- [ ] T072 [US5] Implement loot spawner in crates/plix-server/src/loot/spawner.rs
- [ ] T073 [US5] Add drop_player_items() function in crates/plix-server/src/lib.rs
- [ ] T074 [US5] Add mode_drops_items() helper based on GameMode in crates/plix-server/src/lib.rs
- [ ] T075 [US5] Hook death drop logic into handle_player_death() in crates/plix-server/src/lib.rs
- [ ] T076 [US5] Run tests: cargo test -p plix-server death_drop

**Checkpoint**: US5 complete - death drops work correctly per game mode

---

## Phase 8: User Story 6 - Game Mode Compatibility (Priority: P2)

**Goal**: Each game mode has correct starting loadout and inventory behavior

**Independent Test**: Start Training mode → verify player has Sword + Health Packs

### Tests for User Story 6

- [ ] T077 [P] [US6] Create mode_loadout_test.rs in crates/plix-server/tests/mode_loadout_test.rs
- [ ] T078 [US6] Add test: Training mode gives Sword + Health Packs in crates/plix-server/tests/mode_loadout_test.rs
- [ ] T079 [US6] Add test: TDM/FFA/CTF give Sword only in crates/plix-server/tests/mode_loadout_test.rs
- [ ] T080 [US6] Add test: BR Lite starts with empty hotbar in crates/plix-server/tests/mode_loadout_test.rs

### Implementation for User Story 6

- [ ] T081 [US6] Add starting_loadouts to InventoryConfig in crates/plix-server/src/inventory/config.rs
- [ ] T082 [US6] Implement give_starting_loadout() in crates/plix-server/src/lib.rs
- [ ] T083 [US6] Call give_starting_loadout() on player spawn in crates/plix-server/src/lib.rs
- [ ] T084 [US6] Call give_starting_loadout() on player respawn in crates/plix-server/src/lib.rs
- [ ] T085 [US6] Run tests: cargo test -p plix-server mode_loadout

**Checkpoint**: US6 complete - all game modes have correct loadouts

---

## Phase 9: User Story 7 - Hotbar Configuration (Priority: P3)

**Goal**: Arena TOML can configure hotbar size and loadouts

**Independent Test**: Set hotbar_slots=5 in arena TOML → verify 5-slot hotbar

### Tests for User Story 7

- [ ] T086 [P] [US7] Create hotbar_config_test.rs in crates/plix-server/tests/hotbar_config_test.rs
- [ ] T087 [US7] Add test: arena hotbar_slots overrides default in crates/plix-server/tests/hotbar_config_test.rs
- [ ] T088 [US7] Add test: arena default_loadout populates hotbar in crates/plix-server/tests/hotbar_config_test.rs
- [ ] T089 [US7] Add test: missing config uses defaults (9 slots) in crates/plix-server/tests/hotbar_config_test.rs

### Implementation for User Story 7

- [ ] T090 [US7] Add hotbar_slots and default_loadout to arena TOML format in crates/plix-arena/src/format.rs
- [ ] T091 [US7] Load hotbar config from arena in crates/plix-arena/src/loader.rs
- [ ] T092 [US7] Apply arena config to InventoryConfig on match start in crates/plix-server/src/lib.rs
- [ ] T093 [US7] Run tests: cargo test -p plix-server hotbar_config

**Checkpoint**: US7 complete - arena-level configuration works

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Observability, final integration, and validation

- [ ] T094 Add inventory metrics (items_used_total, pickups_total) in crates/plix-server/src/metrics.rs
- [ ] T095 [P] Add tracing logs for slot_change, item_used, item_picked_up in crates/plix-server/src/lib.rs
- [ ] T096 Run full test suite: cargo test --workspace
- [ ] T097 Run clippy and fix warnings: cargo clippy --workspace
- [ ] T098 Run fmt check: cargo fmt --all -- --check
- [ ] T099 Verify all game modes still work: manual testing or integration tests
- [ ] T100 Update quickstart.md with actual test commands in specs/021-inventory-hotbar/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-9)**: All depend on Foundational phase completion
  - US1, US2, US3 are all P1 and should be done sequentially
  - US4, US5, US6 are P2 and can start after P1 stories
  - US7 is P3 and can start after P2 stories
- **Polish (Phase 10)**: Depends on all user stories being complete

### User Story Dependencies

| Story | Priority | Dependencies | Notes |
|-------|----------|--------------|-------|
| US1 (Slot Selection) | P1 | Foundational | Must complete first - MVP core |
| US2 (Item Pickup) | P1 | Foundational | Can parallel with US1, integrates LootManager |
| US3 (Item Usage) | P1 | Foundational | Can parallel with US1/US2 |
| US4 (Validation) | P2 | US1, US2, US3 | Hardens existing implementations |
| US5 (Death Drops) | P2 | US2 (LootManager) | Reuses pickup system |
| US6 (Mode Compat) | P2 | US1, US2, US3 | Tests full integration |
| US7 (Configuration) | P3 | US6 | Arena TOML extension |

### Parallel Opportunities

**Within Phase 2 (Foundational)**:
```
T010 (ItemStack) || T011 (Hotbar) - different files
```

**After Foundational**:
```
US1 || US2 || US3 - all P1 but can run in parallel
```

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch in parallel (different files):
Task: T010 [P] Implement ItemStack struct
Task: T011 [P] Implement Hotbar struct
```

## Parallel Example: P1 User Stories

```bash
# After Foundational, launch in parallel (if team capacity):
Task: T024-T030 (US1 Slot Selection)
Task: T036-T041 (US2 Item Pickup)
Task: T048-T055 (US3 Item Usage)
```

---

## Implementation Strategy

### MVP First (User Story 1, 2, 3)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational (T008-T019)
3. Complete Phase 3: US1 - Slot Selection (T020-T030)
4. Complete Phase 4: US2 - Item Pickup (T031-T041)
5. Complete Phase 5: US3 - Item Usage (T042-T055)
6. **STOP and VALIDATE**: All P1 stories functional

### Incremental Delivery

1. Setup + Foundational → Core types ready
2. Add US1 → Players can select slots → Demo
3. Add US2 → Players can pick up items → Demo
4. Add US3 → Players can use items → **MVP Complete!**
5. Add US4 → Anti-cheat hardened
6. Add US5 → Death drops work
7. Add US6 → All game modes supported
8. Add US7 → Arena configuration
9. Polish → Production ready

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story independently testable
- Tests written FIRST (TDD per constitution)
- Commit after each task or logical group
- Stop at any checkpoint to validate
