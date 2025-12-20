# Feature 033: CEF Media Embeds

## Overview

This feature adds a media embed panel to the CEF UI overlay, allowing players to watch YouTube videos and Twitch streams while in-game.

## Quick Start

1. **Enable embeds** (enabled by default):
   ```toml
   # ~/.config/plix/config.toml
   [ui]
   cef_embeds = true
   cef_embeds_youtube = true
   cef_embeds_twitch = true
   ```

2. **Start the game** and join a server or local game

3. **Press F8** to open the embed panel

4. **Paste a YouTube URL** (e.g., `https://youtube.com/watch?v=dQw4w9WgXcQ`)

5. **Click Load** - video should appear in the panel

6. **Click anywhere on the panel** to focus it (pause gameplay input)

7. **Press Escape** to return to gameplay (video continues)

8. **Press F8 again** to hide the panel

## Configuration

| Config Key | Type | Default | Description |
|------------|------|---------|-------------|
| `ui.cef_embeds` | bool | true | Master enable/disable for embeds |
| `ui.cef_embeds_youtube` | bool | true | Enable YouTube provider |
| `ui.cef_embeds_twitch` | bool | true | Enable Twitch provider |
| `ui.cef_embeds_spotify` | bool | false | Enable Spotify provider (stubbed) |
| `ui.cef_embeds_autoplay` | bool | false | Autoplay videos on load |
| `ui.cef_embeds_chat` | bool | false | Show Twitch chat alongside stream |
| `ui.cef_embeds_twitch_parent` | string | "localhost" | Twitch embed parent domain |

## Keybinds

| Key | Action |
|-----|--------|
| F8 | Toggle embed panel visibility |
| Escape | Return to gameplay from embed focus |
| Click on panel | Focus embed panel (capture input) |

## Supported URL Formats

### YouTube

- `https://youtube.com/watch?v=VIDEO_ID`
- `https://youtu.be/VIDEO_ID`
- `https://youtube.com/shorts/VIDEO_ID`
- `VIDEO_ID` (direct ID, 11 characters)

All YouTube URLs are converted to the privacy-enhanced format:
`https://www.youtube-nocookie.com/embed/{VIDEO_ID}`

### Twitch

- `https://twitch.tv/CHANNEL_NAME`
- `CHANNEL_NAME` (direct channel name)

Twitch URLs are converted to the player format:
`https://player.twitch.tv/?channel={CHANNEL}&parent={PARENT}`

### Spotify (Stubbed)

- `https://open.spotify.com/track/TRACK_ID`
- Spotify is disabled by default and returns EEMB002

## Error Codes

| Code | Name | Description | Resolution |
|------|------|-------------|------------|
| EEMB001 | InvalidUrl | URL parsing failed or no valid ID | Check URL format |
| EEMB002 | ProviderDisabled | Provider is disabled in config | Enable provider in config.toml |
| EEMB003 | BlockedDomain | Navigation to non-whitelisted domain | Only whitelisted domains allowed |
| EEMB004 | RateLimited | Action within 2s cooldown period | Wait 2 seconds |

## Security

### Domain Whitelist

Only the following domains are allowed for embed navigation:

**YouTube:**
- `youtube.com`
- `www.youtube.com`
- `youtu.be`
- `youtube-nocookie.com`
- `www.youtube-nocookie.com`

**Twitch:**
- `twitch.tv`
- `www.twitch.tv`
- `player.twitch.tv`

**Spotify:**
- `open.spotify.com`

### Blocked Actions

- All navigation to non-whitelisted domains
- All popup windows (window.open)
- All downloads
- All file:// URLs

## Focus Behavior

1. **Panel Hidden**: Gameplay has full input
2. **Panel Visible, Unfocused**: Gameplay has input, panel is passive
3. **Panel Visible, Focused**: Panel has input, gameplay blocked

### Focus Transitions

| Current State | Action | New State |
|---------------|--------|-----------|
| Hidden | F8 pressed | Visible, Unfocused |
| Visible | F8 pressed | Hidden |
| Visible, Unfocused | Click on panel | Visible, Focused |
| Focused | Escape pressed | Visible, Unfocused |
| Focused | Window focus loss | Visible, Unfocused |

## Rate Limiting

To prevent spam, there is a 2-second cooldown between load requests. Attempting to load another video within this window returns EEMB004.

## Debug Mode

Enable bridge debug logging:
```toml
[ui]
debug_bridge = true
```

This logs all embed bridge messages and security events to the console.

## Troubleshooting

### Panel doesn't open
- Check `ui.cef_embeds = true` in config
- Check CEF is working (menu UI loads)

### Video doesn't load
- Verify URL format is supported
- Check provider is enabled
- Check console for error codes

### Twitch embed shows error
- May need to adjust `ui.cef_embeds_twitch_parent`
- Try `localhost` or your machine's hostname

### Input stuck in panel
- Press Escape to release focus
- Alt-tab will also release focus
- Worst case: F8 to hide panel

## Architecture

```text
F8 Key
   │
   ▼
EmbedsManager ──► EmbedPanel (visible, focused)
   │                    │
   │                    ▼
   │              EmbedSlot (provider, content_id, embed_url)
   │                    │
   ▼                    ▼
InputFocus ◄───── CEF Iframe (YouTube/Twitch player)
   │
   ▼
Block gameplay input if EmbedFocus
```

## Files

### Rust (crates/plix-client)

- `src/ui_cef/config.rs` - Embed config fields
- `src/ui_cef/input.rs` - EmbedFocus state
- `src/ui_cef/bridge/messages.rs` - Embed message types
- `src/ui_cef/bridge/serialize.rs` - Embed payload structs
- `src/ui_cef/embeds/mod.rs` - EmbedsManager coordinator
- `src/ui_cef/embeds/config.rs` - EmbedConfig view
- `src/ui_cef/embeds/provider.rs` - EmbedProvider, whitelist
- `src/ui_cef/embeds/normalizer.rs` - URL normalization
- `src/ui_cef/embeds/navigation_guard.rs` - CEF navigation intercept

### Web UI (assets/ui)

- `embeds/embeds.html` - Panel structure
- `embeds/embeds.css` - Panel styles
- `embeds/embeds.js` - Bridge integration
