# Quickstart: TDM Arena Mode

**Feature**: 016-tdm-arena
**Date**: 2025-12-16
**Purpose**: Quick validation steps for TDM mode implementation

## Prerequisites

- Rust toolchain (1.75+ stable)
- plix workspace builds: `cargo build --workspace`
- Test arena with team spawn points configured

## Build & Test

### 1. Run All Tests

```bash
# Unit tests for TDM logic
cargo test -p plix-server --lib match_state

# All workspace tests
cargo test --workspace

# Check linting
cargo clippy --workspace --all-targets
```

### 2. Start TDM Server

```bash
# From repo root
cargo run -p plix-server -- \
    --port 7777 \
    --arena test_arena \
    --tickrate 60
```

### 3. Connect Test Clients

```bash
# Terminal 1: Player on Red team
cargo run -p plix-client -- --name "RedPlayer" --connect 127.0.0.1:7777

# Terminal 2: Player on Blue team
cargo run -p plix-client -- --name "BluePlayer" --connect 127.0.0.1:7777
```

## Validation Checklist

### V1: Team Assignment

- [ ] First player joins → assigned to Red (TEAM_0)
- [ ] Second player joins → assigned to Blue (TEAM_1)
- [ ] Third player joins → assigned to team with fewer players
- [ ] Server log shows: `Player connected, team = 0/1`

### V2: Match Start

- [ ] Both players ready up (ReadyToggle)
- [ ] Countdown begins (3 seconds)
- [ ] Phase transitions: Lobby → Countdown → Playing
- [ ] Client shows countdown UI

### V3: Team Scoring

- [ ] Red player kills Blue player
- [ ] Server log: `Team score update: Red = 1, Blue = 0`
- [ ] Client HUD shows Red: 1, Blue: 0
- [ ] Kill awards point to killer's team only
- [ ] No point for friendly fire (if enabled)

### V4: Respawn & Spectate

- [ ] Player dies → enters dead state
- [ ] Camera switches to killer's viewpoint
- [ ] Respawn timer visible (3 seconds)
- [ ] After 3 seconds → respawn at team spawn
- [ ] Camera returns to first-person

### V5: Match End (Score Limit)

- [ ] Kill until score_limit (25 by default, or 5 for quick test)
- [ ] Match ends immediately when team reaches limit
- [ ] Phase: Playing → EndScreen
- [ ] Client shows winning team and final scores

### V6: Auto-Reset

- [ ] EndScreen displays for 15 seconds
- [ ] Phase: EndScreen → Resetting → Lobby
- [ ] Team scores reset to [0, 0]
- [ ] New match ready to begin

### V7: Edge Cases

- [ ] Simultaneous kills: both teams get points
- [ ] Kill during EndScreen: no points awarded
- [ ] Disconnect mid-match: no score for disconnect
- [ ] Killer disconnects while victim dead: victim respawns normally

## Test Arena Configuration

Ensure your test arena has team spawn points:

```toml
# assets/arenas/test_arena.toml

[[spawn_points]]
position = [5.0, 1.0, 5.0]
rotation = 0.0
team = 0  # Red team

[[spawn_points]]
position = [5.0, 1.0, 25.0]
rotation = 3.14159
team = 1  # Blue team
```

## Quick Test: Score Limit = 3

For rapid testing, modify server config:

```rust
// In main.rs or test
let config = ServerConfig {
    // ...
};
let match_config = MatchConfig {
    score_limit: 3,  // Quick test: 3 kills to win
    respawn_delay_ticks: 60,  // 1 second respawn
    end_screen_ticks: 180,  // 3 seconds end screen
    ..MatchConfig::tdm_default()
};
```

## Expected Logs

### Server Startup

```
INFO plix_server: Starting Plix server
INFO plix_server: Arena loaded: test_arena (spawns: 2)
INFO plix_server: Match config: score_limit=25, respawn=3s
```

### Kill Event

```
DEBUG plix_server: Player 1 killed Player 2
DEBUG plix_server: Team score: Red=1, Blue=0
DEBUG plix_server: Player 2 spectating Player 1
DEBUG plix_server: Player 2 respawn at tick 180
```

### Match End

```
INFO plix_server: Team Red reached score limit (25)
INFO plix_server: Match ended: winner=Red, scores=[25, 18]
INFO plix_server: Phase changed: Playing → EndScreen
```

### Auto-Reset

```
INFO plix_server: EndScreen complete, resetting match
INFO plix_server: Phase changed: EndScreen → Resetting
INFO plix_server: World reset complete
INFO plix_server: Phase changed: Resetting → Lobby
```

## Common Issues

### Issue: No team points awarded

**Check**: Is match in Playing phase? Kill only scores during Playing.

### Issue: Wrong team assignment

**Check**: Team balance algorithm assigns to smaller team. If both equal, implementation choice (usually Red).

### Issue: Spectate not working

**Check**: `spectate_target` field added to PlayerSnapshot? Client camera logic implemented?

### Issue: Match not resetting

**Check**: `end_screen_ticks` configured? `complete_reset()` called after Resetting phase?

## Performance Validation

### Target Metrics

- Tick time < 5ms at 60Hz
- O(1) per kill operation
- No world scans during scoring

### Profiling

```bash
# With release build
cargo run -p plix-server --release -- --port 7777

# Monitor tick times in logs (every 60 ticks)
```
