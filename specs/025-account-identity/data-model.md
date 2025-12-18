# Data Model: Account Identity

**Feature**: 025-account-identity
**Date**: 2025-12-17

## Entities

### 1. DisplayName (plix-common)

A validated string representing a player's visible identity.

```rust
/// Validated display name (1-32 characters, alphanumeric + underscore/hyphen/space)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisplayName(String);

impl DisplayName {
    /// Minimum name length after trimming
    pub const MIN_LEN: usize = 1;
    /// Maximum name length
    pub const MAX_LEN: usize = 32;
    /// Default fallback name
    pub const DEFAULT: &'static str = "Player";

    /// Create and validate a display name
    pub fn new(input: &str) -> Result<Self, DisplayNameError>;

    /// Create with sanitization (never fails, returns fallback if invalid)
    pub fn sanitize(input: &str) -> Self;

    /// Get the inner string
    pub fn as_str(&self) -> &str;

    /// Get base name (without #N suffix if present)
    pub fn base_name(&self) -> &str;

    /// Check if name has disambiguation suffix
    pub fn has_suffix(&self) -> bool;
}

/// Validation error for display names
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DisplayNameError {
    #[error("Name is empty after trimming")]
    Empty,
    #[error("Name exceeds maximum length of {}", DisplayName::MAX_LEN)]
    TooLong,
    #[error("Name contains invalid characters")]
    InvalidCharacters,
}
```

**Validation Rules**:
- Trim leading/trailing whitespace
- Length: 1-32 characters after trim
- Allowed chars: `[a-zA-Z0-9_\- ]`
- On invalid: return `DisplayName::DEFAULT`

---

### 2. SessionId (plix-common)

Unique identifier assigned per connection, used for logging and correlation.

```rust
/// Unique session identifier (per connection, server-local)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u64);

impl SessionId {
    /// Invalid/no session
    pub const NONE: Self = Self(0);

    /// Check if valid
    pub fn is_valid(&self) -> bool { self.0 != 0 }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session({})", self.0)
    }
}
```

---

### 3. AccountId (plix-common) - v2 Placeholder

Future-proof field for authenticated accounts.

```rust
/// Unique account identifier (v2 placeholder - always None in v1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub u64);

impl AccountId {
    /// Invalid/no account (anonymous)
    pub const NONE: Self = Self(0);

    /// Check if valid
    pub fn is_valid(&self) -> bool { self.0 != 0 }
}
```

---

### 4. PlayerProfile (plix-client)

Client-side profile stored locally.

```rust
/// Local player profile (persisted to ~/.config/plix/profile.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    /// Profile format version (for migrations)
    pub version: u32,

    /// Preferred display name
    pub display_name: String,

    /// Account ID (v2 placeholder, always None in v1)
    #[serde(default)]
    pub account_id: Option<u64>,

    /// Auth token (v2 placeholder, always None in v1)
    #[serde(default)]
    pub auth_token: Option<String>,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            version: 1,
            display_name: "Player".to_string(),
            account_id: None,
            auth_token: None,
        }
    }
}
```

**File Location**: `~/.config/plix/profile.toml`

**TOML Format**:
```toml
version = 1
display_name = "MyUsername"
# account_id and auth_token omitted when None
```

---

### 5. NameRegistry (plix-server)

Server-side tracking of display name uniqueness.

```rust
/// Registry for managing unique display names on the server
#[derive(Debug, Default)]
pub struct NameRegistry {
    /// Set of all active display names (with suffixes)
    active_names: HashSet<String>,
    /// Map from base name to set of used suffix numbers
    suffix_map: HashMap<String, HashSet<u32>>,
    /// Map from PlayerId to assigned display name
    player_names: HashMap<PlayerId, String>,
}

impl NameRegistry {
    /// Register a player with a preferred name, returns assigned name
    pub fn register(&mut self, player_id: PlayerId, preferred: &str) -> String;

    /// Unregister a player (on disconnect)
    pub fn unregister(&mut self, player_id: PlayerId);

    /// Change a player's name, returns new assigned name
    pub fn rename(&mut self, player_id: PlayerId, new_name: &str) -> String;

    /// Get a player's current display name
    pub fn get_name(&self, player_id: PlayerId) -> Option<&str>;

    /// Find lowest available suffix for a base name
    fn find_available_name(&self, base: &str) -> String;
}
```

**Disambiguation Algorithm**:
1. Validate/sanitize input to get base name
2. If base name not in `active_names`, use as-is
3. Otherwise, find lowest N where `{base}#N` not in `active_names`
4. Max N = 99; if all taken, reject with "server full for this name"

---

### 6. RenameCooldown (plix-server)

Per-player rate limiting for name changes.

```rust
/// Tracks rename cooldown per player
impl ServerPlayer {
    /// Last tick when player renamed (None if never)
    pub last_rename_tick: Option<Tick>,

    /// Check if player can rename (cooldown expired)
    pub fn can_rename(&self, current_tick: Tick, cooldown_ticks: u32) -> bool;

    /// Record a rename timestamp
    pub fn record_rename(&mut self, tick: Tick);
}
```

**Cooldown**: 3600 ticks (60 seconds at 60 TPS)

---

## Relationships

```
┌─────────────────┐     validates     ┌─────────────────┐
│  DisplayName    │◄─────────────────│  NameRegistry   │
│  (validated)    │                   │  (server)       │
└─────────────────┘                   └─────────────────┘
                                              │
                                              │ assigns
                                              ▼
┌─────────────────┐     stored in     ┌─────────────────┐
│  PlayerProfile  │─────────────────►│  ServerPlayer   │
│  (client disk)  │                   │  (server mem)   │
└─────────────────┘                   └─────────────────┘
                                              │
                                              │ has
                                              ▼
                                      ┌─────────────────┐
                                      │   SessionId     │
                                      │  (logging)      │
                                      └─────────────────┘
```

---

## State Transitions

### Display Name Lifecycle

```
┌───────────┐    Connect       ┌─────────────┐    Validate    ┌─────────────┐
│  Client   │─────────────────►│  Preferred  │───────────────►│  Validated  │
│  Profile  │                  │  Name       │                │  Name       │
└───────────┘                  └─────────────┘                └─────────────┘
                                                                    │
                                                                    ▼
                               ┌─────────────┐    Register     ┌─────────────┐
                               │  Final Name │◄───────────────│ Disambiguate│
                               │  (assigned) │                │ (if needed) │
                               └─────────────┘                └─────────────┘
                                     │
                                     │ /name command
                                     ▼
                               ┌─────────────┐
                               │  Rename     │──► Same flow as above
                               │  Request    │    (with rate limit check)
                               └─────────────┘
```

### Session Lifecycle

```
┌──────────┐   Connect    ┌──────────┐   Disconnect   ┌──────────┐
│  (none)  │─────────────►│  Active  │───────────────►│  (ended) │
│          │              │SessionId │                │          │
└──────────┘              └──────────┘                └──────────┘
                               │
                               │ logged with every event
                               ▼
                          ┌──────────┐
                          │  Logs/   │
                          │ Metrics  │
                          └──────────┘
```
