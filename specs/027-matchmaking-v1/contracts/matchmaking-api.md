# Matchmaking API Contract

**Feature**: 027-matchmaking-v1
**Type**: Internal Client Module API (no network protocol)

## Overview

The matchmaking module is entirely client-side. It does not define new network protocols; instead, it orchestrates existing components (server browser, profile, connection logic) to implement quick join functionality.

## Module API

### `matchmaking::quick_join`

Main entry point for quick join functionality.

```rust
/// Execute a quick join request.
///
/// # Arguments
/// * `request` - The matchmaking request with mode/region preferences
/// * `browser_state` - Server browser state for fetching server list
/// * `protocol_version` - Client's protocol version for compatibility check
///
/// # Returns
/// * `QuickJoinResult` - Outcome of the quick join attempt
///
/// # Behavior
/// 1. Fetches fresh server list from master
/// 2. Filters by mode, region, protocol version, capacity
/// 3. Scores remaining servers
/// 4. Selects best server (random tie-break)
/// 5. Returns result for caller to initiate connection
pub fn select_server(
    request: &QuickJoinRequest,
    servers: &[ServerEntry],
    protocol_version: &str,
    current_time: u64,
) -> QuickJoinResult;
```

### `matchmaking::scoring`

Server scoring implementation.

```rust
/// Score a list of candidate servers.
///
/// # Arguments
/// * `servers` - Pre-filtered server list
/// * `request` - The quick join request (for region matching)
/// * `current_time` - Current Unix timestamp for freshness calculation
///
/// # Returns
/// * Vec of `ServerScore` sorted by total_score descending
pub fn score_servers(
    servers: &[ServerEntry],
    request: &QuickJoinRequest,
    current_time: u64,
) -> Vec<ServerScore>;

/// Calculate score for a single server.
pub fn calculate_score(
    server: &ServerEntry,
    request: &QuickJoinRequest,
    current_time: u64,
) -> ServerScore;
```

### `matchmaking::filtering`

Server filtering implementation.

```rust
/// Filter servers based on request criteria.
///
/// # Mandatory Filters (always applied)
/// - Incompatible protocol version → excluded
/// - Full servers (player_count >= max_players) → excluded
///
/// # Optional Filters (based on request)
/// - Mode filter: server must have requested mode in game_modes (unless mode="any")
/// - Region filter: server must match requested region (unless region="any")
///
/// # Returns
/// * Vec of servers that pass all filters
pub fn filter_servers(
    servers: &[ServerEntry],
    request: &QuickJoinRequest,
    protocol_version: &str,
) -> Vec<ServerEntry>;
```

### `matchmaking::selection`

Server selection with tie-breaking.

```rust
/// Select the best server from scored candidates.
///
/// If multiple servers have the same highest score, selects randomly among them.
///
/// # Arguments
/// * `scored` - Scored servers (must be non-empty)
///
/// # Returns
/// * Selected ServerEntry
pub fn select_best_server(scored: &[ServerScore]) -> ServerEntry;
```

### `matchmaking::retry`

Retry orchestration (used by main.rs integration).

```rust
/// Retry state for quick join attempts.
pub struct RetryState {
    /// Server IDs that have already failed
    pub failed_servers: HashSet<String>,
    /// Number of attempts made
    pub attempts: u8,
}

impl RetryState {
    pub const MAX_ATTEMPTS: u8 = 3;

    pub fn new() -> Self;

    /// Mark a server as failed.
    pub fn mark_failed(&mut self, server_id: &str);

    /// Check if an attempt can be made.
    pub fn can_retry(&self) -> bool;

    /// Check if a server has failed.
    pub fn is_failed(&self, server_id: &str) -> bool;

    /// Increment attempt counter.
    pub fn increment_attempt(&mut self);
}
```

## Console Commands

### `/quickjoin [mode] [region]`

Initiates quick join with optional mode and region overrides.

**Syntax**:
```
/quickjoin                 # Use saved preferences
/quickjoin tdm             # Use tdm mode, saved region
/quickjoin tdm eu          # Use tdm mode, eu region
```

**Valid Modes**: `tdm`, `ffa`, `ctf`, `br`, `training`, `any`
**Valid Regions**: `eu`, `us`, `asia`, `any`

**Output** (console messages):
```
[Matchmaking] Starting quick join: mode=tdm, region=eu
[Matchmaking] Found 15 servers, 8 match criteria
[Matchmaking] Selected: "EU TDM Server #1" (score: 95) at 192.168.1.100:7777
[Matchmaking] Connecting...
```

**Error Output**:
```
[Matchmaking] No servers available matching mode=tdm, region=eu
[Matchmaking] Expanded search to any region...
[Matchmaking] Connection failed (timeout), retrying (2/3)...
[Matchmaking] All connection attempts failed. Use /servers to browse manually.
```

### `/play [mode]`

Alias for `/quickjoin` with simplified syntax.

**Syntax**:
```
/play                      # Same as /quickjoin
/play tdm                  # Same as /quickjoin tdm
```

### `/quickjoin-prefs [setting] [value]`

View or update quick join preferences.

**Syntax**:
```
/quickjoin-prefs           # Show current preferences
/quickjoin-prefs mode tdm  # Set preferred mode to tdm
/quickjoin-prefs region eu # Set preferred region to eu
```

**Output**:
```
[Preferences] Quick Join settings:
  Preferred mode: tdm
  Preferred region: any
```

**Update Output**:
```
[Preferences] Updated preferred mode to: tdm
```

## Integration Points

### With Server Browser (Feature 026)

```rust
// In main.rs quick join handler:

// 1. Fetch fresh server list
browser_state.refresh().await?;

// 2. Get cached servers
let servers = browser_state.servers().await;

// 3. Use matchmaking module
let result = matchmaking::select_server(&request, &servers, PROTOCOL_VERSION, now);
```

### With Profile (Feature 025)

```rust
// Loading preferences:
let profile = load_profile();
let prefs = &profile.matchmaking;

// Saving preferences:
profile.matchmaking.preferred_mode = "ffa".to_string();
save_profile(&profile)?;
```

### With Connection Logic

```rust
// After selection:
if let Some(server) = result.selected_server {
    // Use existing connection logic from Feature 026
    connect_to_server(&server.host, server.port, &profile.display_name);
}
```

## Error Codes

| Code | Description | User Message |
|------|-------------|--------------|
| NO_SERVERS | No servers registered with master | "No servers available" |
| NO_MATCH | No servers match criteria after fallbacks | "No matching servers found" |
| TIMEOUT | Connection timed out (5 seconds) | "Connection timed out" |
| FULL | Server became full between selection and connect | "Server is full" |
| VERSION | Protocol version mismatch | "Incompatible server version" |
| MASTER_UNREACHABLE | Cannot contact master server | "Cannot reach server directory" |
| DEBOUNCE | Request too soon after previous | "Please wait before retrying" |

## Rate Limiting

- Minimum 2 seconds between quick join requests (client-side debounce)
- Prevents spam and excessive master server load
