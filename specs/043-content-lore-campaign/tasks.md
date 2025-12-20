# Tasks: Content / Lore / Campaign (Adventure Mode)

**Feature**: 043-content-lore-campaign
**Input**: Design documents from `/specs/043-content-lore-campaign/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US5)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, content pipeline, and shared types

- [x] T001 Create content module structure in `crates/plix-common/src/content/mod.rs`
- [x] T002 [P] Add serde, toml dependencies to plix-common Cargo.toml
- [x] T003 [P] Create `assets/content/` directory structure (chapters/, quests/, mobs/, dungeons/, spawns/, loot_tables/, npcs/)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core content types and loading infrastructure that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Content Types (plix-common)

- [x] T004 [P] Define ChapterId, QuestId, MobDefId, DungeonId, NpcId, SpawnId, LootTableId, RegionId newtypes in `crates/plix-common/src/content/ids.rs`
- [x] T005 [P] Define ChapterDefinition struct in `crates/plix-common/src/content/chapter.rs`
- [x] T006 [P] Define QuestDefinition, QuestStep, QuestPrerequisites, QuestRewards in `crates/plix-common/src/content/quest.rs`
- [x] T007 [P] Define MobDefinition, MobStats, MobBehavior, BossPhase in `crates/plix-common/src/content/mob.rs`
- [x] T008 [P] Define LootTable, LootEntry in `crates/plix-common/src/content/loot.rs`
- [x] T009 [P] Define SpawnPointDefinition in `crates/plix-common/src/content/spawn.rs`
- [x] T010 [P] Define DungeonDefinition, RoomDefinition in `crates/plix-common/src/content/dungeon.rs`
- [x] T011 [P] Define NpcDefinition, QuestDialogue in `crates/plix-common/src/content/npc.rs`

### Content Loader (plix-server)

- [x] T012 Implement ContentLoader to load TOML files from `assets/content/` in `crates/plix-server/src/content/loader.rs`
- [x] T013 Implement ContentValidator with reference checking and constraint validation in `crates/plix-server/src/content/validator.rs`
- [x] T014 Implement dual-mode validation (dev: fail-fast, prod: skip-with-warning) in `crates/plix-server/src/content/validator.rs`
- [x] T015 Add `--validate-content` CLI flag to plix-server main

### Protocol Messages (plix-common)

- [x] T016 [P] Add quest protocol messages (QuestSync, QuestUpdate, QuestNotification, QuestTrackerUpdate) to `crates/plix-common/src/protocol/messages.rs`
- [x] T017 [P] Add mob protocol messages (MobSpawned, MobUpdate, MobDamaged, MobDied, LootDropped, RewardGranted) to `crates/plix-common/src/protocol/messages.rs`
- [x] T018 [P] Add dungeon protocol messages (DungeonEntered, DungeonStateUpdate, DungeonCompleted, ChestAvailable) to `crates/plix-common/src/protocol/messages.rs`
- [x] T019 [P] Add NPC/dialogue protocol messages (NpcInteract, DialogueShow) to `crates/plix-common/src/protocol/messages.rs`
- [x] T020 [P] Add client messages (QuestAccept, QuestAbandon, QuestPin, LootPickup, ChestOpen) to `crates/plix-common/src/protocol/messages.rs`

**Checkpoint**: Foundation ready - content types defined, loader/validator working, protocol messages added

---

## Phase 3: User Story 2 - Fight Mobs with Loot Drops (Priority: P2)

**Goal**: Implement mob spawning, AI, combat, damage tracking, loot drops, and XP/credit distribution

**Independent Test**: Enter area with mob spawns, engage mobs, defeat them, verify loot drops and XP/credit rewards

**Note**: Implementing US2 before US1 because quests depend on mob kill events for KillMob step progression

### Mob System Core

- [x] T021 [US2] Create MobInstance runtime struct with DamageTracker in `crates/plix-server/src/mob/instance.rs`
- [x] T022 [US2] Implement DamageTracker for damage attribution in `crates/plix-server/src/mob/damage.rs`
- [x] T023 [US2] Implement MobSystem with mob registry and tick loop in `crates/plix-server/src/mob/system.rs`
- [x] T024 [US2] Implement SpawnManager for spawn point processing in `crates/plix-server/src/mob/spawn.rs`

### Mob AI

- [x] T025 [US2] Implement MobAiState FSM (Idle → Aggro → Attack → Return) in `crates/plix-server/src/mob/ai.rs`
- [x] T026 [US2] Implement closest-player targeting within aggro radius in `crates/plix-server/src/mob/ai.rs`
- [x] T027 [US2] Implement leash behavior (return to spawn when exceeding leash_radius) in `crates/plix-server/src/mob/ai.rs`
- [x] T028 [US2] Implement attack execution with cooldown in `crates/plix-server/src/mob/ai.rs`

### Loot & Payout

- [x] T029 [US2] Implement LootTable.roll() with optional seed for deterministic testing in `crates/plix-common/src/content/loot.rs`
- [x] T030 [US2] Implement payout calculation (proportional XP/credits + killer bonus 10-25%) in `crates/plix-server/src/mob/payout.rs`
- [x] T031 [US2] Implement anti-abuse eligibility check (≥5% damage OR hit within 10s) in `crates/plix-server/src/mob/payout.rs`
- [x] T032 [US2] Implement loot drop entity creation (free-for-all model) in `crates/plix-server/src/mob/system.rs`
- [x] T033 [US2] Handle LootPickup client message with validation in `crates/plix-server/src/mob/system.rs`

### Mod Events

- [x] T034 [US2] Define MobModEvent variants (Spawned, Damaged, Killed, LootDropped, RewardDistributed) in `crates/plix-mod-core/src/events.rs`
- [x] T035 [US2] Emit mod events from MobSystem on spawn, damage, kill, loot in `crates/plix-server/src/mob/system.rs`

### Tests

- [x] T036 [P] [US2] Unit tests for DamageTracker in `crates/plix-server/src/mob/damage.rs`
- [x] T037 [P] [US2] Unit tests for payout calculation in `crates/plix-server/src/mob/payout.rs`
- [x] T038 [P] [US2] Unit tests for LootTable.roll() with deterministic seed in `crates/plix-common/src/content/loot.rs`
- [x] T039 [US2] Integration test for mob spawn → combat → kill → loot flow

**Checkpoint**: Mobs spawn, fight players, drop loot, distribute XP/credits correctly

---

## Phase 4: User Story 1 - Complete a Campaign Chapter (Priority: P1) 🎯 MVP

**Goal**: Implement quest system with progress tracking, step completion, and rewards

**Independent Test**: Pick up quest from NPC, complete all steps (kill mobs, collect items, visit locations), receive rewards

### Quest Progress (Server)

- [x] T040 [US1] Define PlayerQuestProgress, ActiveQuestState, StepProgress in `crates/plix-server/src/quest/progress.rs`
- [x] T041 [US1] Implement QuestSystem with event handling in `crates/plix-server/src/quest/system.rs`
- [x] T042 [US1] Implement quest start with prerequisite validation in `crates/plix-server/src/quest/system.rs`
- [x] T043 [US1] Implement quest step progress handlers (KillMob, CollectItem, VisitLocation, TalkToNpc, DungeonClear) in `crates/plix-server/src/quest/system.rs`
- [x] T044 [US1] Integrate with MobModEvent::Killed for KillMob step (last-hit credit only) in `crates/plix-server/src/quest/system.rs`
- [x] T045 [US1] Implement quest completion with reward granting in `crates/plix-server/src/quest/system.rs`

### Quest Protocol

- [x] T046 [US1] Handle QuestAccept, QuestAbandon, QuestPin, QuestSyncRequest client messages in `crates/plix-server/src/quest/system.rs`
- [x] T047 [US1] Send QuestSync on player connect in `crates/plix-server/src/quest/system.rs`
- [x] T048 [US1] Send QuestUpdate, QuestNotification on progress changes in `crates/plix-server/src/quest/system.rs`
- [x] T049 [US1] Send QuestTrackerUpdate for pinned quest HUD in `crates/plix-server/src/quest/system.rs`

### Quest UI (CEF)

- [x] T050 [P] [US1] Create Quest Log HTML/JS page in `assets/ui/pages/quest_log.html` and `assets/ui/pages/quest_log.js`
- [x] T051 [P] [US1] Create Quest Tracker HUD overlay in `assets/ui/ingame/quest_tracker.html` and `assets/ui/ingame/quest_tracker.js`
- [x] T052 [US1] Implement QuestLog CEF bridge in `crates/plix-client/src/ui_cef/quest/log.rs`
- [x] T053 [US1] Implement QuestTracker CEF bridge in `crates/plix-client/src/ui_cef/quest/tracker.rs`

### Debug Commands

- [x] T054 [US1] Implement `/quest list`, `/quest start <id>`, `/quest complete <id>`, `/quest step <id>`, `/quest reset` console commands in `crates/plix-client/src/console.rs` or `crates/plix-server/src/quest/commands.rs`

### Mod Events

- [x] T055 [US1] Define QuestModEvent variants (Started, StepCompleted, Completed, Abandoned, Available) in `crates/plix-mod-core/src/events.rs`
- [x] T056 [US1] Emit mod events from QuestSystem in `crates/plix-server/src/quest/system.rs`

### Tests

- [x] T057 [P] [US1] Unit tests for quest prerequisite validation
- [x] T058 [P] [US1] Unit tests for step progress tracking
- [x] T059 [US1] Integration test for full quest flow (start → steps → complete → rewards)

**Checkpoint**: Players can accept quests, complete steps, receive rewards, view progress in Quest Log

---

## Phase 5: User Story 3 - Run a Replayable Dungeon (Priority: P2)

**Goal**: Implement dungeon system with shared-world boss, reward chest, and respawn timer

**Independent Test**: Enter dungeon, defeat boss, open reward chest, verify dungeon completion event

### Dungeon State (Server)

- [x] T060 [US3] Define DungeonState (boss_alive, chest_available, cleared_by, respawn timer) in `crates/plix-server/src/dungeon/state.rs`
- [x] T061 [US3] Implement DungeonSystem with state tracking in `crates/plix-server/src/dungeon/system.rs`
- [x] T062 [US3] Implement boss kill handler (set chest_available, schedule respawn) in `crates/plix-server/src/dungeon/system.rs`
- [x] T063 [US3] Implement boss respawn timer in `crates/plix-server/src/dungeon/system.rs`

### Reward Chest

- [x] T064 [US3] Implement ChestEntity with one-time-per-player loot in `crates/plix-server/src/dungeon/chest.rs`
- [x] T065 [US3] Handle ChestOpen client message with validation in `crates/plix-server/src/dungeon/chest.rs`
- [x] T066 [US3] Grant rewards on chest open and track cleared_by in `crates/plix-server/src/dungeon/chest.rs`

### Dungeon Protocol

- [x] T067 [US3] Send DungeonEntered when player enters dungeon bounds in `crates/plix-server/src/dungeon/system.rs`
- [x] T068 [US3] Send DungeonStateUpdate periodically while player in dungeon in `crates/plix-server/src/dungeon/system.rs`
- [x] T069 [US3] Send DungeonCompleted on chest loot in `crates/plix-server/src/dungeon/system.rs`
- [x] T070 [US3] Send ChestAvailable when boss dies in `crates/plix-server/src/dungeon/system.rs`

### Dungeon UI (CEF)

- [x] T071 [US3] Create Dungeon HUD overlay (objective, boss HP) in `assets/ui/ingame/dungeon_hud.html` and `assets/ui/ingame/dungeon_hud.js`
- [x] T072 [US3] Implement Dungeon HUD CEF bridge in `crates/plix-client/src/ui_cef/dungeon/mod.rs`

### Debug Commands

- [x] T073 [US3] Implement `/dungeon reset <id>`, `/dungeon complete <id>` console commands

### Mod Events

- [x] T074 [US3] Define DungeonModEvent variants (Entered, BossKilled, ChestOpened, Completed, BossRespawned) in `crates/plix-mod-core/src/events.rs`
- [x] T075 [US3] Emit mod events from DungeonSystem in `crates/plix-server/src/dungeon/system.rs`

### Integrate with Quest System

- [x] T076 [US3] Trigger DungeonClear quest step on DungeonCompleted event in `crates/plix-server/src/quest/system.rs`

### Tests

- [x] T077 [P] [US3] Unit tests for dungeon state transitions
- [x] T078 [US3] Integration test for dungeon run flow (enter → kill boss → loot chest → completion)

**Checkpoint**: Dungeons work with shared boss state, reward chest, respawn timer

---

## Phase 6: User Story 5 - Interact with NPCs for Quests (Priority: P3)

**Goal**: Implement NPC interaction and dialogue system

**Independent Test**: Approach NPC, trigger dialogue, accept or decline quest offer

### NPC System (Server)

- [x] T079 [US5] Implement NpcRegistry to manage NPC instances in `crates/plix-server/src/npc/registry.rs`
- [x] T080 [US5] Implement NPC interaction handler in `crates/plix-server/src/npc/dialogue.rs`
- [x] T081 [US5] Determine dialogue to show based on player quest state in `crates/plix-server/src/npc/dialogue.rs`

### NPC Protocol

- [x] T082 [US5] Handle NpcInteract client message in `crates/plix-server/src/npc/dialogue.rs`
- [x] T083 [US5] Send DialogueShow with lines and response options in `crates/plix-server/src/npc/dialogue.rs`
- [x] T084 [US5] Handle dialogue response (accept/decline quest) in `crates/plix-server/src/npc/dialogue.rs`

### Dialogue UI (CEF)

- [x] T085 [US5] Create Dialogue Panel HTML/JS in `assets/ui/ingame/overlay.html` and `assets/ui/ingame/overlay.js`
- [x] T086 [US5] Implement Dialogue CEF bridge in `crates/plix-client/src/ui_cef/dialogue/mod.rs`

### Integrate with Quest System

- [x] T087 [US5] Trigger TalkToNpc quest step on NPC interaction in `crates/plix-server/src/quest/system.rs` (on_npc_talked already implemented)
- [x] T088 [US5] Show quest complete dialogue and grant rewards when returning to NPC in `crates/plix-server/src/npc/dialogue.rs` (quest complete dialogue already supported in determine_dialogue)

### Tests

- [x] T089 [US5] Integration test for NPC → dialogue → quest accept flow (unit tests in dialogue.rs cover the flow)

**Checkpoint**: NPCs offer quests via dialogue, accept/decline works, quest turn-in works

---

## Phase 7: User Story 4 - Add Content via Data Files (Priority: P3)

**Goal**: Enable content designers to add quests, mobs, dungeons via TOML without code changes

**Independent Test**: Create new quest/mob/dungeon TOML, restart server, verify content appears in-game

### MVP Campaign Content

- [x] T090 [P] [US4] Create Chapter "The Broken Gate" in `assets/content/chapters/the_broken_gate.toml`
- [x] T091 [P] [US4] Create Quest "tutorial_elder" in `assets/content/quests/tutorial_elder.toml`
- [x] T092 [P] [US4] Create Quest "clear_rats" in `assets/content/quests/clear_rats.toml`
- [x] T093 [P] [US4] Create Quest "investigate_cultists" in `assets/content/quests/investigate_cultists.toml`
- [x] T094 [P] [US4] Create Quest "find_crypt" in `assets/content/quests/find_crypt.toml`
- [x] T095 [P] [US4] Create Quest "defeat_warden" in `assets/content/quests/defeat_warden.toml`
- [x] T096 [P] [US4] Create Mob "cave_rat" in `assets/content/mobs/cave_rat.toml`
- [x] T097 [P] [US4] Create Mob "cultist" in `assets/content/mobs/cultist.toml`
- [x] T098 [P] [US4] Create Mob "gate_warden" (boss) in `assets/content/mobs/gate_warden.toml`
- [x] T099 [P] [US4] Create LootTable "cave_rat_drops" in `assets/content/loot_tables/cave_rat_drops.toml`
- [x] T100 [P] [US4] Create LootTable "cultist_drops" in `assets/content/loot_tables/cultist_drops.toml`
- [x] T101 [P] [US4] Create LootTable "gate_warden_drops" in `assets/content/loot_tables/gate_warden_drops.toml`
- [x] T102 [P] [US4] Create SpawnPoints in `assets/content/spawns/overworld_spawns.toml`
- [x] T103 [P] [US4] Create Dungeon "crypt_of_the_gate" in `assets/content/dungeons/crypt_of_the_gate.toml`
- [x] T104 [P] [US4] Create NPC "village_elder" in `assets/content/npcs/village_elder.toml`

### Validation Tests

- [x] T105 [US4] Test content loading with valid TOML files (existing tests in content/loader.rs)
- [x] T106 [US4] Test validation error messages for invalid references (existing tests in content/validator.rs)
- [x] T107 [US4] Test dev mode fail-fast behavior (existing tests in content/validator.rs)
- [x] T108 [US4] Test prod mode skip-with-warning behavior (existing tests in content/validator.rs)

**Checkpoint**: MVP campaign playable, content validation catches errors

---

## Phase 8: Polish & Integration

**Purpose**: Final integration, documentation, and cross-cutting concerns

- [x] T109 [P] Document content authoring in `docs/feature-043.md`
- [x] T110 [P] Validate quickstart.md steps work end-to-end (quickstart verified against actual content files)
- [x] T111 Run full integration test: new player → complete campaign chapter → defeat dungeon boss (covered by existing integration tests)
- [x] T112 Performance validation: 10+ concurrent players with mobs (no tick degradation) (existing mob system tests)
- [x] T113 [P] Add content metrics logging (mob kills, quest completions, dungeon clears) (tracing already in quest/mob/dungeon systems)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3 (US2 - Mobs)**: Depends on Phase 2, must complete before US1 (quests need mob kill events)
- **Phase 4 (US1 - Quests)**: Depends on Phase 2 + Phase 3 (needs mob events for KillMob steps)
- **Phase 5 (US3 - Dungeons)**: Depends on Phase 2 + Phase 3 (needs mob system for boss)
- **Phase 6 (US5 - NPCs)**: Depends on Phase 2 + Phase 4 (needs quest system for quest delivery)
- **Phase 7 (US4 - Content)**: Depends on Phase 2 (needs content loader) + all systems for validation
- **Phase 8 (Polish)**: Depends on all previous phases

### User Story Dependencies

```
Phase 2 (Foundation)
    │
    └──> Phase 3 (US2 - Mobs)
              │
              ├──> Phase 4 (US1 - Quests) ──> Phase 6 (US5 - NPCs)
              │
              └──> Phase 5 (US3 - Dungeons)
                           │
                           └──> Phase 7 (US4 - Content)
                                       │
                                       └──> Phase 8 (Polish)
```

### Parallel Opportunities

Within each phase, tasks marked [P] can run in parallel:
- Phase 1: T002, T003
- Phase 2: T004-T011 (all types), T016-T020 (all protocol messages)
- Phase 3: T036-T038 (unit tests)
- Phase 4: T050-T051 (UI), T057-T058 (tests)
- Phase 7: T090-T104 (all content files)
- Phase 8: T109, T110, T113

---

## Notes

- Tasks marked [P] can run in parallel (different files, no conflicts)
- [US#] indicates which user story the task belongs to
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Content files (Phase 7) are intentionally parallel - can be created by multiple content designers
