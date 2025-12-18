# Quickstart: FFA Arena Mode

**Feature**: 017-ffa-arena | **Date**: 2025-12-16

## Prerequisites

- Rust 1.75+ (stable channel)
- Existing plix workspace builds successfully
- Familiarity with existing TDM implementation

## Quick Test

After implementation, verify FFA mode works:

```bash
# 1. Build the project
cargo build --release

# 2. Run tests (should include new FFA tests)
cargo test

# 3. Start server with FFA arena
cargo run --release --bin plix-server -- --arena assets/arenas/ffa_arena.toml

# 4. In another terminal, start client
cargo run --release --bin plix-client

# 5. Connect second client and test kills
#    - Verify individual scores increase
#    - Verify winner declared at score_limit
```

## Creating an FFA Arena

### Minimal FFA Arena Config

Create `assets/arenas/my_ffa.toml`:

```toml
[metadata]
name = "My FFA Arena"
version = "1.0.0"
size = [64, 32, 64]
game_mode = "ffa"    # This makes it FFA mode

# Spawn points (team field is ignored for FFA)
[[spawn_points]]
team = 0
position = [10.0, 1.0, 10.0]
rotation = 0.0

[[spawn_points]]
team = 0
position = [32.0, 1.0, 32.0]
rotation = 90.0

[[spawn_points]]
team = 0
position = [54.0, 1.0, 54.0]
rotation = 180.0

[[spawn_points]]
team = 0
position = [32.0, 1.0, 54.0]
rotation = 270.0

[blocks]
floor = { y = 0, block = "stone" }
walls = { border = true, height = 8, block = "brick" }
```

### Key Points

1. **game_mode = "ffa"**: Required field to enable FFA scoring
2. **Spawn points**: Team field is ignored, all spawns are neutral
3. **More spawns**: FFA benefits from many spawn points spread across arena

## Configuration Overrides

FFA uses defaults from spec, but you can override:

```toml
[metadata]
name = "Custom FFA"
version = "1.0.0"
size = [64, 32, 64]
game_mode = "ffa"

# Optional: Override defaults
score_limit = 20        # Default: 15
respawn_delay = 5.0     # Default: 3.0 seconds
end_screen_delay = 15.0 # Default: 10.0 seconds
```

## Code Changes Summary

### Files Modified

| File | Change |
|------|--------|
| `plix-common/src/types.rs` | Add `GameMode` enum |
| `plix-arena/src/format.rs` | Add `game_mode` to `ArenaMetadata` |
| `plix-common/src/protocol/messages.rs` | Add `game_mode` to `MatchState` |
| `plix-server/src/match_state.rs` | Add `ffa_default()` constructor |
| `plix-server/src/lib.rs` | Branch on game_mode in kill processing |
| `assets/arenas/test_arena.toml` | Add `game_mode = "tdm"` for explicit mode |

### Files Added

| File | Description |
|------|-------------|
| `assets/arenas/ffa_arena.toml` | Example FFA arena |

## Testing FFA Mode

### Unit Tests

```bash
# Run all match_state tests including new FFA tests
cargo test -p plix-server match_state

# Expected new tests:
# - test_ffa_kill_increments_player_score
# - test_ffa_suicide_no_score
# - test_ffa_score_limit_ends_match
# - test_ffa_winner_is_player_id
```

### Integration Tests

```bash
# Run integration tests
cargo test -p plix-server integration

# Expected scenarios:
# - FFA match flow: lobby → playing → endscreen → reset
# - Multiple players competing for kills
# - Winner declared correctly
```

### Manual Testing

1. Start server with FFA arena
2. Connect 2+ clients
3. Kill another player → verify your score increases
4. Kill yourself (fall damage) → verify no score change
5. Reach score_limit → verify match ends with you as winner
6. Wait for end_screen → verify reset to lobby

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --bin plix-server
```

### Key Log Messages

```
INFO  Match initialized with game_mode=Ffa, score_limit=15
INFO  Kill: player_1 eliminated player_2 (FFA: +1 score)
INFO  Player player_1 reached score_limit (15), match ending
INFO  Match phase transition: Playing -> EndScreen
INFO  Match winner: player_1
```

## Common Issues

### Issue: Arena loads but uses TDM scoring

**Cause**: Missing or misspelled `game_mode` field
**Solution**: Ensure `game_mode = "ffa"` is in `[metadata]` section

### Issue: Spawn points not working

**Cause**: FFA arena has no spawn points defined
**Solution**: Add at least 1 spawn point (team field doesn't matter)

### Issue: Score not incrementing

**Cause**: Match not in Playing phase
**Solution**: Wait for countdown to complete, check phase in logs

## Next Steps

After FFA is working:

1. Create diverse FFA arena layouts
2. Tune score_limit for desired match duration
3. Test with 4+ players for gameplay balance
4. Consider adding FFA-specific UI elements (out of scope for this feature)
