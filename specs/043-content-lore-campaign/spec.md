# Feature Specification: Content / Lore / Campaign (Adventure Mode)

**Feature Branch**: `043-content-lore-campaign`
**Created**: 2025-12-20
**Status**: Draft
**Input**: User description: "Open an Adventure branch with quest system, mobs, dungeons, and lore - a coherent MVP focused on data-driven extensibility and mod compatibility"

## Clarifications

### Session 2025-12-20

- Q: Are dungeons shared world locations or instanced per-player? → A: Shared world - single dungeon location, all players share boss state, first kill is global
- Q: How is loot distributed when multiple players are present? → A: Free-for-all - single loot drop, first player to pick up gets it
- Q: How is quest/XP/credit distributed in multiplayer kills? → A: Last-hit only for quest KillMob credit; XP/credits proportional to damage dealt with last-hit bonus (10-25%); anti-abuse threshold (min 5% damage or hit within 10s); CollectItem strictly individual
- Q: What happens when content validation fails in production? → A: Skip invalid content with warning logs, server continues with valid content only (dev mode still fails fast)
- Q: How does a mob choose its target when multiple players are in aggro range? → A: Closest player - always target nearest player in range

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Complete a Campaign Chapter (Priority: P1)

As a player, I want to follow a short campaign (1 chapter with 3-5 quests) that leads me to a dungeon and boss fight, so that I experience a coherent adventure with clear progression and rewards.

**Why this priority**: This is the core experience that validates the entire quest-mob-dungeon integration. Without a working campaign loop, the individual systems have no context.

**Independent Test**: Can be fully tested by starting the game, picking up the first quest from an NPC, completing all quest steps (kill mobs, visit locations, collect items), entering the dungeon, defeating the boss, and receiving rewards. Delivers the complete adventure experience.

**Acceptance Scenarios**:

1. **Given** a new player enters the game world, **When** they approach the quest-giver NPC, **Then** they see a quest offer with title, description, and Accept/Decline options
2. **Given** a player accepts a quest, **When** they open the Quest Log, **Then** they see the quest listed as active with all steps and current progress
3. **Given** a player completes all required quest steps, **When** they return to the NPC or reach the objective, **Then** they receive quest rewards (XP, items, currency) and see completion notification
4. **Given** a player enters the dungeon portal, **When** they defeat the boss and open the reward chest, **Then** the dungeon completion event fires and quest progress updates

---

### User Story 2 - Fight Mobs with Loot Drops (Priority: P2)

As a player, I want to encounter various mobs in the world with different behaviors, defeat them in combat, and receive loot drops, so that I have engaging combat encounters and progression incentives.

**Why this priority**: Mobs are the primary interactive content that populates the world. Without mobs, quests like "Kill X" cannot function and the world feels empty.

**Independent Test**: Can be fully tested by entering an area with mob spawns, engaging mobs in combat, defeating them, and verifying loot appears. Delivers combat engagement independently.

**Acceptance Scenarios**:

1. **Given** a player enters a region with mob spawns, **When** they move within the aggro radius of a mob, **Then** the mob detects and pursues the player
2. **Given** a mob is aggroed on a player, **When** the mob reaches attack range, **Then** it attacks using its defined behavior (melee, ranged, or boss pattern)
3. **Given** a player kills a mob, **When** the mob dies, **Then** loot drops according to the mob's loot table and the player's kill is attributed correctly for quest progress
4. **Given** a mob moves too far from its spawn point, **When** it exceeds the leash distance, **Then** it returns to spawn and resets aggro

---

### User Story 3 - Run a Replayable Dungeon (Priority: P2)

As a player, I want to enter and complete dungeons that have rooms, a boss, and reward chests, so that I can repeatedly challenge myself and earn rewards.

**Why this priority**: Dungeons provide the climax of the campaign and endgame replayability. They integrate mobs and quests into a contained, focused experience.

**Independent Test**: Can be fully tested by entering the dungeon portal, navigating through 3-5 rooms, defeating the boss, opening the reward chest, and exiting. Delivers a complete dungeon run.

**Acceptance Scenarios**:

1. **Given** a player approaches a dungeon entrance portal, **When** they interact with it, **Then** they are transported to the dungeon entry room
2. **Given** a player is in a dungeon, **When** they view the HUD, **Then** they see dungeon name, objective (e.g., "Defeat the Gate Warden"), and boss status (alive/dead)
3. **Given** a player defeats the dungeon boss, **When** the boss dies, **Then** a reward chest appears or becomes interactable
4. **Given** a player opens the reward chest, **When** they loot it, **Then** they receive rewards based on the dungeon's loot table and the dungeon_completed event fires

---

### User Story 4 - Add Content via Data Files (Priority: P3)

As a content designer, I want to add new quests, mobs, and dungeons by creating data files without modifying code, so that I can rapidly iterate on content.

**Why this priority**: Extensibility is key for long-term content growth and mod support. This enables the community and designers to contribute without programming knowledge.

**Independent Test**: Can be fully tested by creating a new quest/mob/dungeon TOML file, restarting the server, and verifying the new content appears in-game. Delivers content authoring capability.

**Acceptance Scenarios**:

1. **Given** a designer creates a new quest TOML file in `assets/content/quests/`, **When** the server loads, **Then** the quest is available to players matching its prerequisites
2. **Given** a designer creates a new mob definition, **When** they reference it in a spawn point, **Then** mobs of that type spawn at the specified locations
3. **Given** a data file has invalid references (e.g., nonexistent mob_id), **When** the server loads, **Then** a clear error message identifies the issue and the server fails fast in dev mode
4. **Given** a designer modifies a data file, **When** using the content validator tool, **Then** they receive validation results before runtime

---

### User Story 5 - Interact with NPCs for Quests (Priority: P3)

As a player, I want to talk to NPCs who offer quests and provide dialogue, so that I receive narrative context and quest information.

**Why this priority**: NPCs are the interface between players and the quest system, providing lore context and quest delivery mechanism.

**Independent Test**: Can be fully tested by approaching an NPC, triggering dialogue, reading their lines, and accepting or declining a quest. Delivers NPC interaction independently.

**Acceptance Scenarios**:

1. **Given** a player approaches a quest-giver NPC, **When** they interact, **Then** a dialogue panel appears with NPC's lines and response options
2. **Given** an NPC offers a quest, **When** the player selects "Accept", **Then** the quest is added to their active quests and the dialogue closes
3. **Given** an NPC offers a quest, **When** the player selects "Decline", **Then** the dialogue closes and the quest remains available for later
4. **Given** a player has completed a quest for an NPC, **When** they interact again, **Then** the NPC shows completion dialogue and provides rewards

---

### Edge Cases

- What happens when a player disconnects mid-quest? Progress must persist on reconnection.
- What happens when a mob dies but no player dealt damage (e.g., fall damage, environment)? No player is credited.
- What happens when a player deals minimal damage (1 hit) to "tag" a mob? Anti-abuse rules apply: must deal >=5% damage OR hit within 10s of death to receive XP/credits.
- What happens when a player's inventory is full when receiving loot? Loot drops to ground or is held in overflow.
- What happens when quest prerequisites change after a player started the quest chain? Active quests remain valid.
- What happens when a dungeon boss is killed while multiple players are present? All eligible players receive credit and rewards.
- What happens when spawn points exceed the region's mob limit? New spawns are delayed until slots free up.

## Requirements *(mandatory)*

### Functional Requirements

#### Quest System

- **FR-001**: System MUST support quest definitions with stable ID, title, descriptions, chapter association, prerequisites, ordered steps, rewards, and repeatable flag
- **FR-002**: System MUST support at least 5 quest step types: CollectItem, KillMob, VisitLocation, TalkToNpc, DungeonClear
- **FR-003**: System MUST store player quest progress server-side (ActiveQuests, CompletedQuests, QuestStepProgress) with persistence across sessions
- **FR-004**: System MUST validate quest progress server-side: KillMob credit to last-hit player only, CollectItem to collecting player only, items checked against server inventory
- **FR-005**: System MUST emit events: on_quest_started, on_step_completed, on_quest_completed
- **FR-006**: System MUST provide Quest Log UI showing active/completed quests with steps and progress
- **FR-007**: System MUST provide Quest Tracker HUD showing pinned quest with current step
- **FR-008**: System MUST display notifications for quest started, step completed, quest completed
- **FR-009**: System MUST provide debug commands: `/quest list`, `/quest start <id>`, `/quest complete <id>`, `/quest reset`

#### Mob System

- **FR-010**: System MUST support mob definitions with stable ID, display name, stats (HP, damage, speed, armor), behavior type, loot table, XP reward, and tags
- **FR-011**: System MUST support at least 4 behavior types: Aggro (basic melee pursuit), Patrol, Ranged, Boss (with phases)
- **FR-012**: System MUST implement mob AI server-side with perception (aggro radius, closest-player targeting), pathfinding/steering, attack execution, and leash return
- **FR-013**: System MUST support Boss behavior with at least one phase transition (e.g., special attack below 50% HP)
- **FR-014**: System MUST support spawn point definitions with mob type, count, respawn timer, radius, and region limits
- **FR-015**: System MUST enforce max mobs per region with backpressure (delay spawns when at limit)
- **FR-016**: System MUST handle loot drops server-authoritatively using free-for-all model: single drop per kill, first player to collect receives item
- **FR-017**: System MUST emit events: on_mob_spawned, on_mob_killed with attribution to killing player
- **FR-017a**: System MUST distribute XP/credits proportionally to damage dealt (damage_share = player_damage / total_damage), with last-hit bonus of 10-25%
- **FR-017b**: System MUST enforce anti-abuse eligibility: player must deal >=5% total damage OR hit within 10 seconds of death to receive XP/credits
- **FR-018**: System MUST track metrics: mob kills, drops, spawn counts

#### Dungeon System

- **FR-019**: System MUST support dungeon definitions with stable ID, display name, difficulty, rooms list, boss mob ID, entry location, completion criteria, and rewards
- **FR-020**: System MUST support dungeons with 3-5 rooms (MVP: prefab/fixed layout in world)
- **FR-021**: System MUST track dungeon state in shared world model: boss alive/dead globally, completion credit given to all present players on kill, boss respawns on configurable timer
- **FR-022**: System MUST trigger reward chest access upon boss defeat
- **FR-023**: System MUST emit event: on_dungeon_completed when player defeats boss and loots chest
- **FR-024**: System MUST provide Dungeon HUD showing title, objective, and boss status
- **FR-025**: System MUST display dungeon completion notification with rewards summary

#### Lore & Campaign

- **FR-026**: System MUST support chapter definitions with stable ID, title, intro/outro text, and quest lists (mainline and optional side quests)
- **FR-027**: System MUST support NPC definitions with stable ID, name, location, and quest-giver flags
- **FR-028**: System MUST support NPC dialogue with 2-5 lines and Accept/Decline response options
- **FR-029**: System MUST display dialogue via CEF-based dialogue panel

#### Content System

- **FR-030**: All content (quests, mobs, dungeons, NPCs, chapters) MUST be defined in data files (TOML format)
- **FR-031**: System MUST load content from `assets/content/` directory with subdirectories: quests/, mobs/, dungeons/, npcs/, chapters/
- **FR-032**: System MUST validate content on load: unique IDs, valid references, required fields
- **FR-033**: System MUST fail fast with clear error messages when content validation fails in development mode; in production mode, skip invalid content with warning logs and continue with valid content only
- **FR-034**: System MUST support deterministic loot rolls when seed is provided (for testing)

#### Mod Integration

- **FR-035**: System MUST expose mod events for: quest lifecycle, dungeon completion, mob spawns/kills
- **FR-036**: Content definitions MUST be loadable from mod directories (deferred to stable mod distribution)

### Key Entities

- **Quest**: Represents a player objective with ordered steps, prerequisites, and rewards. Linked to chapters and NPCs.
- **QuestStep**: A single objective within a quest (CollectItem, KillMob, VisitLocation, TalkToNpc, DungeonClear) with progress tracking.
- **QuestProgress**: Per-player state tracking active quests, completed quests, and step progress. Server-authoritative.
- **Mob**: An enemy entity with stats, behavior, loot table, and spawn rules. Server-controlled.
- **MobInstance**: A spawned mob in the world with current HP, position, aggro target, and leash origin.
- **SpawnPoint**: Defines where and how mobs spawn (type, count, respawn timer, region limits).
- **LootTable**: Defines drop chances for items when a mob dies or chest is opened.
- **Dungeon**: A contained combat area with rooms, boss, and rewards. Tracked per-player for completion.
- **NPC**: A non-hostile entity that provides dialogue and quests. Fixed location in world.
- **Chapter**: A narrative container grouping quests into a campaign segment with intro/outro text.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can complete the MVP campaign chapter (3-5 quests leading to dungeon boss) in a single session
- **SC-002**: Quest progress persists correctly across player reconnections with zero data loss
- **SC-003**: Mobs respond to player presence within 0.5 seconds of entering aggro range
- **SC-004**: Boss encounters support at least one phase transition based on HP threshold
- **SC-005**: Content designers can add a new quest/mob/dungeon by creating data files without code changes
- **SC-006**: Content validation catches 100% of invalid references (missing IDs) and provides actionable error messages
- **SC-007**: Dungeon runs are completable within 10-15 minutes for average gameplay
- **SC-008**: Loot drops occur consistently according to defined loot table probabilities (verified with deterministic seeds)
- **SC-009**: System supports 10+ concurrent players engaging mobs without server tick degradation
- **SC-010**: All unit and integration tests pass before feature merge

## Assumptions

- The existing inventory system (Feature 021) is functional and can be used for item rewards and loot
- The existing combat system (Feature 003) provides damage, death, and respawn mechanics compatible with mob combat
- The CEF UI shell (Feature 030+) is available for dialogue panels, quest log, and HUD elements
- The mod event system (Feature 034) exists for exposing quest/mob/dungeon events
- Server-side persistence (Feature 014) is available for quest progress storage
- The game world supports portal/trigger zones for dungeon entries
- Default mob respawn timers: overworld 60-120 seconds, boss 10-15 minutes (or per dungeon run)

## MVP Content Defaults

The following content will be included as the initial playable campaign:

- **Chapter**: "The Broken Gate" - introduction to the adventure mode
- **Quests** (3-5 mainline + 1 side):
  1. Talk to the Village Elder (tutorial quest)
  2. Clear the cave rats from the mine entrance
  3. Investigate the cultist activity
  4. Find the entrance to the Crypt
  5. Defeat the Gate Warden (dungeon boss)
- **Mobs** (3 types):
  - Cave Rat: Low HP trash mob, melee aggro, common drops
  - Cultist: Medium HP, ranged attacks, uncommon drops
  - Gate Warden: Boss with phase transition, guaranteed drops
- **Dungeon**: "Crypt of the Gate" - 3-5 rooms leading to Gate Warden boss

## Out of Scope

- Full 10+ hour campaign content
- Cinematic cutscenes
- Complex branching dialogue trees
- Multi-group dungeon instances (infrastructure dependent)
- In-game content editor
- Localization of content text
