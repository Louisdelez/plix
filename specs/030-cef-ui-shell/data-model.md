# Data Model: CEF UI Shell

**Date**: 2025-12-18
**Feature**: 030-cef-ui-shell

## Overview

This feature introduces CEF integration for rendering HTML/CSS/JS as a GPU texture. The data model consists of configuration, state management, and GPU resource entities.

## Entities

### CefConfig

Configuration for CEF UI integration. Stored in client config file.

```rust
/// CEF UI configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CefConfig {
    /// Enable CEF UI (default: true if feature available)
    pub enabled: bool,

    /// Enable CEF DevTools (default: false)
    pub devtools: bool,

    /// Path to initial HTML page (relative to assets/ui/)
    pub initial_page: String,

    /// CEF frame rate limit (default: 60, range: 1-120)
    pub frame_rate: u32,
}

impl Default for CefConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            devtools: false,
            initial_page: "index.html".to_string(),
            frame_rate: 60,
        }
    }
}
```

**TOML representation** (in client config):

```toml
[ui]
cef_enabled = true
cef_devtools = false
cef_initial_page = "index.html"
cef_frame_rate = 60
```

**Validation Rules**:
- `frame_rate` must be 1-120
- `initial_page` must be a valid relative path (no `..`, no absolute paths)
- `initial_page` must exist in `assets/ui/` directory

---

### InputFocus

State machine tracking input routing between game and CEF UI.

```rust
/// Input focus state - determines where input events are routed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputFocus {
    /// Game has focus - input goes to gameplay (movement, camera, etc.)
    #[default]
    Game,

    /// CEF UI has focus - input goes to HTML UI
    CefUI,
}

impl InputFocus {
    /// Check if CEF should receive input
    pub fn is_cef_focused(&self) -> bool {
        matches!(self, InputFocus::CefUI)
    }

    /// Check if game should receive input
    pub fn is_game_focused(&self) -> bool {
        matches!(self, InputFocus::Game)
    }
}
```

**State Transitions**:

| From | Event | To |
|------|-------|-----|
| Game | Mouse click on CEF UI area | CefUI |
| CefUI | Escape key pressed | Game |
| CefUI | Click outside UI area | Game |
| CefUI | UI closed/hidden | Game |
| Any | CEF crash/fallback | Game |

---

### CefTexture

GPU texture containing the CEF rendered frame.

```rust
/// GPU texture for CEF rendered content
pub struct CefTexture {
    /// wgpu texture handle
    texture: wgpu::Texture,

    /// Texture view for rendering
    view: wgpu::TextureView,

    /// Bind group for shader access
    bind_group: wgpu::BindGroup,

    /// Current texture dimensions
    width: u32,
    height: u32,

    /// Texture format (BGRA to match CEF output)
    format: wgpu::TextureFormat, // Bgra8Unorm
}

impl CefTexture {
    /// Create texture with given dimensions
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self;

    /// Update texture from CEF paint buffer (BGRA)
    pub fn update(&self, queue: &wgpu::Queue, buffer: &[u8], width: u32, height: u32);

    /// Resize texture (recreates GPU resources)
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);

    /// Get bind group for rendering
    pub fn bind_group(&self) -> &wgpu::BindGroup;
}
```

**Constraints**:
- Maximum texture size: 4096x4096 (configurable)
- Texture format: `Bgra8Unorm` (matches CEF OnPaint output)
- Update frequency: matches CEF frame rate (up to 60fps)

---

### CefShell

Main integration component managing CEF lifecycle.

```rust
/// CEF integration shell - manages browser instance and rendering
pub struct CefShell {
    /// CEF configuration
    config: CefConfig,

    /// Current input focus state
    focus: InputFocus,

    /// GPU texture for rendered content
    texture: Option<CefTexture>,

    /// CEF browser instance (opaque handle)
    browser: Option<CefBrowser>,

    /// Initialization status
    status: CefStatus,

    /// Pending paint buffer (double-buffered)
    pending_paint: Option<PaintBuffer>,
}

/// CEF initialization status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CefStatus {
    /// Not yet initialized
    Uninitialized,

    /// Initializing (CEF subprocess starting)
    Initializing,

    /// Ready and running
    Ready,

    /// Failed to initialize (fallback active)
    Failed { reason: &'static str },

    /// Disabled by config or flag
    Disabled,
}

/// Paint buffer from CEF OnPaint callback
struct PaintBuffer {
    data: Vec<u8>,
    width: u32,
    height: u32,
}
```

**Lifecycle**:

```
Uninitialized → Initializing → Ready
                     ↓
                  Failed → (fallback to native UI)

Disabled (if config cef_enabled = false)
```

**Public API**:

```rust
impl CefShell {
    /// Create new shell (does not initialize CEF yet)
    pub fn new(config: CefConfig) -> Self;

    /// Initialize CEF (must be called from main thread)
    pub fn initialize(&mut self, device: &wgpu::Device) -> Result<(), CefError>;

    /// Shutdown CEF cleanly
    pub fn shutdown(&mut self);

    /// Navigate to a local HTML page
    pub fn navigate(&mut self, page: &str) -> Result<(), CefError>;

    /// Reload current page
    pub fn reload(&mut self);

    /// Handle window resize
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);

    /// Process CEF message loop (call each frame)
    pub fn process_messages(&mut self);

    /// Update texture from pending paint (call before render)
    pub fn update_texture(&mut self, queue: &wgpu::Queue);

    /// Forward mouse event to CEF
    pub fn on_mouse_event(&mut self, event: &MouseEvent);

    /// Forward keyboard event to CEF
    pub fn on_keyboard_event(&mut self, event: &KeyboardEvent);

    /// Set input focus
    pub fn set_focus(&mut self, focus: InputFocus);

    /// Get current focus state
    pub fn focus(&self) -> InputFocus;

    /// Get texture for rendering (None if not ready)
    pub fn texture(&self) -> Option<&CefTexture>;

    /// Check if CEF is ready
    pub fn is_ready(&self) -> bool;

    /// Check if fallback should be used
    pub fn should_fallback(&self) -> bool;
}
```

---

### CefError

Error types for CEF operations.

```rust
/// CEF operation errors
#[derive(Debug, thiserror::Error)]
pub enum CefError {
    #[error("CEF initialization failed: {0}")]
    InitFailed(String),

    #[error("CEF not initialized")]
    NotInitialized,

    #[error("Page not found: {0}")]
    PageNotFound(String),

    #[error("CEF subprocess crashed")]
    SubprocessCrashed,

    #[error("Invalid page path: {0}")]
    InvalidPath(String),

    #[error("Texture creation failed: {0}")]
    TextureError(String),
}
```

---

## Relationships

```
┌─────────────┐
│  CefConfig  │◄──────────────────────┐
└─────────────┘                       │
       │                              │
       ▼                              │
┌─────────────┐    owns      ┌────────┴────────┐
│  CefShell   │─────────────►│   CefTexture    │
└─────────────┘              └─────────────────┘
       │
       │ manages
       ▼
┌─────────────┐
│ InputFocus  │
└─────────────┘
```

## State Diagram: CefShell Lifecycle

```
                    ┌───────────────────────────────────────┐
                    │                                       │
                    ▼                                       │
┌──────────────┐  initialize()  ┌──────────────┐           │
│Uninitialized │───────────────►│ Initializing │           │
└──────────────┘                └──────────────┘           │
       │                              │                    │
       │ disabled                     │ success            │ failure
       │                              ▼                    │
       │                        ┌──────────┐               │
       │                        │  Ready   │◄──────────────┤
       │                        └──────────┘               │
       │                              │                    │
       │                              │ crash              │
       │                              ▼                    │
       │                        ┌──────────┐               │
       └───────────────────────►│  Failed  │───────────────┘
                                └──────────┘
                                      │
                                      │ fallback to Feature 005 native UI
                                      ▼
```

## File Locations

| Entity | Location |
|--------|----------|
| CefConfig | `crates/plix-client/src/ui_cef/config.rs` |
| InputFocus | `crates/plix-client/src/ui_cef/input.rs` |
| CefTexture | `crates/plix-client/src/ui_cef/texture.rs` |
| CefShell | `crates/plix-client/src/ui_cef/mod.rs` |
| CefError | `crates/plix-client/src/ui_cef/mod.rs` |
