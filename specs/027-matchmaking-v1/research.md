# Research: Matchmaking v1 (Quick Join)

**Feature**: 027-matchmaking-v1
**Date**: 2025-12-17

## Research Summary

This feature builds entirely on existing infrastructure. No external research or new dependencies required.

## Decisions

### D1: Server Scoring Algorithm Weights

**Decision**: Use fixed additive scoring with predefined weights from spec FR-009 to FR-011.

**Rationale**: Simple additive scoring is deterministic, testable, and meets the <100ms performance target. The weights are specified in the requirements and can be tuned later without architectural changes.

**Alternatives Considered**:
- Multiplicative scoring: More complex, harder to debug, no clear benefit for v1
- ML-based scoring: Out of scope per IX. Scoping, adds unnecessary complexity

**Scoring Formula**:
```
total_score =
    region_bonus (if region matches: +50)
    + capacity_bonus (if 1-80% full: +30)
    + freshness_bonus (if last_seen < 30s ago: +20)
    + player_bonus (+1 per player, up to 80% capacity)
    + ping_bonus (optional: +40 if <50ms, +20 if <100ms)
```

### D2: Profile Extension Strategy

**Decision**: Extend existing `PlayerProfile` struct with optional `MatchmakingPreferences` field.

**Rationale**: Feature 025 already provides atomic TOML save/load via `profile.toml`. Adding a `[matchmaking]` section is a minimal change that preserves backward compatibility with existing profiles.

**Alternatives Considered**:
- Separate matchmaking.toml file: Adds file management complexity, splits user preferences across files
- Embed in config.toml (Feature 005): That file is for UI settings, not player preferences

**Implementation**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchmakingPreferences {
    #[serde(default = "default_mode")]
    pub preferred_mode: String,  // "tdm", "ffa", etc.
    #[serde(default = "default_region")]
    pub preferred_region: String,  // "any", "eu", "us", "asia"
}

// Add to PlayerProfile:
#[serde(default)]
pub matchmaking: MatchmakingPreferences,
```

### D3: Server List Fetching Strategy

**Decision**: Reuse existing `BrowserState::refresh()` for async fetch, add blocking variant for game loop.

**Rationale**: Feature 026 already implemented server list fetching. The matchmaking module calls `BrowserState::servers()` to get cached entries after refresh.

**Alternatives Considered**:
- Duplicate fetch logic in matchmaking module: Violates DRY, creates maintenance burden
- Always async with tokio spawn: More complex for blocking game loop context

### D4: Tie-Breaking Implementation

**Decision**: Use `rand::thread_rng().gen_range(0..tied_count)` for random selection.

**Rationale**: Random selection is fairest for load distribution (per Clarification 3). The `rand` crate is already a workspace dependency.

**Alternatives Considered**:
- Deterministic (first in list): Would always hit same server, unfair load distribution
- By server name hash: Pseudo-random but deterministic, still unfair

### D5: Auto-Retry Architecture

**Decision**: Simple loop with `HashSet<String>` for failed server IDs.

**Rationale**: Tracking failed servers by ID prevents re-selection during retry. 3-attempt limit prevents infinite loops. Simple, testable, meets requirements.

**Implementation Flow**:
```
1. Fetch fresh server list
2. Filter and score servers
3. Select best server (with tie-break)
4. Attempt connection (5s timeout)
5. If success → done
6. If failure → add server_id to failed_set, attempt < 3 → goto 3
7. If attempts exhausted → show error
```

### D6: Console Command Structure

**Decision**: Two commands: `/quickjoin` (full control) and `/play` (convenience alias).

**Rationale**: `/quickjoin <mode> <region>` for explicit control, `/play` or `/play <mode>` for quick access using saved preferences.

**Command Variants**:
- `/quickjoin` → use saved mode + region
- `/quickjoin tdm` → use tdm + saved region
- `/quickjoin tdm eu` → use tdm + eu
- `/play` → alias for `/quickjoin`
- `/play tdm` → alias for `/quickjoin tdm`
- `/quickjoin-prefs` → show current preferences
- `/quickjoin-prefs mode tdm` → set preferred mode
- `/quickjoin-prefs region eu` → set preferred region

## Dependencies Verification

| Dependency | Status | Notes |
|------------|--------|-------|
| Feature 025 (Identity) | Available | PlayerProfile, profile.toml, save/load functions |
| Feature 026 (Server Browser) | Available | BrowserState, ServerEntry, fetch_servers |
| rand crate | Available | Workspace dependency, used for tie-breaking |
| reqwest crate | Available | Workspace dependency with blocking feature |

## No NEEDS CLARIFICATION Items

All technical questions resolved through:
1. Spec clarifications (retry behavior, preferences location, tie-breaking)
2. Existing codebase patterns (profile persistence, server browser)
3. Standard Rust practices (HashSet for deduplication, additive scoring)
