# Quickstart: CEF Media Embeds

**Feature**: 033-cef-embeds
**Date**: 2025-12-18

## Overview

This feature adds a media embed panel to the CEF UI overlay, allowing players to watch YouTube videos and Twitch streams while in-game.

## Prerequisites

- Feature 030 (CEF UI Shell) - Provides CEF runtime
- Feature 031 (CEF Menus) - Provides bridge protocol
- Feature 032 (CEF In-Game UI) - Provides ingame overlay patterns

## Quick Test

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

## Configuration Reference

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

## URL Formats Supported

### YouTube

- `https://youtube.com/watch?v=VIDEO_ID`
- `https://youtu.be/VIDEO_ID`
- `https://youtube.com/shorts/VIDEO_ID`
- `VIDEO_ID` (direct ID, 11 characters)

### Twitch

- `https://twitch.tv/CHANNEL_NAME`
- `CHANNEL_NAME` (direct channel name)

## Error Codes

| Code | Meaning | Resolution |
|------|---------|------------|
| EEMB001 | Invalid URL | Check URL format, ensure valid video/channel ID |
| EEMB002 | Provider disabled | Enable provider in config.toml |
| EEMB003 | Domain blocked | Only whitelisted domains allowed |
| EEMB004 | Rate limited | Wait 2 seconds between load requests |

## Debug Mode

Enable bridge debug logging:
```toml
[ui]
debug_bridge = true
```

This logs all embed bridge messages to the console.

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

## Files Changed

### Rust (crates/plix-client)

- `src/ui_cef/config.rs` - Embed config fields
- `src/ui_cef/input.rs` - EmbedFocus state
- `src/ui_cef/bridge/messages.rs` - Embed message types
- `src/ui_cef/bridge/handlers.rs` - Embed message handlers
- `src/ui_cef/embeds/mod.rs` - EmbedsManager (NEW)
- `src/ui_cef/embeds/provider.rs` - EmbedProvider, whitelist (NEW)
- `src/ui_cef/embeds/normalizer.rs` - URL normalization (NEW)

### Web UI (assets/ui)

- `embeds/embeds.html` - Panel structure (NEW)
- `embeds/embeds.css` - Panel styles (NEW)
- `embeds/embeds.js` - Bridge integration (NEW)
