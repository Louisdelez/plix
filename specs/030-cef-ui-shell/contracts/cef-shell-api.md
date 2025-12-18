# Internal API Contract: CefShell

**Feature**: 030-cef-ui-shell
**Date**: 2025-12-18
**Type**: Internal Rust API (not network protocol)

## Overview

CefShell is the main public interface for CEF integration. It manages the CEF browser instance, handles input routing, and provides the rendered texture for compositing.

## Module Path

```rust
use plix_client::ui_cef::{CefShell, CefConfig, CefError, InputFocus};
```

## API Contract

### Initialization

```rust
/// Create a new CefShell instance
/// Does not initialize CEF - call initialize() separately
pub fn new(config: CefConfig) -> Self;

/// Initialize CEF subsystem
/// Must be called from main thread before first frame
/// Returns error if CEF binaries missing or init fails
pub fn initialize(&mut self, device: &wgpu::Device) -> Result<(), CefError>;

/// Shutdown CEF cleanly
/// Must be called before dropping CefShell
/// Safe to call multiple times
pub fn shutdown(&mut self);
```

### Navigation

```rust
/// Navigate to a local HTML page
/// Path is relative to assets/ui/
/// Rejects absolute paths and paths with ".."
pub fn navigate(&mut self, page: &str) -> Result<(), CefError>;

/// Reload current page
/// No-op if no page loaded
pub fn reload(&mut self);
```

### Frame Updates

```rust
/// Process CEF message loop
/// Must be called every frame to process browser events
/// Safe to call when not initialized (no-op)
pub fn process_messages(&mut self);

/// Update GPU texture from latest CEF paint
/// Call before rendering to get latest content
/// Uses dirty rects for partial updates when possible
pub fn update_texture(&mut self, queue: &wgpu::Queue);

/// Handle window resize
/// Resizes CEF viewport and recreates texture
pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);
```

### Input Handling

```rust
/// Forward mouse event to CEF
/// Only processed when InputFocus::CefUI
pub fn on_mouse_event(&mut self, event: &MouseEvent);

/// Forward keyboard event to CEF
/// Only processed when InputFocus::CefUI
pub fn on_keyboard_event(&mut self, event: &KeyboardEvent);

/// Set input focus state
/// Affects which component receives input
pub fn set_focus(&mut self, focus: InputFocus);

/// Get current focus state
pub fn focus(&self) -> InputFocus;
```

### State Queries

```rust
/// Get texture for rendering
/// Returns None if CEF not ready or disabled
pub fn texture(&self) -> Option<&CefTexture>;

/// Check if CEF is ready for use
pub fn is_ready(&self) -> bool;

/// Check if fallback UI should be used
/// True when: disabled, failed, or not initialized
pub fn should_fallback(&self) -> bool;

/// Get current status
pub fn status(&self) -> CefStatus;
```

## Input Event Types

```rust
/// Mouse event forwarded to CEF
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub button: MouseButton,
    pub kind: MouseEventKind,
    pub modifiers: Modifiers,
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub enum MouseEventKind {
    Move,
    Press,
    Release,
    Scroll { delta_x: f32, delta_y: f32 },
}

/// Keyboard event forwarded to CEF
pub struct KeyboardEvent {
    pub key_code: u32,      // Virtual key code
    pub scan_code: u32,     // Hardware scan code
    pub kind: KeyEventKind,
    pub modifiers: Modifiers,
    pub character: Option<char>,
}

pub enum KeyEventKind {
    Press,
    Release,
    Char,  // Character input (for text fields)
}

pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
```

## Usage Example

```rust
use plix_client::ui_cef::{CefShell, CefConfig, InputFocus};

// In client initialization
let config = CefConfig::default();
let mut cef = CefShell::new(config);

if let Err(e) = cef.initialize(&device) {
    tracing::warn!("CEF init failed: {}, using fallback", e);
}

// In game loop
loop {
    // Process CEF events
    cef.process_messages();

    // Handle input
    if cef.focus().is_cef_focused() {
        cef.on_mouse_event(&mouse_event);
        cef.on_keyboard_event(&key_event);
    } else {
        // Handle game input
    }

    // Before render
    cef.update_texture(&queue);

    // In render pass
    if let Some(texture) = cef.texture() {
        render_ui_quad(texture.bind_group());
    } else if cef.should_fallback() {
        render_native_ui();
    }
}

// On shutdown
cef.shutdown();
```

## Error Handling

All fallible operations return `Result<_, CefError>`. The shell is designed to gracefully degrade:

- If `initialize()` fails, `should_fallback()` returns `true`
- If CEF crashes at runtime, status changes to `Failed` and fallback activates
- All methods are safe to call on uninitialized/failed shell (no-ops or return None)

## Thread Safety

- `CefShell` is `!Send` and `!Sync` - must be used from main thread only
- CEF requires main thread for most operations
- Texture updates must happen on render thread (same as main in plix)

## Feature Flag

All types are available only when `cef-ui` feature is enabled:

```toml
[dependencies]
plix-client = { version = "0.1", features = ["cef-ui"] }
```

When feature is disabled, the module provides stub types that always fallback.
