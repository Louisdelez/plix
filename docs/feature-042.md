# Feature 042: Accessibility

## Overview

This feature adds accessibility options to Plix including keybinding remapping, visual accessibility settings (UI scale, colorblind modes, high contrast), and audio event subtitles for deaf/hard-of-hearing players.

## Quick Start

### Keybinding Remapping

1. **Open Settings** (from main menu)
2. Navigate to **Accessibility** tab
3. Click on any keybinding row
4. Press the new key within 5 seconds
5. If a conflict occurs, choose to **Swap** or **Cancel**

### Visual Accessibility

1. **Adjust UI Scale** (75%-150%)
2. **Toggle High Contrast** mode for better visibility
3. **Select Colorblind Preset**: Protanopia, Deuteranopia, or Tritanopia

### Subtitles

1. **Enable Subtitles** in Accessibility settings
2. Choose text size: Small, Medium, or Large
3. Adjust background opacity (0-100%)

## Configuration

Accessibility settings are stored in `~/.config/plix/config.toml` under the `[accessibility]` section:

```toml
[accessibility]
ui_scale = 100
high_contrast = false
colorblind_preset = "none"

[accessibility.subtitles]
enabled = false
size = "medium"
background_opacity = 75
duration_ms = 3000
```

### Settings Reference

| Setting | Type | Default | Range | Description |
|---------|------|---------|-------|-------------|
| `ui_scale` | u8 | 100 | 75-150 | UI scale percentage |
| `high_contrast` | bool | false | - | Enable high contrast mode |
| `colorblind_preset` | string | "none" | see below | Colorblind simulation |
| `subtitles.enabled` | bool | false | - | Enable subtitles |
| `subtitles.size` | string | "medium" | small/medium/large | Subtitle text size |
| `subtitles.background_opacity` | u8 | 75 | 0-100 | Subtitle background opacity |
| `subtitles.duration_ms` | u32 | 3000 | 1000-10000 | How long subtitles display |

### Colorblind Presets

| Preset | Description |
|--------|-------------|
| `none` | No color adjustment |
| `protanopia` | Red-blind simulation (~1% of males) |
| `deuteranopia` | Green-blind simulation (~6% of males) |
| `tritanopia` | Blue-blind simulation (rare) |

## Console Commands

Accessibility features can also be controlled via console commands:

| Command | Description | Example |
|---------|-------------|---------|
| `/rebind <action> <key>` | Rebind an action | `/rebind jump space` |
| `/rebind` | List all keybindings | `/rebind` |
| `/rebind reset` | Reset to defaults | `/rebind reset` |
| `/ui_scale <value>` | Set UI scale | `/ui_scale 125` |
| `/colorblind <preset>` | Set colorblind mode | `/colorblind deuteranopia` |
| `/highcontrast <on|off>` | Toggle high contrast | `/highcontrast on` |
| `/subtitles <on|off>` | Toggle subtitles | `/subtitles on` |

## Keybinding Actions

The following actions can be remapped:

| Action | Default Key | Description |
|--------|-------------|-------------|
| `forward` | W | Move forward |
| `backward` | S | Move backward |
| `left` | A | Move left |
| `right` | D | Move right |
| `jump` | Space | Jump |
| `attack` | LMB | Attack |
| `place_block` | RMB | Place block |
| `remove_block` | LMB | Remove block |
| `pause` | Escape | Pause game |
| `toggle_debug_overlay` | F3 | Toggle debug info |

## Subtitle Events

Subtitles appear for the following audio events:

| Event | Default Text | Description |
|-------|--------------|-------------|
| Chat Message | [Chat] | New chat message received |
| Player Join | [Player Joined] | Player connected to server |
| Player Leave | [Player Left] | Player disconnected |
| Player Hit | [Hit!] | You took damage |
| Block Place | [Block Placed] | Block placement sound |
| Block Break | [Block Broken] | Block breaking sound |
| Match Start | [Match Starting] | Match countdown/start |
| Match End | [Match Ended] | Match conclusion |

## Bridge Messages

### Request Messages

| Message Type | Payload | Description |
|--------------|---------|-------------|
| `GetKeybinds` | - | Get all keybindings |
| `RebindAction` | `{action, key}` | Rebind an action |
| `SwapKeybinds` | `{action1, action2}` | Swap two bindings |
| `ResetKeybinds` | - | Reset to defaults |
| `GetAccessibilitySettings` | - | Get current settings |
| `SetAccessibility` | settings object | Update settings |

### Push Events

| Event Type | Payload | Description |
|------------|---------|-------------|
| `KeybindsList` | `{bindings: [...]}` | List of keybindings |
| `KeybindConflict` | conflict details | Conflict detected |
| `KeybindCaptureTimeout` | `{action}` | 5s capture timeout |
| `AccessibilitySettings` | settings object | Settings update |
| `SubtitleShow` | `{id, event, text, duration_ms}` | Show subtitle |
| `SubtitleClear` | - | Clear all subtitles |

## Implementation Details

### Subtitle Queue

- Maximum 3 subtitles displayed simultaneously
- Drop-oldest behavior when queue is full
- Configurable display duration (default 3 seconds)
- Subtitles expire automatically

### Keybind Capture

- 5-second timeout for key capture
- Conflict detection with swap resolution
- Capture cancelled on Escape press

### Visual Filters

Colorblind presets use CSS SVG filters (feColorMatrix) applied to the UI root element. These simulate how colors appear to individuals with various forms of color vision deficiency.

## Files

### Rust Modules

- `crates/plix-client/src/accessibility/mod.rs` - Module exports
- `crates/plix-client/src/accessibility/config.rs` - Configuration types
- `crates/plix-client/src/accessibility/audio_events.rs` - Audio event enum
- `crates/plix-client/src/accessibility/keybind_capture.rs` - Capture state machine
- `crates/plix-client/src/accessibility/subtitle_queue.rs` - Subtitle queue

### Assets

- `assets/ui/css/accessibility.css` - Accessibility styles
- `assets/ui/css/colorblind-filters.svg` - SVG color filters

### JavaScript

- `assets/ui/pages/settings.js` - Settings UI (accessibility tab)
- `assets/ui/ingame/overlay.js` - Subtitle display
