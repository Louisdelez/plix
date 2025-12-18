# Research: Training Mode

**Feature**: 020-training-mode | **Date**: 2025-12-17

## Executive Summary

Research completed for implementing Training Mode sandbox with basic bots. Key decisions: reuse existing GameMode pattern, add Training variant, create isolated training/ module following ctf/ and br_lite/ patterns, use simple state machine for bot behaviors without pathfinding.

---

## Research Tasks

### R1: Bot Entity Architecture

**Question**: How should bots be represented - as full ServerPlayer entities or simplified bot-specific structs?

**Decision**: Use a dedicated `TrainingBot` struct separate from `ServerPlayer`

**Rationale**:
- ServerPlayer carries significant overhead (network addr, anti-cheat state, pending inputs) not needed for bots
- Bots don't need input queuing, network metrics, or ready state
- Simplified struct allows O(1) tick updates without processing irrelevant fields
- Clear separation makes bot code easier to test and extend

**Alternatives Considered**:
- Reuse ServerPlayer with `is_bot: bool` flag - rejected due to unnecessary complexity and field pollution
- Generic Entity trait - overkill for MVP, can be added later if needed

### R2: Bot Behavior Implementation

**Question**: What's the simplest implementation for dummy/roam/strafe behaviors?

**Decision**: Enum-based behavior with per-tick update function

**Rationale**:
```rust
pub enum BotBehavior {
    Dummy,                      // No movement
    Roam { timer: u32, dir: Vec3 }, // Random direction changes
    Strafe { phase: f32, center: Vec3, radius: f32 }, // Oscillating movement
}
```
- Dummy: No update needed, position fixed at spawn
- Roam: Change direction randomly every N ticks (e.g., 60 = 1 second), move at slow speed
- Strafe: Use sin/cos oscillation around center point, simple phase increment

**Alternatives Considered**:
- Trait-based behaviors (`impl BotAI`) - more flexible but over-engineered for 3 simple behaviors
- External behavior scripts - not needed for MVP, pure Rust is simpler

### R3: Integration with Match State Machine

**Question**: How should Training mode integrate with the existing match state machine?

**Decision**: Add `GameMode::Training` variant, skip victory conditions, stay in `Playing` phase indefinitely

**Rationale**:
- Existing match state machine already handles Lobby→Countdown→Playing→EndScreen→Resetting
- Training mode: Lobby→Playing (stays indefinitely, no EndScreen transition)
- No score limit, no time limit, no team scores
- Reset via keyboard triggers soft reset (reposition player/bots, clear stats) without phase change

**Alternatives Considered**:
- Completely separate state machine - unnecessary duplication
- Special "Training" phase - complicates existing code paths

### R4: Stats Tracking Implementation

**Question**: How to track hits and accuracy efficiently?

**Decision**: Dedicated `TrainingStats` struct with server-side tracking

**Rationale**:
```rust
pub struct TrainingStats {
    pub hits: u32,
    pub kills: u32,
    pub attacks: u32,     // For accuracy calculation
    pub session_start: Tick,
}
```
- Increment `attacks` on each attack input (if tracking enabled)
- Increment `hits` on confirmed hit (reuse existing HitConfirmed event)
- Increment `kills` on bot elimination
- Accuracy = hits / attacks (handle div by zero)
- Session duration = current_tick - session_start

**Alternatives Considered**:
- Reuse player kills/deaths counters - doesn't track attacks for accuracy
- Complex per-weapon stats - out of scope for MVP

### R5: Reset Mechanism

**Question**: How to implement keyboard-triggered session reset?

**Decision**: Add new `ClientMessage::TrainingReset` and handle in main server loop

**Rationale**:
- Client detects reset key press, sends `ClientMessage::TrainingReset`
- Server validates (must be in training mode, player exists)
- Server calls `TrainingCoordinator::reset()`:
  1. Reposition player to spawn
  2. Reset player health to 100
  3. Despawn all bots and respawn at initial positions
  4. Clear TrainingStats
- Broadcast `GameEvent::TrainingReset` to confirm

**Alternatives Considered**:
- Chat command `/reset` - requires chat system not yet implemented
- Pause menu button - requires UI changes, keyboard binding is simpler

### R6: Stats Display via Debug Output

**Question**: How to implement keyboard-triggered stats display?

**Decision**: Add `ClientMessage::TrainingStatsRequest`, server logs and optionally sends response

**Rationale**:
- Client detects stats key press, sends `ClientMessage::TrainingStatsRequest`
- Server handles by:
  1. Computing current stats (accuracy, duration)
  2. `info!("Training stats: hits={}, kills={}, accuracy={}%, duration={}s", ...)`
  3. Optionally send `ServerMessage::TrainingStatsResponse` to client
- Client can log to console or display in debug overlay

**Alternatives Considered**:
- Continuous stats in snapshot - adds bandwidth overhead for debug feature
- HUD overlay - requires UI work, debug console is MVP-appropriate

### R7: Bot Spawn Points

**Question**: Where should bots spawn in the training arena?

**Decision**: Reuse existing arena spawn points (team-agnostic) or add optional `bot_spawns` config

**Rationale**:
- Training arenas can define optional `[[bot_spawns]]` array in TOML
- If not present, use regular spawn points filtered by team (or any team for training)
- Bot spawn selection: random from available points, avoid player position
- On respawn: pick different spawn point if possible

**Alternatives Considered**:
- Hardcoded positions - not flexible
- Random world positions - may spawn in walls

---

## Technical Decisions Summary

| Decision | Choice | Justification |
|----------|--------|---------------|
| Bot Entity Type | Dedicated TrainingBot struct | Simpler, no network overhead |
| Behavior Pattern | Enum with per-tick update | Minimal, no trait complexity |
| Match Integration | GameMode::Training variant | Reuses existing state machine |
| Stats Storage | TrainingStats struct | Clean separation, easy reset |
| Reset Trigger | ClientMessage::TrainingReset | Server-authoritative |
| Stats Display | Debug log + optional message | MVP-appropriate, no UI needed |
| Bot Spawns | Arena spawn points (reuse/extend) | Flexible, uses existing system |

---

## Key Patterns from Existing Code

### Pattern 1: Game Mode Coordinator (from CTF/BR Lite)
```rust
// crates/plix-server/src/ctf/mod.rs pattern
pub struct CtfCoordinator {
    state: CtfState,
}

impl CtfCoordinator {
    pub fn tick(&mut self, ...) -> Vec<CtfEvent> { ... }
}
```
Training mode will follow this pattern with `TrainingCoordinator`.

### Pattern 2: GameMode Enum (from types.rs)
```rust
pub enum GameMode {
    Tdm, Ffa, Ctf, BrLite,
    // Add: Training
}
```

### Pattern 3: Match Config Factory (from match_state.rs)
```rust
impl MatchConfig {
    pub fn tdm_default() -> Self { ... }
    pub fn ffa_default() -> Self { ... }
    // Add: training_default() with no score/time limits
}
```

---

## Implementation Notes

1. **Bot Health**: Bots use u8 health (0-100) like players. When `invincibility_bots = true`, hits register but don't reduce health.

2. **Bot Distinction**: Bots should have a visual marker for client rendering. Options:
   - Different team color (e.g., TeamId(255) = "Bot Team")
   - Flag in PlayerSnapshot (add `is_bot: bool`)
   - Separate bot snapshot in training-specific message

3. **Performance**: With 20 bots at 60Hz:
   - 20 position updates per tick = 1200/sec
   - Each update: simple Vec3 addition + bounds check
   - Estimated: < 0.1ms per tick for all bots

4. **Future Extension Points**:
   - BotBehavior enum can add more variants (PatrolPath, ChasePlayer)
   - TrainingConfig can add difficulty levels
   - Stats can expand to per-weapon breakdown
