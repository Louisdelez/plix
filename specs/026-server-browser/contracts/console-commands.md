# Console Commands Contract

**Feature**: 026-server-browser
**Location**: `plix-client/src/console.rs`

## Server Browser Commands

### /servers

Fetch and display the server list from master server.

**Syntax**:
```
/servers [search_term] [flags]
```

**Arguments**:
| Argument | Required | Description |
|----------|----------|-------------|
| search_term | No | Filter servers by name, tags, or region (case-insensitive substring) |

**Flags**:
| Flag | Description |
|------|-------------|
| `--players` | Only show servers with players (player_count > 0) |
| `--compatible` | Only show servers with matching protocol version |
| `--sort=players` | Sort by player count descending (default) |
| `--sort=recent` | Sort by last_seen descending |

**Examples**:
```
/servers                    # List all servers
/servers ctf                # Search for "ctf" in name/tags/region
/servers --players          # Only servers with players
/servers eu --compatible    # Search "eu", compatible versions only
```

**Output Format**:
```
Server List (3 servers):
[1] EU Competitive CTF (16/32) [eu-west] [ctf, competitive] ping: 45ms
[2] US Casual FFA (8/16) [us-east] [ffa, casual] ping: ?
[3] Asia Training (0/32) [asia] [training] ping: 120ms

Use /connect <number> to join a server
```

**Error Output**:
```
Error: Could not reach master server (timeout)
Error: No servers found matching "xyz"
```

---

### /connect

Connect to a server from the list.

**Syntax**:
```
/connect <index>
```

**Arguments**:
| Argument | Required | Description |
|----------|----------|-------------|
| index | Yes | Server index from /servers list (1-based) |

**Examples**:
```
/connect 1      # Connect to first server in list
/connect 3      # Connect to third server
```

**Output Format**:
```
Connecting to EU Competitive CTF (192.168.1.100:7777)...
Connected!
```

**Error Output**:
```
Error: Invalid server index. Use /servers first.
Error: Connection failed: timeout
Error: Connection failed: incompatible version (server: 0.2.0, client: 0.1.0)
Error: Server offline
```

---

### /favorite

Add a server to favorites.

**Syntax**:
```
/favorite <index>
```

**Arguments**:
| Argument | Required | Description |
|----------|----------|-------------|
| index | Yes | Server index from /servers list (1-based) |

**Examples**:
```
/favorite 1     # Add first server to favorites
```

**Output Format**:
```
Added "EU Competitive CTF" to favorites
```

**Error Output**:
```
Error: Invalid server index. Use /servers first.
Error: Server already in favorites
```

---

### /unfavorite

Remove a server from favorites.

**Syntax**:
```
/unfavorite <index>
```

**Arguments**:
| Argument | Required | Description |
|----------|----------|-------------|
| index | Yes | Server index from /favorites list (1-based) |

**Examples**:
```
/unfavorite 1   # Remove first favorite
```

**Output Format**:
```
Removed "EU Competitive CTF" from favorites
```

**Error Output**:
```
Error: Invalid favorite index. Use /favorites first.
```

---

### /favorites

Display saved favorite servers.

**Syntax**:
```
/favorites
```

**Output Format** (with online status):
```
Favorite Servers (2 servers):
[1] EU Competitive CTF (16/32) [eu-west] [ONLINE]
[2] US Casual FFA [us-east] [OFFLINE]

Use /connect <number> to join, /unfavorite <number> to remove
```

**Output Format** (empty):
```
No favorite servers saved. Use /favorite <index> after /servers.
```

---

## Command Result Types

Commands return one of these result types:

```rust
pub enum CommandResult {
    /// Command sent to server
    SendMessage(ClientMessage),
    /// Command handled client-side only
    ClientOnly(String),
    /// Requires async operation (server browser)
    AsyncOperation(BrowserOperation),
    /// Not a command
    NotACommand,
    /// Invalid syntax
    InvalidSyntax(String),
    /// Unknown command
    UnknownCommand(String),
}

pub enum BrowserOperation {
    FetchServers { search: Option<String>, flags: FilterFlags },
    Connect { index: usize },
    AddFavorite { index: usize },
    RemoveFavorite { index: usize },
    ListFavorites,
}
```

## State Management

The client maintains ephemeral state for server browser:

```rust
pub struct BrowserState {
    /// Last fetched server list (cleared on disconnect)
    pub servers: Vec<ServerEntry>,
    /// Currently applied filters
    pub active_filters: FilterFlags,
    /// Timestamp of last fetch
    pub last_fetch: Option<Instant>,
}
```

- Server list persists during session for index-based commands
- Favorites persist to `~/.config/plix/servers.toml`
- `/servers` replaces the current list
- Indices are 1-based for user friendliness

## Integration with Existing Commands

The `/help` command is updated to include server browser commands:

```
Available commands:
/balance, /bal  - Show your current coin balance
/buy <offer_id> - Purchase an item from the shop
/shop           - List available shop offers
/name <name>    - Change your display name (60s cooldown)
/servers        - Browse available servers
/connect <num>  - Connect to a server from the list
/favorite <num> - Add server to favorites
/unfavorite <n> - Remove server from favorites
/favorites      - Show your favorite servers
/help           - Show this help message
```
