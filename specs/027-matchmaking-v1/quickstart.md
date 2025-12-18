# Quickstart: Matchmaking v1 (Quick Join)

**Feature**: 027-matchmaking-v1

## Overview

Quick Join allows players to automatically find and connect to the best available game server based on their preferences, without manually browsing the server list.

## Basic Usage

### Quick Join with Defaults

```
/quickjoin
```

Uses your saved mode and region preferences (defaults: tdm, any region).

### Quick Join with Mode

```
/quickjoin ffa
```

Searches for FFA servers in any region.

### Quick Join with Mode and Region

```
/quickjoin tdm eu
```

Searches for TDM servers in the EU region.

### Quick Play Shortcut

```
/play
/play ctf
```

Alias for `/quickjoin` - same behavior, shorter command.

## Managing Preferences

### View Current Preferences

```
/quickjoin-prefs
```

Output:
```
[Preferences] Quick Join settings:
  Preferred mode: tdm
  Preferred region: any
```

### Set Preferred Mode

```
/quickjoin-prefs mode ffa
```

Valid modes: `tdm`, `ffa`, `ctf`, `br`, `training`, `any`

### Set Preferred Region

```
/quickjoin-prefs region eu
```

Valid regions: `eu`, `us`, `asia`, `any`

## Menu Integration

1. Press **ESC** to open pause menu
2. Select **Quick Play**
3. System automatically finds and connects to a server

## How Server Selection Works

1. **Fetch**: Gets fresh server list from master server
2. **Filter**: Removes incompatible/full servers
3. **Score**: Ranks servers by:
   - Region match (+50 points)
   - Partially filled (+30 points)
   - Recent heartbeat (+20 points)
   - Player count (+1 per player)
   - Low ping (+40 for <50ms, +20 for <100ms)
4. **Select**: Picks highest-scoring server (random tie-break)
5. **Connect**: Auto-connects with your display name

## Fallback Behavior

If no exact match found:
1. First expands region to "any"
2. Then expands mode to "any"
3. If still nothing, shows "No servers available"

## Auto-Retry

If connection fails:
- Automatically tries up to 3 different servers
- Failed servers are excluded from retry
- After 3 failures, suggests using `/servers` browser

## Example Session

```
> /quickjoin tdm eu
[Matchmaking] Starting quick join: mode=tdm, region=eu
[Matchmaking] Found 12 servers, 5 match criteria
[Matchmaking] Selected: "EU TDM Arena #2" (score: 87) at game.example.com:7777
[Matchmaking] Connecting...
[Connected] Joined "EU TDM Arena #2" as PlayerOne
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "No servers available" | Check master server is reachable, try `/servers` |
| "Connection timed out" | Server may be overloaded, system auto-retries |
| "Incompatible version" | Update client to latest version |
| Wrong mode/region | Check prefs with `/quickjoin-prefs` |

## Related Commands

- `/servers` - Browse full server list (Feature 026)
- `/connect <n>` - Connect to specific server by index
- `/name <name>` - Change display name (Feature 025)
