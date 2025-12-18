# Research: FFA Arena Mode

**Feature**: 017-ffa-arena | **Date**: 2025-12-16

## Research Summary

This feature has minimal unknowns because it reuses 95%+ of existing TDM infrastructure. The primary research focuses on design decisions for the minimal new functionality required.

## Decision Log

### D1: GameMode Enum Location

**Decision**: Place `GameMode` enum in `plix-common` (shared between crates)

**Rationale**:
- Used by plix-arena (arena config parsing)
- Used by plix-server (kill processing branching)
- Used by plix-common/protocol (client awareness in MatchState)
- Single source of truth prevents version drift

**Alternatives Considered**:
- Define in plix-arena only → requires re-export complexity
- Define in plix-server only → arena crate can't parse config
- Duplicate in each crate → version drift risk

### D2: Arena Config Field Name and Format

**Decision**: Add `game_mode` field to `ArenaMetadata` struct (serde-compatible string)

**Rationale**:
- `ArenaMetadata` already holds arena-level configuration
- TOML format: `game_mode = "ffa"` or `game_mode = "tdm"`
- Default to "tdm" for backward compatibility with existing arenas

**Alternatives Considered**:
- Separate `[game_settings]` section → over-engineering for single field
- Filename convention (e.g., `_ffa.toml`) → brittle, not explicit
- Top-level field outside `[metadata]` → breaks current structure

### D3: Spawn Point Selection for FFA

**Decision**: Use existing spawn points, treat `team` field as ignored (any spawn is neutral)

**Rationale**:
- FFA doesn't have teams, so all spawns are equally valid
- Existing arenas have spawn points defined by team - these work as-is for FFA
- No need for explicit "neutral" spawn markers
- Simple round-robin or random selection among all spawns

**Alternatives Considered**:
- Require separate `[ffa_spawns]` section → extra work, redundant data
- Add `team = null` for neutral spawns → breaking change to existing arenas
- Filter by `team = 255` (magic number) → implicit convention, error-prone

### D4: Scoring Logic Branching

**Decision**: Branch on `game_mode` in kill processing (plix-server/src/lib.rs)

**Rationale**:
- Single branch point keeps logic centralized
- FFA: call `update_player_score` + `check_score_limit` (existing)
- TDM: call `award_team_kill` + `check_team_score_limit` (existing)
- Both paths already implemented, just need conditional

**Alternatives Considered**:
- Strategy pattern (trait object) → over-engineering for 2 modes
- Separate server binaries → deployment complexity
- Runtime mode switching → out of scope, adds state complexity

### D5: Client Awareness of Game Mode

**Decision**: Add `game_mode: GameMode` field to `MatchState` in protocol messages

**Rationale**:
- Client needs to know mode for correct UI (individual vs team scores)
- MatchState already broadcast in WorldSnapshot
- Simple field addition, no new message types

**Alternatives Considered**:
- Separate GameModeInfo message → unnecessary complexity
- Infer from presence of team_winner → fragile, implicit
- Client config override → violates server-authoritative principle

### D6: FFA Default Configuration Values

**Decision**: Use spec-defined defaults: `score_limit=15`, `respawn_delay=180 ticks`, `end_screen=600 ticks`

**Rationale**:
- Spec FR-020/021/022 explicitly define defaults
- Add `MatchConfig::ffa_default()` method alongside existing `tdm_default()`
- Arena config can override these values

**Alternatives Considered**:
- Same defaults as TDM → spec requires different values
- No defaults (require explicit config) → breaks ease-of-use principle

## Best Practices Applied

### Rust Enum for Game Mode

```rust
// plix-common/src/types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    #[default]
    Tdm,
    Ffa,
}
```

- `#[default]` ensures TDM for backward compatibility
- `#[serde(rename_all = "lowercase")]` matches TOML format
- `Copy` trait for cheap passing

### Arena Config Validation

Per FR-026 and constitution V (Code Quality):
- Validate `game_mode` field on arena load
- Error on unknown mode values
- Log warning if FFA arena has no spawn points

### Event-Driven Updates

Per constitution II (Performance):
- Kill scoring is event-driven (on kill event)
- No polling or per-tick score checks
- Match end check is part of kill processing (O(1))

## Dependencies

No new external dependencies required. All functionality uses existing crates:
- `serde` - already used for serialization
- `bincode` - already used for network protocol
- `tracing` - already used for logging

## Resolved Clarifications

| Original Unknown | Resolution |
|-----------------|------------|
| Game mode selection mechanism | Arena config field (user decision from /speckit.clarify) |
| Spawn point handling for FFA | Reuse existing spawns, ignore team field |
| Client UI requirements | Out of scope (spec: no advanced UI) |
| Config defaults | Spec-defined: 15 kills, 3s respawn, 10s end screen |

## Next Steps

Phase 1 will produce:
1. `data-model.md` - Entity definitions and state transitions
2. `contracts/` - Event schemas for FFA-specific events
3. `quickstart.md` - Developer guide for testing FFA mode
