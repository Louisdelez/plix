# Quickstart: Match Flow

**Feature**: 006-match-flow | **Date**: 2025-12-15

## Prerequisites

- Rust 1.75+ installed
- Project builds successfully (`cargo build`)
- Basic understanding of existing match_state.rs and messages.rs

## Implementation Order

Follow this order to ensure dependencies are satisfied:

### Step 1: Protocol Changes (plix-common)

**File**: `crates/plix-common/src/protocol/messages.rs`

1. Update `MatchPhase` enum:
```rust
pub enum MatchPhase {
    Lobby,       // was WaitingForPlayers
    Countdown,   // unchanged
    Playing,     // unchanged
    EndScreen,   // was RoundEnd + MatchEnd
    Resetting,   // new
}
```

2. Add `ReadyToggle` to `ClientMessage`:
```rust
pub enum ClientMessage {
    // ... existing variants ...
    ReadyToggle,
}
```

3. Add new `GameEvent` variants:
```rust
pub enum GameEvent {
    // ... existing variants ...
    MatchPhaseChanged { from: MatchPhase, to: MatchPhase },
    CountdownTick { remaining: u8 },
    ScoreUpdate { player_id: PlayerId, kills: u16, deaths: u16 },
}
```

4. Add `PlayerScore` struct and update `MatchState`

**Verify**: `cargo build -p plix-common`

### Step 2: Server Player Ready State (plix-server)

**File**: `crates/plix-server/src/session.rs`

1. Add `is_ready: bool` field to `ServerPlayer`
2. Initialize to `false` in `ServerPlayer::new()`
3. Add `clear_ready()` method to reset ready state

**Verify**: `cargo test -p plix-server`

### Step 3: Match State Machine (plix-server)

**File**: `crates/plix-server/src/match_state.rs`

1. Update `MatchConfig` with new fields (score_limit, arena_rotation, etc.)
2. Refactor `MatchStateMachine` phases
3. Add ready state aggregation
4. Implement new transition logic

**Key method**:
```rust
pub fn update(&mut self, tick: Tick, ready_count: usize, total_players: usize) -> Option<MatchPhase> {
    match self.state.phase {
        MatchPhase::Lobby => {
            if ready_count == total_players && total_players >= self.config.min_players {
                self.transition_to(MatchPhase::Countdown, tick);
            }
        }
        // ... other phases
    }
}
```

**Verify**: `cargo test -p plix-server`

### Step 4: Phase Restrictions (plix-server)

**File**: `crates/plix-server/src/game_loop.rs` (or equivalent)

1. Check phase before processing combat:
```rust
if match_state.phase != MatchPhase::Playing {
    // Skip damage processing
    return;
}
```

2. Check phase before processing block edits:
```rust
if match_state.phase != MatchPhase::Playing {
    return Err(BlockEditRejectReason::InvalidPhase);
}
```

**Verify**: Manual test - start server, verify no damage in Lobby

### Step 5: Scoring & Match End (plix-server)

**File**: `crates/plix-server/src/scoring.rs` (new file)

1. Create scoring module
2. Integrate with kill processing
3. Check score limit after each kill
4. Check time limit in tick update

**Verify**: `cargo test -p plix-server -- scoring`

### Step 6: Arena Rotation (plix-server)

**File**: `crates/plix-server/src/arena_rotation.rs` (new file)

1. Track current arena index
2. Implement rotation during Resetting phase
3. Load arena from plix-arena

**Verify**: Configure rotation, complete match, verify arena change

### Step 7: Client UI (plix-client)

**Files**:
- `crates/plix-client/src/ui/match_hud.rs` (new)
- `crates/plix-client/src/ui/end_screen.rs` (new)

1. Ready button in Lobby phase
2. Countdown overlay
3. Match timer display
4. Scoreboard
5. End screen with final results

**Verify**: Visual inspection during gameplay

### Step 8: Integration Tests

**File**: `crates/plix-server/tests/match_flow.rs` (new)

1. Full match cycle test
2. Disconnect edge cases
3. Scoring scenarios
4. Arena rotation

**Verify**: `cargo test -p plix-server -- match_flow`

## Testing Commands

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Run match-specific tests
cargo test match_flow
cargo test scoring
cargo test phase

# Run lints
cargo clippy
cargo fmt --check

# Manual testing
# Terminal 1: Start server
cargo run -p plix-server

# Terminal 2: Start client
cargo run -p plix-client
```

## Common Issues

| Issue | Solution |
|-------|----------|
| Protocol mismatch | Ensure plix-common builds first, bump protocol version |
| Phase not transitioning | Check ready state aggregation, min_players config |
| Tests failing | Run `cargo test` after each step, fix before continuing |
| Countdown not starting | Verify all players are ready AND min_players met |

## Definition of Done

- [ ] All 6 implementation steps complete
- [ ] `cargo test` passes (all existing + new tests)
- [ ] `cargo clippy` shows no warnings
- [ ] `cargo fmt --check` passes
- [ ] Manual test: complete full match cycle Lobby → End → Lobby
- [ ] Manual test: arena rotation works (if configured)
- [ ] Manual test: late joiner handled correctly
