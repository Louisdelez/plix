# Data Model: CEF Media Embeds

**Feature**: 033-cef-embeds
**Date**: 2025-12-18

## Entities

### EmbedProvider

Enum representing supported media providers.

```text
EmbedProvider
├── YouTube
├── Twitch
└── Spotify (stubbed)
```

**Attributes**:
- `name: String` - Display name ("YouTube", "Twitch", "Spotify")
- `whitelist_domains: Vec<String>` - Allowed domains for this provider

**Whitelist by Provider**:

| Provider | Allowed Domains |
|----------|-----------------|
| YouTube | `youtube.com`, `www.youtube.com`, `youtu.be`, `youtube-nocookie.com`, `www.youtube-nocookie.com` |
| Twitch | `twitch.tv`, `www.twitch.tv`, `player.twitch.tv` |
| Spotify | `open.spotify.com` |

### EmbedConfig

User configuration for embed feature. Persisted in client config TOML under `[ui]` section.

```text
EmbedConfig
├── enabled: bool (default: true)
├── youtube_enabled: bool (default: true)
├── twitch_enabled: bool (default: true)
├── spotify_enabled: bool (default: false)
├── autoplay: bool (default: false)
├── twitch_chat: bool (default: false)
└── twitch_parent: String (default: "localhost")
```

**TOML Example**:
```toml
[ui]
cef_embeds = true
cef_embeds_youtube = true
cef_embeds_twitch = true
cef_embeds_spotify = false
cef_embeds_autoplay = false
cef_embeds_chat = false
cef_embeds_twitch_parent = "localhost"
```

**Validation Rules**:
- `twitch_parent` must not be empty if Twitch is enabled
- All boolean fields default to spec-defined values

### EmbedSlot

Represents a single active embed instance.

```text
EmbedSlot
├── provider: Option<EmbedProvider>
├── content_id: Option<String> (video ID, channel name)
├── embed_url: Option<String> (canonical URL)
└── state: SlotState
```

**SlotState Enum**:
```text
SlotState
├── Empty      # No content loaded
├── Loading    # Content being loaded
├── Playing    # Content active
└── Error      # Load failed
```

**Lifecycle**:
```text
Empty → Loading → Playing
  ↑        ↓         ↓
  ←←←←←← Error ←←←←←←
  ↑                   ↓
  ←←←←←←←←←←←←←←←←←←←
        (stop/clear)
```

### EmbedPanel

UI panel state (client-side only, not persisted).

```text
EmbedPanel
├── visible: bool
├── focused: bool
├── slot: EmbedSlot
└── last_action: Option<Instant> (for rate limiting)
```

**State Transitions**:

| Current State | Action | Next State |
|---------------|--------|------------|
| visible=false | F8 pressed | visible=true, focused=false |
| visible=true | F8 pressed | visible=false, focused=false |
| visible=true, focused=false | Click on panel | focused=true |
| focused=true | Escape pressed | focused=false (visible unchanged) |
| focused=true | Window focus lost | focused=false |

### EmbedError

Error type for embed operations.

```text
EmbedError
├── code: String
└── message: String
```

**Error Codes**:

| Code | Meaning | When |
|------|---------|------|
| EEMB001 | Invalid URL or ID | URL parsing failed, no valid video/channel ID found |
| EEMB002 | Provider disabled | Provider toggle is off in config |
| EEMB003 | Blocked domain | Navigation attempted to non-whitelisted domain |
| EEMB004 | Rate limited | Action within 2s cooldown period |

## Relationships

```text
EmbedConfig ←────── EmbedsManager
     │                    │
     │                    ├── EmbedPanel
     │                    │       │
     │                    │       └── EmbedSlot
     │                    │               │
     └── enables ─────────┴── EmbedProvider
```

## Input Focus Integration

Extends existing `InputFocus` enum:

```text
InputFocus
├── Game         # Normal gameplay
├── ChatTyping   # Chat input focused (Feature 032)
└── EmbedFocus   # Embed panel focused (Feature 033)
```

**Focus Rules**:
- Only one focus state active at a time
- `EmbedFocus` and `ChatTyping` both block gameplay input
- Escape always returns to `Game`
- Window focus loss returns to `Game`
