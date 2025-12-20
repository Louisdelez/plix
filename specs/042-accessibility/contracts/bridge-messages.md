# Bridge Message Contracts: Accessibility

**Feature**: 042-accessibility
**Version**: 1.0.0
**Protocol**: CEF Bridge (existing from Feature 030/032)

## Overview

This document defines the bridge message contracts for accessibility features. Messages extend the existing CEF bridge infrastructure from Features 030 and 032.

## Message Direction

| Direction | Description |
|-----------|-------------|
| Rust → JS | Settings sync, subtitle display, conflict notifications |
| JS → Rust | User actions (rebind, reset, setting changes) |

---

## Rust → JS Messages

### `accessibility_settings`

Sync current accessibility settings to UI on load or change.

```typescript
interface AccessibilitySettingsMsg {
  type: "accessibility_settings";
  payload: {
    ui_scale: number;           // 75-150
    high_contrast: boolean;
    colorblind_preset: "none" | "protanopia" | "deuteranopia" | "tritanopia";
    subtitles_enabled: boolean;
    subtitle_size: "small" | "medium" | "large";
    subtitle_bg_opacity: number; // 0-100
  };
}
```

**Trigger**: On settings page load, after any setting change confirmation.

---

### `keybinds_list`

Send complete keybindings list to UI.

```typescript
interface KeybindsListMsg {
  type: "keybinds_list";
  payload: {
    bindings: Array<{
      action: string;          // Action enum variant name
      action_display: string;  // Human-readable name
      key: string;             // Key enum variant name
      key_display: string;     // Human-readable key name
    }>;
  };
}
```

**Trigger**: On Controls settings page load, after rebind/reset.

---

### `keybind_conflict`

Notify UI of detected binding conflict.

```typescript
interface KeybindConflictMsg {
  type: "keybind_conflict";
  payload: {
    target_action: string;       // Action being rebound
    target_action_display: string;
    conflicting_action: string;  // Action that has the key
    conflicting_action_display: string;
    key: string;                 // The conflicting key
    key_display: string;
  };
}
```

**Trigger**: When user attempts to bind a key already in use.

**UI Response**: Show modal with Swap/Cancel options.

---

### `keybind_capture_timeout`

Notify UI that key capture timed out.

```typescript
interface KeybindCaptureTimeoutMsg {
  type: "keybind_capture_timeout";
  payload: {
    action: string;  // Action that was being rebound
  };
}
```

**Trigger**: After 5 seconds of no input during capture.

**UI Response**: Exit listening state, restore previous binding display.

---

### `subtitle_show`

Display a subtitle for an audio event.

```typescript
interface SubtitleShowMsg {
  type: "subtitle_show";
  payload: {
    id: string;         // Unique ID for this subtitle instance
    event_type: string; // AudioEvent enum variant
    text: string;       // Display text (e.g., "[Chat]", "[Player Joined]")
    duration_ms: number; // How long to display
  };
}
```

**Trigger**: When an audio event fires and subtitles are enabled.

**UI Response**: Add to subtitle queue, auto-dismiss after duration.

---

### `subtitle_clear`

Clear all subtitles (e.g., on settings change).

```typescript
interface SubtitleClearMsg {
  type: "subtitle_clear";
  payload: {};
}
```

**Trigger**: When subtitles disabled or settings change requires reset.

---

## JS → Rust Messages

### `rebind_action`

Request to rebind an action to a new key.

```typescript
interface RebindActionMsg {
  type: "rebind_action";
  payload: {
    action: string;  // Action enum variant name
    key: string;     // Key enum variant name
  };
}
```

**Rust Response**: Either `keybinds_list` (success) or `keybind_conflict` (conflict detected).

---

### `swap_keybinds`

Resolve a conflict by swapping two action bindings.

```typescript
interface SwapKeybindsMsg {
  type: "swap_keybinds";
  payload: {
    action1: string;  // First action
    action2: string;  // Second action
  };
}
```

**Rust Response**: `keybinds_list` with updated bindings.

---

### `reset_keybinds`

Reset all keybindings to defaults.

```typescript
interface ResetKeybindsMsg {
  type: "reset_keybinds";
  payload: {};
}
```

**Rust Response**: `keybinds_list` with default bindings.

---

### `start_keybind_capture`

Enter listening mode for keybind capture.

```typescript
interface StartKeybindCaptureMsg {
  type: "start_keybind_capture";
  payload: {
    action: string;  // Action to rebind
  };
}
```

**Rust Response**: Starts 5-second timeout timer. Next key input sends `rebind_action` or `keybind_capture_timeout` on timeout.

---

### `cancel_keybind_capture`

Cancel active keybind capture (e.g., Escape pressed).

```typescript
interface CancelKeybindCaptureMsg {
  type: "cancel_keybind_capture";
  payload: {};
}
```

**Rust Response**: Cancels timeout, no binding change.

---

### `set_accessibility`

Update an accessibility setting.

```typescript
interface SetAccessibilityMsg {
  type: "set_accessibility";
  payload: {
    setting: "ui_scale" | "high_contrast" | "colorblind" | "subtitles_enabled" | "subtitle_size" | "subtitle_bg_opacity";
    value: number | boolean | string;
  };
}
```

**Value Types**:
- `ui_scale`: number (75-150)
- `high_contrast`: boolean
- `colorblind`: string ("none", "protanopia", "deuteranopia", "tritanopia")
- `subtitles_enabled`: boolean
- `subtitle_size`: string ("small", "medium", "large")
- `subtitle_bg_opacity`: number (0-100)

**Rust Response**: `accessibility_settings` with updated values.

---

### `get_accessibility_settings`

Request current accessibility settings.

```typescript
interface GetAccessibilitySettingsMsg {
  type: "get_accessibility_settings";
  payload: {};
}
```

**Rust Response**: `accessibility_settings` message.

---

### `get_keybinds`

Request current keybindings list.

```typescript
interface GetKeybindsMsg {
  type: "get_keybinds";
  payload: {};
}
```

**Rust Response**: `keybinds_list` message.

---

## Error Handling

### Invalid Action/Key Names

If JS sends an invalid action or key name:

```typescript
interface ErrorMsg {
  type: "error";
  payload: {
    code: "INVALID_ACTION" | "INVALID_KEY" | "INVALID_SETTING" | "INVALID_VALUE";
    message: string;
    context: string;  // Original message type that caused error
  };
}
```

### Value Out of Range

Values outside valid ranges are clamped and a warning is logged (no error message sent to UI).

---

## Sequence Diagrams

### Keybind Rebind Flow (No Conflict)

```
UI                          Rust
 |-- start_keybind_capture -->|
 |     (action: "Forward")    |
 |                            | [Start 5s timer]
 |<-- (UI shows "Press...")   |
 |                            |
 | [User presses "Up"]        |
 |-- rebind_action ---------->|
 |   (action: "Forward",      |
 |    key: "Up")              |
 |                            | [Update config, save]
 |<-- keybinds_list ----------|
 |   (updated bindings)       |
```

### Keybind Rebind Flow (With Conflict)

```
UI                          Rust
 |-- start_keybind_capture -->|
 |                            |
 | [User presses "W"]         |
 |-- rebind_action ---------->|
 |   (action: "Jump", key:"W")|
 |                            | [Detect: "W" used by "Forward"]
 |<-- keybind_conflict -------|
 |                            |
 | [Show Swap/Cancel modal]   |
 |                            |
 |-- swap_keybinds ---------->|
 |   (action1: "Jump",        |
 |    action2: "Forward")     |
 |                            | [Swap bindings, save]
 |<-- keybinds_list ----------|
```

### Keybind Capture Timeout

```
UI                          Rust
 |-- start_keybind_capture -->|
 |                            | [Start 5s timer]
 |<-- (UI shows "Press...")   |
 |                            |
 | [5 seconds pass, no input] |
 |                            |
 |<-- keybind_capture_timeout-|
 |                            |
 | [Restore previous display] |
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-19 | Initial contract definition |
