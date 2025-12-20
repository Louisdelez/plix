# Data Model: Tooling Mods

**Feature**: 038-tooling-mods
**Date**: 2025-12-19

## SDK Types

### Core Types

```
ModId
├── value: String (unique identifier, e.g., "my-chat-filter")
└── validation: lowercase alphanumeric + hyphens, 3-64 chars

SDKVersion
├── abi_version: u8 (currently 1)
└── api_version: u8 (currently 1)

ModError
├── code: ErrorCode (EMOD001-EMOD007)
├── message: String
└── context: Option<String>

ErrorCode (enum)
├── InvalidArgument (1)
├── PermissionDenied (2)
├── NotFound (3)
├── OutOfBounds (4)
├── RateLimited (5)
├── WorldNotReady (6)
└── Unsupported (7)
```

### Capability Types

```
Capability (bitmask u32)
├── WORLD_READ: 0x01
├── WORLD_WRITE: 0x02
├── ENTITY_READ: 0x04
├── ENTITY_WRITE: 0x08
├── NET_SEND: 0x10
├── EVENT_CANCEL_CHAT: 0x20
└── EVENT_CANCEL_BLOCKS: 0x40
```

### Event Types

```
EventType (enum)
├── ServerStart (0x01)
├── ServerStop (0x02)
├── PlayerJoin (0x03)
├── PlayerLeave (0x04)
├── PlayerChat (0x05)
├── BlockPlaced (0x06)
├── BlockBroken (0x07)
├── EntitySpawned (0x08)
└── EntityDespawned (0x09)

EventContext
├── event_type: EventType
├── tick: u64
├── cancellable: bool
└── cancelled: Cell<bool>

PlayerChatPayload
├── player_id: u64
└── text: String

BlockPlacedPayload
├── player_id: Option<u64>
├── pos: IVec3
└── block_id: u16

(similar for other event payloads)
```

### World Types

```
BlockPos
├── x: i32
├── y: i32
└── z: i32

RaycastHit
├── pos: BlockPos
├── block_id: u16
├── face: BlockFace
└── distance: f32

BlockFace (enum)
├── Top, Bottom, North, South, East, West
```

### Entity Types

```
EntityHandle
├── index: u32
└── generation: u32

Transform
├── position: Vec3
├── rotation: Quat
└── scale: Vec3

DamageSource
├── kind: DamageKind
└── attacker: Option<EntityHandle>

DamageKind (enum)
├── Melee, Ranged, Fall, Explosion, Environment, Mod
```

### Network Types

```
MessageTarget (enum)
├── Server
├── Client(u64)
├── AllClients
└── Team(u8)

ModMessage
├── channel: String (format: "mod:<mod_id>:<name>")
├── target: MessageTarget
└── payload: Vec<u8> (max 8KB)
```

### Timer Types

```
TimerHandle
└── id: u32

TimerConfig
├── delay_ms: u32 (min 50)
├── repeat: bool
└── callback_id: u32
```

## ModProject Structure

```
ModProject
├── manifest: ModManifest
├── source_dir: PathBuf
├── output_dir: PathBuf
└── template: Option<TemplateName>

ModManifest (mod.toml)
├── id: ModId
├── name: String
├── version: SemVer
├── api_version: u8
├── capabilities: Vec<Capability>
├── events: Vec<EventType>
├── description: Option<String>
├── authors: Vec<String>
└── license: Option<String>
```

## ModBundle Structure

```
ModBundle (.plixmod)
├── manifest: ModManifest (mod.toml)
├── wasm: Vec<u8> (mod.wasm)
├── assets: Option<HashMap<String, Vec<u8>>>
├── sha256: String (hex digest)
└── size: u64 (bytes, max 10 MB)

Bundle Layout (ZIP):
├── mod.toml
├── mod.wasm
└── assets/ (optional)
    └── *.json, *.toml, etc.
```

## Template Structure

```
Template
├── name: TemplateName (chat-filter, world-query, timers-net)
├── description: String
├── files: HashMap<String, TemplateFile>
└── capabilities: Vec<Capability>

TemplateFile
├── path: String (relative)
├── content: String
└── templated: bool (contains {{placeholders}})

TemplateName (enum)
├── ChatFilter
├── WorldQuery
└── TimersNet
```

## DevConfig (Server-Side)

```
DevConfig (in server config)
├── hot_reload: bool (default: false)
├── watch_paths: Vec<PathBuf>
├── debounce_ms: u32 (default: 200)
├── reload_policy: ReloadPolicy
└── unsigned_allowed: bool (default: false)

ReloadPolicy (enum)
├── FallbackToPrevious (default)
├── DisableOnFailure
└── BlockWithPlayers

HotReloadState
├── watcher: notify::RecommendedWatcher
├── pending_changes: HashMap<PathBuf, Instant>
├── reload_count: u64
└── last_reload: Option<Instant>
```

## CLI State

```
CliContext
├── working_dir: PathBuf
├── templates_dir: PathBuf
├── cache_dir: PathBuf
└── verbosity: Verbosity

Verbosity (enum)
├── Quiet, Normal, Verbose, Debug

ValidationResult
├── passed: bool
├── errors: Vec<ValidationError>
└── warnings: Vec<ValidationWarning>

ValidationError
├── code: String (e.g., "E001")
├── message: String
└── location: Option<String>
```

## State Transitions

### Mod Lifecycle

```
[Created] → [Building] → [Built] → [Packing] → [Packed] → [Validated]
                ↓                      ↓                        ↓
            [BuildFailed]         [PackFailed]           [ValidationFailed]
```

### Hot-Reload Lifecycle

```
[Watching] → [ChangeDetected] → [Debouncing] → [Reloading]
                                                    ↓
                                    [ReloadSuccess] / [ReloadFailed]
                                          ↓              ↓
                                    [Watching]    [FallbackActive]
```

## Validation Rules

| Entity | Rule | Error Code |
|--------|------|------------|
| ModId | 3-64 chars, lowercase alphanumeric + hyphens | E001 |
| Bundle size | ≤ 10 MB | E002 |
| WASM exports | mod_init, mod_on_event, mod_shutdown present | E003 |
| Manifest | Valid TOML, required fields present | E004 |
| Capabilities | Valid capability IDs only | E005 |
| Events | Valid event type IDs only | E006 |
| API version | ≤ current SDK API version | E007 |

## Relationships

```
ModProject 1──1 ModManifest
ModProject 1──1 ModBundle (after pack)
ModBundle 1──* Asset
Template 1──* TemplateFile
DevConfig 1──1 HotReloadState (when enabled)
```
