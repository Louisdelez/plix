# Tasks: Crafting Lite

**Input**: Design documents from `/specs/023-crafting-lite/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Included per user tasks and Constitution requirement (V. Code Quality - mandatory testing)

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace crates**: `crates/plix-common/src/`, `crates/plix-server/src/`, `crates/plix-client/src/`
- **Tests**: `crates/plix-server/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create crafting module structure and add shared types

- [x] T001 Add ItemId::SCRAP constant (value 5) in crates/plix-common/src/types.rs
- [x] T002 [P] Add ItemKind::Resource variant in crates/plix-common/src/inventory/item.rs
- [x] T003 [P] Create crafting module directory structure crates/plix-server/src/crafting/
- [x] T004 Create crafting module entry point in crates/plix-server/src/crafting/mod.rs
- [x] T005 [P] Add SCRAP_DEF item definition in crates/plix-server/src/inventory/item_registry.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and protocol messages that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Create RecipeId struct (String newtype) in crates/plix-server/src/crafting/recipe.rs
- [ ] T007 Create Ingredient struct (item_id, quantity) in crates/plix-server/src/crafting/recipe.rs
- [ ] T008 Create Recipe struct (id, inputs, output_item, output_quantity) in crates/plix-server/src/crafting/recipe.rs
- [ ] T009 Add Recipe validation (non-empty inputs, output_quantity > 0) in crates/plix-server/src/crafting/recipe.rs
- [ ] T010 [P] Create CraftFailReason enum in crates/plix-server/src/crafting/errors.rs
- [ ] T011 [P] Add CraftRequest variant to ClientMessage in crates/plix-common/src/protocol/messages.rs
- [ ] T012 [P] Add CraftResult variant to GameEvent in crates/plix-common/src/protocol/messages.rs
- [ ] T013 Add count_item method to Hotbar in crates/plix-common/src/inventory/hotbar.rs
- [ ] T014 Add consume_items method to Hotbar in crates/plix-common/src/inventory/hotbar.rs
- [ ] T015 Add can_add_check method to Hotbar (validate output space) in crates/plix-common/src/inventory/hotbar.rs
- [ ] T016 Add unit tests for count_item and consume_items in crates/plix-common/src/inventory/hotbar.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Simple Item Crafting (Priority: P1) 🎯 MVP

**Goal**: Players can craft items when they have required ingredients in hotbar

**Independent Test**: Player with 2+ SCRAP crafts health_pack recipe → SCRAP consumed, HEALTH_PACK added

### Tests for User Story 1

- [ ] T017 [P] [US1] Unit test: recipe lookup (exists/not exists) in crates/plix-server/src/crafting/recipe.rs
- [ ] T018 [P] [US1] Unit test: ingredient validation (sufficient/insufficient) in crates/plix-server/src/crafting/system.rs
- [ ] T019 [P] [US1] Unit test: output space validation in crates/plix-server/src/crafting/system.rs
- [ ] T020 [P] [US1] Unit test: atomic craft (success consumes inputs + adds output) in crates/plix-server/src/crafting/system.rs
- [ ] T021 [P] [US1] Unit test: atomic craft (failure leaves hotbar unchanged) in crates/plix-server/src/crafting/system.rs

### Implementation for User Story 1

- [ ] T022 [US1] Create RecipeRegistry with get/exists methods in crates/plix-server/src/crafting/recipe.rs
- [ ] T023 [US1] Add static RECIPE_REGISTRY with 3 v1 recipes (health_pack, sword, bow) in crates/plix-server/src/crafting/recipe.rs
- [ ] T024 [US1] Create CraftSystem struct in crates/plix-server/src/crafting/system.rs
- [ ] T025 [US1] Implement validate_ingredients method in crates/plix-server/src/crafting/system.rs
- [ ] T026 [US1] Implement validate_output_space method in crates/plix-server/src/crafting/system.rs
- [ ] T027 [US1] Implement apply_craft method (consume inputs, add output) in crates/plix-server/src/crafting/system.rs
- [ ] T028 [US1] Implement try_craft orchestration method in crates/plix-server/src/crafting/system.rs
- [ ] T029 [US1] Export CraftSystem from crafting module in crates/plix-server/src/crafting/mod.rs

**Checkpoint**: Core crafting logic complete - can craft items with valid ingredients

---

## Phase 4: User Story 2 - Server-Authoritative Validation (Priority: P1)

**Goal**: Server validates all craft requests and rejects invalid ones appropriately

**Independent Test**: Send invalid craft requests → all rejected with correct error codes

### Tests for User Story 2

- [ ] T030 [P] [US2] Unit test: unknown recipe returns UnknownRecipe error in crates/plix-server/src/crafting/system.rs
- [ ] T031 [P] [US2] Unit test: missing ingredients returns MissingIngredients error in crates/plix-server/src/crafting/system.rs
- [ ] T032 [P] [US2] Unit test: full hotbar returns HotbarFull error in crates/plix-server/src/crafting/system.rs
- [ ] T033 [P] [US2] Unit test: dead player returns PlayerDead error in crates/plix-server/src/crafting/system.rs

### Implementation for User Story 2

- [ ] T034 [US2] Add is_alive parameter to try_craft validation in crates/plix-server/src/crafting/system.rs
- [ ] T035 [US2] Handle CraftRequest in Server message processing in crates/plix-server/src/lib.rs
- [ ] T036 [US2] Send CraftResult event back to requesting player in crates/plix-server/src/lib.rs
- [ ] T037 [US2] Send InventoryUpdate after successful craft in crates/plix-server/src/lib.rs

**Checkpoint**: Server properly validates and responds to all craft requests

---

## Phase 5: User Story 3 - Atomic Craft Operations (Priority: P1)

**Goal**: Craft operations are all-or-nothing - no partial states ever occur

**Independent Test**: Verify inventory unchanged when craft fails at any validation step

### Tests for User Story 3

- [ ] T038 [P] [US3] Unit test: multi-slot ingredient consumption in crates/plix-server/src/crafting/system.rs
- [ ] T039 [P] [US3] Unit test: output stacking with existing stack in crates/plix-server/src/crafting/system.rs
- [ ] T040 [P] [US3] Unit test: output to empty slot when no stackable in crates/plix-server/src/crafting/system.rs

### Implementation for User Story 3

- [ ] T041 [US3] Ensure consume_items handles multi-slot consumption in crates/plix-common/src/inventory/hotbar.rs
- [ ] T042 [US3] Verify atomic sequence: validate-all → then apply-all in crates/plix-server/src/crafting/system.rs

**Checkpoint**: Atomicity guaranteed for all craft operations

---

## Phase 6: User Story 4 - Game Mode Configuration (Priority: P2)

**Goal**: Crafting can be enabled/disabled per game mode

**Independent Test**: Craft succeeds in Training, fails in TDM with "crafting disabled"

### Tests for User Story 4

- [ ] T043 [P] [US4] Unit test: Training mode allows crafting in crates/plix-server/src/crafting/config.rs
- [ ] T044 [P] [US4] Unit test: TDM mode rejects crafting in crates/plix-server/src/crafting/config.rs
- [ ] T045 [P] [US4] Unit test: BR Lite mode allows crafting in crates/plix-server/src/crafting/config.rs

### Implementation for User Story 4

- [ ] T046 [US4] Create CraftConfig struct (enabled, allowed_recipes) in crates/plix-server/src/crafting/config.rs
- [ ] T047 [US4] Create get_craft_config(GameMode) function in crates/plix-server/src/crafting/config.rs
- [ ] T048 [US4] Integrate mode check in try_craft validation in crates/plix-server/src/crafting/system.rs
- [ ] T049 [US4] Export config from crafting module in crates/plix-server/src/crafting/mod.rs

**Checkpoint**: Crafting respects game mode configuration

---

## Phase 7: User Story 5 - Resource Items for Crafting (Priority: P2)

**Goal**: SCRAP resource spawns as loot and in Training loadout

**Independent Test**: SCRAP appears in BR Lite loot drops and Training starter inventory

### Tests for User Story 5

- [ ] T050 [P] [US5] Unit test: SCRAP item definition exists in item registry in crates/plix-server/src/inventory/item_registry.rs
- [ ] T051 [P] [US5] Unit test: Training loadout includes 5x SCRAP in crates/plix-server/src/inventory/config.rs

### Implementation for User Story 5

- [ ] T052 [US5] Update Training loadout to include 5x SCRAP in crates/plix-server/src/inventory/config.rs
- [ ] T053 [US5] Add SCRAP to BR Lite loot table in crates/plix-server/src/br_lite/loot.rs

**Checkpoint**: Resource items available for crafting in appropriate modes

---

## Phase 8: User Story 6 - Crafting Feedback (Priority: P3)

**Goal**: Players receive clear feedback on craft success/failure

**Independent Test**: CraftResult event sent with correct success/failure info

### Tests for User Story 6

- [ ] T054 [P] [US6] Integration test: successful craft sends CraftResult(success=true) in crates/plix-server/tests/crafting_test.rs
- [ ] T055 [P] [US6] Integration test: failed craft sends CraftResult with fail_reason in crates/plix-server/tests/crafting_test.rs

### Implementation for User Story 6

- [ ] T056 [US6] Ensure CraftResult includes output_item and output_quantity on success in crates/plix-server/src/lib.rs
- [ ] T057 [US6] Ensure CraftResult includes detailed fail_reason on failure in crates/plix-server/src/lib.rs

**Checkpoint**: Client receives appropriate feedback for all craft attempts

---

## Phase 9: User Story 7 - Extensible Recipe System (Priority: P3)

**Goal**: New recipes can be added without changing core crafting logic

**Independent Test**: Add new recipe to registry, verify it works immediately

### Tests for User Story 7

- [ ] T058 [P] [US7] Unit test: registry accepts new recipes in crates/plix-server/src/crafting/recipe.rs
- [ ] T059 [P] [US7] Unit test: new recipe craftable without system changes in crates/plix-server/src/crafting/system.rs

### Implementation for User Story 7

- [ ] T060 [US7] Document recipe addition process in crates/plix-server/src/crafting/recipe.rs
- [ ] T061 [US7] Verify registry lookup is generic (no hardcoded recipe IDs in system.rs)

**Checkpoint**: Recipe system is extensible

---

## Phase 10: Cooldown & Rate Limiting

**Goal**: Prevent craft spamming with 1-second cooldown after successful crafts

**Independent Test**: Craft succeeds, immediate retry fails, retry after 1s succeeds

### Tests for Cooldown

- [ ] T062 [P] Unit test: cooldown not active initially in crates/plix-server/src/crafting/cooldown.rs
- [ ] T063 [P] Unit test: cooldown triggers after successful craft in crates/plix-server/src/crafting/cooldown.rs
- [ ] T064 [P] Unit test: craft rejected during cooldown in crates/plix-server/src/crafting/cooldown.rs
- [ ] T065 [P] Unit test: cooldown expires after 60 ticks in crates/plix-server/src/crafting/cooldown.rs
- [ ] T066 [P] Unit test: failed craft does not trigger cooldown in crates/plix-server/src/crafting/cooldown.rs

### Implementation for Cooldown

- [ ] T067 Create CraftCooldown struct (next_allowed_tick) in crates/plix-server/src/crafting/cooldown.rs
- [ ] T068 Implement is_ready and trigger methods in crates/plix-server/src/crafting/cooldown.rs
- [ ] T069 Add CraftCooldown to ServerPlayer in crates/plix-server/src/session.rs
- [ ] T070 Integrate cooldown check in try_craft in crates/plix-server/src/crafting/system.rs
- [ ] T071 Trigger cooldown only on successful craft in crates/plix-server/src/lib.rs
- [ ] T072 Export cooldown from crafting module in crates/plix-server/src/crafting/mod.rs

**Checkpoint**: Rate limiting prevents craft spamming

---

## Phase 11: Client Console Command

**Goal**: Players can trigger crafts via `/craft <recipe_id>` console command

**Independent Test**: Type `/craft health_pack` in console → CraftRequest sent to server

### Implementation for Console Command

- [ ] T073 Add /craft command parser in crates/plix-client/src/console.rs (if exists) or lib.rs
- [ ] T074 Send CraftRequest message when command invoked
- [ ] T075 Display CraftResult feedback in console output

**Checkpoint**: Client can trigger crafts via console

---

## Phase 12: Observability & Metrics

**Goal**: Track craft success/failure metrics and log events

**Independent Test**: Verify counters increment and logs appear on craft operations

### Implementation for Metrics

- [ ] T076 Create CraftMetrics struct (crafts_success, crafts_failed by reason) in crates/plix-server/src/crafting/metrics.rs
- [ ] T077 Add CraftMetrics to Server state in crates/plix-server/src/lib.rs
- [ ] T078 Increment success counter on craft success in crates/plix-server/src/lib.rs
- [ ] T079 Increment failure counter with reason on craft failure in crates/plix-server/src/lib.rs
- [ ] T080 Add tracing::info! log on craft success in crates/plix-server/src/lib.rs
- [ ] T081 Add tracing::debug! log on craft failure in crates/plix-server/src/lib.rs
- [ ] T082 Export metrics from crafting module in crates/plix-server/src/crafting/mod.rs

**Checkpoint**: Full observability for crafting operations

---

## Phase 13: Integration Tests

**Goal**: End-to-end tests for complete crafting flows

### Integration Tests

- [ ] T083 [P] Integration test: Training mode full craft flow in crates/plix-server/tests/crafting_test.rs
- [ ] T084 [P] Integration test: BR Lite craft with looted SCRAP in crates/plix-server/tests/crafting_test.rs
- [ ] T085 [P] Integration test: TDM mode crafting disabled in crates/plix-server/tests/crafting_test.rs
- [ ] T086 [P] Integration test: cooldown enforcement in crates/plix-server/tests/crafting_test.rs
- [ ] T087 [P] Integration test: all 3 recipes work correctly in crates/plix-server/tests/crafting_test.rs

---

## Phase 14: Polish & Non-Regression

**Purpose**: Final cleanup and validation

- [ ] T088 Run cargo fmt --all and fix formatting
- [ ] T089 Run cargo clippy and fix warnings
- [ ] T090 Run all plix-server tests to verify no regressions
- [ ] T091 Verify Feature 021 (Hotbar) functionality unchanged
- [ ] T092 Verify Feature 022 (Weapons) functionality unchanged
- [ ] T093 Verify existing game modes (TDM/FFA/CTF) unchanged when crafting disabled

**Checkpoint**: Feature complete, all tests passing, no regressions

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories 1-3 (Phases 3-5)**: All P1 priority, should complete sequentially (core crafting logic)
- **User Stories 4-7 (Phases 6-9)**: P2/P3 priority, can proceed after core crafting works
- **Cooldown (Phase 10)**: Can run parallel to P2/P3 stories
- **Client (Phase 11)**: Requires server-side crafting complete
- **Metrics (Phase 12)**: Can run parallel to client work
- **Integration (Phase 13)**: Requires all features complete
- **Polish (Phase 14)**: Final phase after all implementation

### User Story Dependencies

- **US1 (Simple Crafting)**: Foundation only - MVP standalone
- **US2 (Validation)**: Depends on US1 (needs craft system to validate)
- **US3 (Atomicity)**: Depends on US1 (validates atomic behavior of US1)
- **US4 (Game Mode)**: Can start after US1-3 (adds config layer)
- **US5 (Resources)**: Can start after Foundation (item definitions)
- **US6 (Feedback)**: Depends on US1-2 (needs craft results to send)
- **US7 (Extensibility)**: Can validate after US1 complete

### Parallel Opportunities

Within Phase 2 (Foundational):
```
T006-T009 sequential (recipe types)
T010, T011, T012 parallel (different files)
T013-T015 sequential (hotbar methods)
```

Within Phase 3 (US1 Tests):
```
T017, T018, T019, T020, T021 all parallel (different test functions)
```

Within Phase 10 (Cooldown):
```
T062-T066 all parallel (different test functions)
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 Only)

1. Complete Phase 1: Setup (T001-T005)
2. Complete Phase 2: Foundational (T006-T016)
3. Complete Phase 3: US1 Simple Crafting (T017-T029)
4. Complete Phase 4: US2 Validation (T030-T037)
5. Complete Phase 5: US3 Atomicity (T038-T042)
6. **STOP and VALIDATE**: Core crafting works
7. Run: `cargo test -p plix-server crafting`

### Incremental Delivery

1. MVP (Phases 1-5) → Core crafting functional
2. Add Cooldown (Phase 10) → Anti-spam protection
3. Add Game Mode Config (Phase 6) → Mode-specific rules
4. Add Resources (Phase 7) → SCRAP in loot/loadouts
5. Add Client Command (Phase 11) → Player-facing interface
6. Add Feedback/Metrics (Phases 8, 12) → Observability
7. Add Integration Tests (Phase 13) → Full validation
8. Polish (Phase 14) → Ship ready

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- All 7 user stories are independently testable after foundation
- Tests use existing `cargo test` framework
- Constitution V requires mandatory testing for simulation/network code
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
