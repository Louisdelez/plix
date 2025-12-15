# Data Model: Minimal Native UI

**Feature**: 005-minimal-ui-native
**Date**: 2025-12-15

## Overview

This document defines the data structures for the minimal native UI feature, including configuration persistence, menu state management, and keybind system.

---

## Core Structures

### GameConfig

Persistent configuration stored in TOML format.

```rust
/// Game configuration persisted to config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Mouse sensitivity multiplier (0.0001 to 0.01)
    pub sensitivity: f32,

    /// Field of view in degrees (60 to 110)
    pub fov_degrees: f32,

    /// Fullscreen mode enabled
    pub fullscreen: bool,

    /// Master audio muted
    pub audio_muted: bool,

    /// Key bindings for all rebindable actions
    pub keybinds: Keybinds,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.003,
            fov_degrees: 70.0,
            fullscreen: false,
            audio_muted: false,
            keybinds: Keybinds::default(),
        }
    }
}
```

**Validation Rules**:
- `sensitivity`: Clamp to [0.0001, 0.01] on load
- `fov_degrees`: Clamp to [60.0, 110.0] on load
- `fullscreen`: Boolean, no validation needed
- `audio_muted`: Boolean, no validation needed
- `keybinds`: If any key missing, use default for that action

---

### Keybinds

Maps actions to keys, supporting rebinding.

```rust
/// All rebindable player actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Attack,
    PlaceBlock,
    RemoveBlock,
    Pause,
}

/// Key bindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinds {
    /// Map of action to bound key
    pub bindings: HashMap<Action, Key>,
}

impl Default for Keybinds {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(Action::Forward, Key::W);
        bindings.insert(Action::Backward, Key::S);
        bindings.insert(Action::Left, Key::A);
        bindings.insert(Action::Right, Key::D);
        bindings.insert(Action::Jump, Key::Space);
        bindings.insert(Action::Attack, Key::LeftClick);
        bindings.insert(Action::PlaceBlock, Key::RightClick);
        bindings.insert(Action::RemoveBlock, Key::LeftClick);
        bindings.insert(Action::Pause, Key::Escape);
        Self { bindings }
    }
}

impl Keybinds {
    /// Get the key bound to an action
    pub fn get(&self, action: Action) -> Option<Key> {
        self.bindings.get(&action).copied()
    }

    /// Set a key binding for an action
    pub fn set(&mut self, action: Action, key: Key) {
        self.bindings.insert(action, key);
    }

    /// Find which action (if any) is bound to a key
    pub fn action_for_key(&self, key: Key) -> Option<Action> {
        self.bindings.iter()
            .find(|(_, &k)| k == key)
            .map(|(&action, _)| action)
    }

    /// Swap bindings between two actions (for conflict resolution)
    pub fn swap(&mut self, action1: Action, action2: Action) {
        let key1 = self.bindings.get(&action1).copied();
        let key2 = self.bindings.get(&action2).copied();
        if let (Some(k1), Some(k2)) = (key1, key2) {
            self.bindings.insert(action1, k2);
            self.bindings.insert(action2, k1);
        }
    }
}
```

---

### Key Enum (Extended)

Extended from current implementation to support all rebindable keys.

```rust
/// Key enumeration for input (extended for rebinding)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,

    // Special keys
    Space,
    Escape,
    Enter,
    Tab,
    Backspace,

    // Modifiers
    Ctrl,
    Shift,
    Alt,

    // Arrow keys
    Up, Down, Left, Right,

    // Mouse buttons
    LeftClick,
    RightClick,
    MiddleClick,
}

impl Key {
    /// Convert from winit KeyCode
    pub fn from_keycode(code: winit::keyboard::KeyCode) -> Option<Self> {
        use winit::keyboard::KeyCode;
        match code {
            KeyCode::KeyA => Some(Key::A),
            KeyCode::KeyB => Some(Key::B),
            // ... (full mapping)
            KeyCode::Space => Some(Key::Space),
            KeyCode::Escape => Some(Key::Escape),
            KeyCode::Enter => Some(Key::Enter),
            _ => None,
        }
    }
}
```

---

### MenuState

Menu state machine for UI navigation.

```rust
/// Current menu state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuState {
    /// Normal gameplay - cursor grabbed, crosshair visible
    None,

    /// Pause menu open - cursor released, crosshair hidden
    Paused,

    /// Settings menu (sub-menu of Paused)
    Settings,

    /// Waiting for key input to rebind an action
    KeybindRebind {
        action: Action,
    },
}

impl Default for MenuState {
    fn default() -> Self {
        Self::None
    }
}
```

**State Transitions**:

| From | Input | To | Side Effects |
|------|-------|-----|--------------|
| None | ESC | Paused | Release cursor, hide crosshair |
| Paused | ESC or Resume | None | Grab cursor, show crosshair |
| Paused | Settings | Settings | - |
| Paused | Quit | Exit | Clean shutdown |
| Settings | Back/ESC | Paused | - |
| Settings | Rebind(action) | KeybindRebind | - |
| KeybindRebind | Any key | Settings | Bind key (with conflict check) |
| KeybindRebind | ESC | Settings | Cancel rebind |

---

### SettingsMenuItem

For settings menu navigation.

```rust
/// Settings menu item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsMenuItem {
    Sensitivity,
    FieldOfView,
    Fullscreen,
    AudioMute,
    Keybinds,
    Back,
}

/// Settings menu state
#[derive(Debug, Clone)]
pub struct SettingsMenu {
    /// Currently selected item
    pub selected: SettingsMenuItem,

    /// Items in display order
    pub items: Vec<SettingsMenuItem>,
}

impl Default for SettingsMenu {
    fn default() -> Self {
        Self {
            selected: SettingsMenuItem::Sensitivity,
            items: vec![
                SettingsMenuItem::Sensitivity,
                SettingsMenuItem::FieldOfView,
                SettingsMenuItem::Fullscreen,
                SettingsMenuItem::AudioMute,
                SettingsMenuItem::Keybinds,
                SettingsMenuItem::Back,
            ],
        }
    }
}
```

---

### PauseMenuItem

For pause menu navigation.

```rust
/// Pause menu item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuItem {
    Resume,
    Settings,
    QuitToDesktop,
}

/// Pause menu state
#[derive(Debug, Clone)]
pub struct PauseMenu {
    /// Currently selected item
    pub selected: PauseMenuItem,
}

impl Default for PauseMenu {
    fn default() -> Self {
        Self {
            selected: PauseMenuItem::Resume,
        }
    }
}
```

---

## TOML File Format

Example `config.toml`:

```toml
sensitivity = 0.003
fov_degrees = 70.0
fullscreen = false
audio_muted = false

[keybinds.bindings]
Forward = "W"
Backward = "S"
Left = "A"
Right = "D"
Jump = "Space"
Attack = "LeftClick"
PlaceBlock = "RightClick"
RemoveBlock = "LeftClick"
Pause = "Escape"
```

---

## Relationships

```
GameConfig
├── sensitivity: f32
├── fov_degrees: f32
├── fullscreen: bool
├── audio_muted: bool
└── keybinds: Keybinds
    └── bindings: HashMap<Action, Key>

GameState
├── config: GameConfig
├── menu_state: MenuState
├── pause_menu: PauseMenu
├── settings_menu: SettingsMenu
├── input: InputManager
│   └── uses keybinds for action mapping
└── camera: Camera
    └── fov set from config
```

---

## Invariants

1. **Config Always Valid**: `GameConfig` always contains valid values after loading (clamped/defaulted).

2. **All Actions Bound**: Every `Action` has a binding in `Keybinds` (defaults used for missing).

3. **Single Menu State**: Only one `MenuState` active at a time.

4. **Cursor State Matches Menu**: `cursor_grabbed == (menu_state == MenuState::None)`.

5. **Crosshair Matches Menu**: Crosshair visible only when `menu_state == MenuState::None`.

6. **Settings Applied Immediately**: Changes to config fields take effect without restart.

7. **Config Saved on Change**: Every settings change triggers config file save.
