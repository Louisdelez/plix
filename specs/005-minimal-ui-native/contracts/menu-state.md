# Contract: Menu State Machine

**Feature**: 005-minimal-ui-native
**Date**: 2025-12-15

## Overview

Specifies the menu state machine, state transitions, input handling per state, and side effects of each transition.

---

## State Definitions

### MenuState::None (Playing)

Normal gameplay state.

| Property | Value |
|----------|-------|
| Cursor | Grabbed (locked/confined) |
| Cursor Visible | No |
| Crosshair | Visible |
| Gameplay Input | Enabled |
| Network | Active |
| Menu Rendering | None |

### MenuState::Paused

Pause menu open.

| Property | Value |
|----------|-------|
| Cursor | Released |
| Cursor Visible | Yes |
| Crosshair | Hidden |
| Gameplay Input | Blocked |
| Network | Active (maintained) |
| Menu Rendering | Pause menu items |

### MenuState::Settings

Settings submenu.

| Property | Value |
|----------|-------|
| Cursor | Released |
| Cursor Visible | Yes |
| Crosshair | Hidden |
| Gameplay Input | Blocked |
| Network | Active |
| Menu Rendering | Settings items |

### MenuState::KeybindRebind { action }

Awaiting key input for rebinding.

| Property | Value |
|----------|-------|
| Cursor | Released |
| Cursor Visible | Yes |
| Crosshair | Hidden |
| Gameplay Input | Blocked |
| Network | Active |
| Menu Rendering | Rebind prompt |
| Special | Next key press captured for binding |

---

## State Transitions

### State Diagram

```
                    ┌─────────────────────┐
                    │                     │
     ESC/Resume     │    MenuState::None  │     Mouse Click
    ┌───────────────│      (Playing)      │◄──────────────────┐
    │               │                     │                    │
    │               └──────────┬──────────┘                    │
    │                          │                               │
    │                          │ ESC                           │
    │                          ▼                               │
    │               ┌─────────────────────┐                    │
    │               │                     │                    │
    └──────────────►│  MenuState::Paused  │                    │
                    │                     │                    │
                    └──────────┬──────────┘                    │
                               │                               │
                 Settings      │         Quit                  │
                    ┌──────────┴─────────┐                     │
                    │                    │                     │
                    ▼                    ▼                     │
         ┌─────────────────────┐   ┌──────────┐               │
         │                     │   │  EXIT    │               │
         │ MenuState::Settings │   │  GAME    │               │
         │                     │   └──────────┘               │
         └──────────┬──────────┘                              │
                    │                                          │
         Back/ESC   │         Rebind                          │
              ┌─────┴─────┐                                    │
              │           │                                    │
              ▼           ▼                                    │
    ┌──────────┐   ┌─────────────────────────┐                │
    │  Paused  │   │ MenuState::KeybindRebind│                │
    └──────────┘   │     { action }          │                │
                   └──────────┬──────────────┘                │
                              │                               │
               Key Press      │       ESC (cancel)            │
                    ┌─────────┴─────────┐                     │
                    │                   │                     │
                    ▼                   ▼                     │
              ┌──────────┐       ┌──────────┐                 │
              │ Settings │       │ Settings │                 │
              │ (bound)  │       │(canceled)│                 │
              └──────────┘       └──────────┘                 │
```

### Transition Table

| From | Input | To | Side Effects |
|------|-------|-----|--------------|
| None | ESC key | Paused | Release cursor, show cursor, hide crosshair |
| Paused | ESC key | None | Grab cursor, hide cursor, show crosshair |
| Paused | Resume selected | None | Grab cursor, hide cursor, show crosshair |
| Paused | Settings selected | Settings | None |
| Paused | Quit selected | EXIT | Clean shutdown, disconnect from server |
| Settings | ESC key | Paused | None |
| Settings | Back selected | Paused | None |
| Settings | Keybinds selected | Settings (keybinds view) | None |
| Settings | Rebind(action) | KeybindRebind { action } | None |
| KeybindRebind | ESC key | Settings | None (rebind canceled) |
| KeybindRebind | Any other key | Settings | Bind key to action (with conflict check) |

---

## Input Handling Per State

### MenuState::None

| Input | Action |
|-------|--------|
| ESC | Transition to Paused |
| WASD | Movement input |
| Space | Jump input |
| LMB | Attack/RemoveBlock |
| RMB | PlaceBlock |
| Mouse motion | Camera rotation |

### MenuState::Paused

| Input | Action |
|-------|--------|
| ESC | Transition to None (resume) |
| Up Arrow / W | Select previous item |
| Down Arrow / S | Select next item |
| Enter | Activate selected item |
| Mouse click on item | Activate clicked item |

Menu items:
1. Resume
2. Settings
3. Quit to Desktop

### MenuState::Settings

| Input | Action |
|-------|--------|
| ESC | Transition to Paused |
| Up Arrow / W | Select previous item |
| Down Arrow / S | Select next item |
| Left Arrow / A | Decrease value (sensitivity, FOV) |
| Right Arrow / D | Increase value (sensitivity, FOV) |
| Enter | Toggle (fullscreen, audio) or enter rebind mode (keybinds) |
| Mouse click | Select/activate item |

Settings items:
1. Sensitivity (slider: 0.0001 - 0.01)
2. Field of View (slider: 60 - 110)
3. Fullscreen (toggle)
4. Audio (toggle: muted/unmuted)
5. Keybinds (opens keybind list)
6. Back

### MenuState::KeybindRebind

| Input | Action |
|-------|--------|
| ESC | Cancel rebind, return to Settings |
| Any key (keyboard or mouse) | Bind key to action, check conflicts, return to Settings |

---

## Side Effects

### Cursor Management

```rust
fn apply_cursor_state(&mut self, menu_state: MenuState) {
    match menu_state {
        MenuState::None => {
            self.grab_cursor();
            self.window.set_cursor_visible(false);
        }
        _ => {
            self.release_cursor();
            self.window.set_cursor_visible(true);
        }
    }
}

fn grab_cursor(&mut self) {
    let _ = self.window.set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));
}

fn release_cursor(&mut self) {
    let _ = self.window.set_cursor_grab(CursorGrabMode::None);
}
```

### Crosshair Visibility

```rust
fn should_show_crosshair(&self) -> bool {
    self.menu_state == MenuState::None
}
```

### Input Blocking

```rust
fn should_process_gameplay_input(&self) -> bool {
    self.menu_state == MenuState::None
}
```

---

## Keybind Conflict Resolution

When a key is pressed in `KeybindRebind` state:

```rust
fn handle_rebind_key(&mut self, key: Key) {
    let action = self.rebinding_action;

    // Check for conflict
    if let Some(conflicting_action) = self.config.keybinds.action_for_key(key) {
        if conflicting_action != action {
            // Show conflict warning
            self.show_conflict_warning(conflicting_action, key);

            // Swap bindings (per spec: Warn + Swap)
            self.config.keybinds.swap(action, conflicting_action);
        }
    } else {
        // No conflict, just bind
        self.config.keybinds.set(action, key);
    }

    // Save config
    save_config(&self.config);

    // Return to settings
    self.menu_state = MenuState::Settings;
}
```

### Conflict Warning

When swapping bindings, display a brief notification:
- Format: "{action1} and {action2} bindings swapped"
- Duration: 2 seconds
- Display: In window title or overlay (implementation dependent)

---

## Network Behavior

### While Paused

- Client continues receiving server snapshots
- Client does NOT send gameplay input
- Ping/pong heartbeat continues (if implemented)
- Connection timeout clock continues

### Timeout Prevention

If connection timeout occurs while paused:
- Option A: Automatic resume and reconnect
- Option B: Show "Connection lost" in menu

For MVP: Keep connection active, no special handling needed if paused less than 60 seconds (per spec SC-005).

---

## Menu Item Selection

### Visual Feedback

Selected item indicated by:
- Highlight color (e.g., yellow background)
- Position indicator (e.g., ">" prefix)
- Display in window title: "PLIX | Paused | > Resume"

### Navigation Wrapping

```rust
fn select_next(&mut self) {
    let count = self.items.len();
    let current = self.items.iter().position(|i| *i == self.selected).unwrap_or(0);
    let next = (current + 1) % count;
    self.selected = self.items[next];
}

fn select_previous(&mut self) {
    let count = self.items.len();
    let current = self.items.iter().position(|i| *i == self.selected).unwrap_or(0);
    let prev = (current + count - 1) % count;
    self.selected = self.items[prev];
}
```

---

## Value Adjustment

### Sensitivity

```rust
const SENSITIVITY_STEP: f32 = 0.0005;
const SENSITIVITY_MIN: f32 = 0.0001;
const SENSITIVITY_MAX: f32 = 0.01;

fn adjust_sensitivity(&mut self, delta: i32) {
    self.config.sensitivity += delta as f32 * SENSITIVITY_STEP;
    self.config.sensitivity = self.config.sensitivity.clamp(SENSITIVITY_MIN, SENSITIVITY_MAX);
    self.input.set_sensitivity(self.config.sensitivity);
    save_config(&self.config);
}
```

### Field of View

```rust
const FOV_STEP: f32 = 5.0;
const FOV_MIN: f32 = 60.0;
const FOV_MAX: f32 = 110.0;

fn adjust_fov(&mut self, delta: i32) {
    self.config.fov_degrees += delta as f32 * FOV_STEP;
    self.config.fov_degrees = self.config.fov_degrees.clamp(FOV_MIN, FOV_MAX);
    self.camera.set_fov(self.config.fov_degrees);
    save_config(&self.config);
}
```

### Toggles

```rust
fn toggle_fullscreen(&mut self) {
    self.config.fullscreen = !self.config.fullscreen;
    self.apply_fullscreen();
    save_config(&self.config);
}

fn toggle_audio_mute(&mut self) {
    self.config.audio_muted = !self.config.audio_muted;
    // Apply to audio system when implemented
    save_config(&self.config);
}
```

---

## Test Cases

### TC-001: ESC toggles pause menu

```rust
#[test]
fn esc_toggles_pause() {
    // Start in None state
    // Press ESC -> Paused
    // Press ESC -> None
}
```

### TC-002: Resume returns to gameplay

```rust
#[test]
fn resume_returns_to_gameplay() {
    // In Paused state
    // Select Resume, press Enter
    // Verify None state, cursor grabbed
}
```

### TC-003: Quit exits cleanly

```rust
#[test]
fn quit_exits_cleanly() {
    // In Paused state
    // Select Quit, press Enter
    // Verify clean exit
}
```

### TC-004: Settings navigation

```rust
#[test]
fn settings_navigation() {
    // In Settings state
    // Press Down -> next item
    // Press Up -> previous item
    // Wrap around at ends
}
```

### TC-005: Keybind rebind with conflict

```rust
#[test]
fn keybind_conflict_swap() {
    // Forward = W, Backward = S
    // Rebind Forward to S
    // Verify Forward = S, Backward = W (swapped)
}
```

### TC-006: Network maintained during pause

```rust
#[test]
fn network_maintained_during_pause() {
    // Connect to server
    // Pause for 30 seconds
    // Resume, verify still connected
}
```
