# Research: Content / Lore / Campaign (Adventure Mode)

**Feature**: 043-content-lore-campaign
**Date**: 2025-12-20
**Status**: Complete

## Overview

This document consolidates research findings for implementing the Adventure Mode MVP. All technical decisions have been resolved through spec clarifications and codebase analysis.

---

## R1: Content Serialization Format

### Decision
**TOML** for all content definitions.

### Rationale
- Human-readable and easy to edit (Constitution IV: mod-friendly)
- Already used in the project for arena definitions (`assets/arenas/*.toml`)
- Native `toml` crate support in workspace (`toml = "0.8"`)
- Comments supported (useful for content documentation)

### Alternatives Considered
| Format | Pros | Cons | Rejected Because |
|--------|------|------|------------------|
| JSON | Wide support, fast parsing | No comments, verbose | Poor for manual content editing |
| YAML | Comments, clean syntax | Indentation-sensitive, parser complexity | Potential for subtle bugs |
| RON | Rust-native, type-safe | Less familiar to content creators | Barrier to mod community |

---

## R2: Quest Progression Storage

### Decision
Extend existing `plix-server::persist` module for quest progress.

### Rationale
- Feature 014 (World Persistence) provides file-based save/load infrastructure
- `SaveScheduler` already handles periodic saves
- `WorldStore` pattern can be extended for player quest state
- Consistent with existing architecture

### Implementation Approach
```rust
// New file: crates/plix-server/src/quest/progress.rs
pub struct PlayerQuestProgress {
    pub player_id: PlayerId,
    pub active_quests: HashMap<QuestId, QuestStepProgress>,
    pub completed_quests: HashSet<QuestId>,
}

// Persistence via existing WorldStore pattern
impl Persistable for PlayerQuestProgress { ... }
```

### Alternatives Considered
| Approach | Rejected Because |
|----------|------------------|
| SQLite | Adds dependency; overkill for MVP |
| Separate save file | Fragments persistence logic |
| In-memory only | Violates FR-003 (persistence across sessions) |

---

## R3: Mob AI Architecture

### Decision
Simple finite state machine (FSM) with 4 states: Idle, Aggro, Attack, Return.

### Rationale
- Sufficient for MVP behaviors (Aggro, Patrol, Ranged, Boss)
- Easy to extend for future behaviors
- Minimal per-tick CPU cost
- Deterministic (Constitution II: tick stability)

### State Transitions
```
┌─────────┐  player enters aggro radius  ┌─────────┐
│  Idle   │ ────────────────────────────>│  Aggro  │
└─────────┘                               └─────────┘
     ^                                         │
     │                                         │ player in attack range
     │ leash distance exceeded                 v
     │                                    ┌─────────┐
┌─────────┐  target dead/lost            │ Attack  │
│ Return  │<─────────────────────────────└─────────┘
└─────────┘
```

### Boss Phase Handling
- Boss behavior wraps base FSM with phase logic
- Phase transition at HP threshold (e.g., 50%)
- Phase modifies attack patterns, not core FSM

---

## R4: Damage Attribution System

### Decision
Per-mob `DamageTracker` struct tracking contributor damage with timestamps.

### Rationale
- Required for proportional XP/credits (Clarification Q3)
- Enables anti-abuse filtering (5% threshold, 10s window)
- Minimal memory overhead (~32 bytes per contributor)

### Data Structure
```rust
pub struct DamageTracker {
    pub total_damage: u32,
    pub contributors: Vec<DamageContribution>,
}

pub struct DamageContribution {
    pub player_id: PlayerId,
    pub damage: u32,
    pub last_hit_tick: Tick,
}
```

### Payout Formula
```rust
fn calculate_payout(tracker: &DamageTracker, base_xp: u32, killer: PlayerId) -> Vec<(PlayerId, u32)> {
    let eligible: Vec<_> = tracker.contributors.iter()
        .filter(|c| c.damage >= tracker.total_damage / 20 || c.last_hit_tick >= death_tick - 600) // 5% or 10s
        .collect();

    eligible.iter().map(|c| {
        let share = c.damage as f32 / tracker.total_damage as f32;
        let mut xp = (base_xp as f32 * share) as u32;
        if c.player_id == killer {
            xp += (base_xp as f32 * KILLER_BONUS_PCT) as u32; // 10-25%
        }
        (c.player_id, xp)
    }).collect()
}
```

---

## R5: CEF UI Integration

### Decision
Extend existing CEF shell (Feature 030) with quest/dialogue pages.

### Rationale
- CEF infrastructure already in place (`plix-client/src/ui_cef/`)
- Bridge pattern established for Rust<->JS communication
- Consistent with settings and server browser UIs

### New Pages Required
| Page | Purpose | Bridge Messages |
|------|---------|-----------------|
| `quest_log.html` | Quest list with active/completed filter | `QuestListRequest`, `QuestListResponse` |
| `quest_tracker.html` | HUD overlay for pinned quest | `TrackerUpdate` |
| `dialogue.html` | NPC dialogue panel | `DialogueShow`, `DialogueChoice` |
| `dungeon_hud.html` | Dungeon objective overlay | `DungeonStateUpdate` |

### Bridge Message Flow
```
Server                    Client (Rust)              CEF (JS)
   │                           │                        │
   ├─ QuestProgressUpdate ────>│                        │
   │                           ├─ bridge.send_to_js() ─>│
   │                           │                        ├─ updateQuestTracker()
   │                           │<─ DialogueChoice ──────┤
   │<─ DialogueResponse ───────┤                        │
```

---

## R6: Content Validation Strategy

### Decision
Two-mode validator: fail-fast (dev) and skip-with-warning (prod).

### Rationale
- Clarification Q4 explicitly specified this behavior
- Dev mode catches errors early during content iteration
- Prod mode maintains server uptime despite invalid content

### Validation Checks
| Check | Severity | Dev Behavior | Prod Behavior |
|-------|----------|--------------|---------------|
| Unique IDs | Error | Panic | Skip duplicate, warn |
| Valid references (mob_id, item_id, etc.) | Error | Panic | Skip referrer, warn |
| Required fields | Error | Panic | Skip entry, warn |
| Value ranges (HP > 0, respawn > 0) | Warning | Log | Log |
| Schema version mismatch | Error | Panic | Skip file, warn |

### Implementation
```rust
pub enum ValidationMode {
    Development, // fail-fast
    Production,  // skip-with-warning
}

pub struct ContentValidator {
    mode: ValidationMode,
    errors: Vec<ContentError>,
    warnings: Vec<ContentWarning>,
}

impl ContentValidator {
    pub fn validate(&mut self, content: &Content) -> Result<ValidContent, ValidationFailed> {
        // ... validation logic ...
        match self.mode {
            ValidationMode::Development if !self.errors.is_empty() => {
                Err(ValidationFailed(self.errors.clone()))
            }
            ValidationMode::Production => {
                for error in &self.errors {
                    warn!(id = %error.id, reason = %error.reason, "Skipping invalid content");
                }
                Ok(content.filter_valid())
            }
            _ => Ok(content.clone())
        }
    }
}
```

---

## R7: Event System Integration

### Decision
Extend existing `plix-mod-core` event system for quest/mob/dungeon events.

### Rationale
- Feature 034 (Mod API Core) established event infrastructure
- Events already used for chat, combat, etc.
- Consistent with modding architecture

### New Events for Mod API
```rust
// Quest Events
pub enum QuestEvent {
    Started { player_id: PlayerId, quest_id: QuestId },
    StepCompleted { player_id: PlayerId, quest_id: QuestId, step_index: usize },
    Completed { player_id: PlayerId, quest_id: QuestId, rewards: Vec<Reward> },
}

// Mob Events
pub enum MobEvent {
    Spawned { mob_id: MobInstanceId, definition_id: MobDefId, position: Vec3 },
    Killed { mob_id: MobInstanceId, killer: Option<PlayerId>, contributors: Vec<PlayerId> },
    Damaged { mob_id: MobInstanceId, attacker: PlayerId, damage: u32 },
}

// Dungeon Events
pub enum DungeonEvent {
    Entered { player_id: PlayerId, dungeon_id: DungeonId },
    BossKilled { dungeon_id: DungeonId, killers: Vec<PlayerId> },
    Completed { dungeon_id: DungeonId, player_id: PlayerId },
}
```

---

## R8: Spawn System Architecture

### Decision
Region-based spawn management with soft limits and respawn queues.

### Rationale
- FR-015 requires max mobs per region with backpressure
- Existing `plix-arena::SpawnManager` provides pattern
- Queue prevents spawn bursts after mass kills

### Data Structures
```rust
pub struct SpawnSystem {
    spawn_points: HashMap<SpawnId, SpawnPointState>,
    region_limits: HashMap<RegionId, RegionMobLimit>,
    respawn_queue: BinaryHeap<RespawnEntry>, // by tick
}

pub struct RegionMobLimit {
    pub max_mobs: u32,
    pub current_count: u32,
}

pub struct RespawnEntry {
    pub spawn_point_id: SpawnId,
    pub respawn_tick: Tick,
}
```

### Tick Processing
1. Process respawn queue (spawn if region under limit)
2. For each spawn point, check respawn timer
3. If timer elapsed and region has capacity, spawn mob
4. If region at limit, delay spawn (backpressure)

---

## Summary

All technical decisions resolved. No outstanding NEEDS CLARIFICATION items.

| Research Area | Decision | Key Insight |
|---------------|----------|-------------|
| R1: Serialization | TOML | Consistency with existing arena format |
| R2: Quest Storage | Extend persist module | Reuse Feature 014 infrastructure |
| R3: Mob AI | Simple FSM | Sufficient for MVP, easy to extend |
| R4: Damage Attribution | Per-mob tracker | Required for proportional XP |
| R5: CEF UI | Extend existing shell | Consistent with Feature 030+ |
| R6: Validation | Dual-mode validator | Per clarification Q4 |
| R7: Events | Extend mod-core | Per Feature 034 patterns |
| R8: Spawns | Region-based with queue | Backpressure for tick stability |

**Next**: Phase 1 - Generate data-model.md and contracts
