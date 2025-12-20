//! CEF UI Shell - Optional HTML/CSS/JS UI rendering (Feature 030)
//!
//! This module provides an optional CEF (Chromium Embedded Framework) integration
//! for rendering HTML/CSS/JS UI as a GPU texture inside the game. When CEF is
//! unavailable or disabled, the system falls back to the native UI (Feature 005).
//!
//! # Feature Flag
//!
//! This module is conditionally compiled with the `cef-ui` feature flag.
//! When disabled, stub implementations are provided that always fall back.
//!
//! # Usage
//!
//! ```ignore
//! use plix_client::ui_cef::{CefShell, CefConfig};
//!
//! let config = CefConfig::default();
//! let mut shell = CefShell::new(config);
//!
//! // Initialize CEF (may fail, triggering fallback)
//! if let Err(e) = shell.initialize(&device) {
//!     tracing::warn!("CEF init failed: {}, using fallback", e);
//! }
//!
//! // In game loop
//! shell.process_messages();
//! shell.update_texture(&queue);
//!
//! // Render CEF texture if ready
//! if let Some(texture) = shell.texture() {
//!     // Render as fullscreen quad
//! }
//! ```

pub mod bridge;
mod config;
pub mod dialogue;
pub mod dungeon;
pub mod embeds;
pub mod ingame;
mod input;
pub mod menus;
pub mod quest;

pub use config::CefConfig;
pub use input::InputFocus;

use thiserror::Error;

/// CEF operation errors
#[derive(Debug, Error)]
pub enum CefError {
    /// CEF initialization failed
    #[error("CEF initialization failed: {0}")]
    InitFailed(String),

    /// CEF not initialized
    #[error("CEF not initialized")]
    NotInitialized,

    /// Page not found
    #[error("Page not found: {0}")]
    PageNotFound(String),

    /// CEF subprocess crashed
    #[error("CEF subprocess crashed")]
    SubprocessCrashed,

    /// Invalid page path
    #[error("Invalid page path: {0}")]
    InvalidPath(String),

    /// Texture creation failed
    #[error("Texture creation failed: {0}")]
    TextureError(String),
}

/// CEF initialization status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CefStatus {
    /// Not yet initialized
    #[default]
    Uninitialized,

    /// Initializing (CEF subprocess starting)
    Initializing,

    /// Ready and running
    Ready,

    /// Failed to initialize (fallback active)
    Failed,

    /// Disabled by config or flag
    Disabled,
}

/// CEF integration shell - manages browser instance and rendering
///
/// This is a stub implementation that always uses fallback mode.
/// The actual CEF integration will be added after the spike validates
/// the chosen CEF binding approach.
pub struct CefShell {
    /// CEF configuration
    config: CefConfig,

    /// Current input focus state
    focus: InputFocus,

    /// Initialization status
    status: CefStatus,
}

impl CefShell {
    /// Create new shell (does not initialize CEF yet)
    pub fn new(config: CefConfig) -> Self {
        let status = if config.enabled {
            CefStatus::Uninitialized
        } else {
            CefStatus::Disabled
        };

        Self {
            config,
            focus: InputFocus::default(),
            status,
        }
    }

    /// Initialize CEF (stub - always fails until CEF binding is integrated)
    ///
    /// # Errors
    ///
    /// Returns `CefError::InitFailed` in stub mode (always).
    /// After CEF binding integration, may also return other errors.
    #[cfg(feature = "cef-ui")]
    pub fn initialize(&mut self, _device: &wgpu::Device) -> Result<(), CefError> {
        if self.status == CefStatus::Disabled {
            return Ok(()); // Already disabled, nothing to do
        }

        self.status = CefStatus::Initializing;

        // TODO: Actual CEF initialization after spike
        // For now, always fail to trigger fallback
        self.status = CefStatus::Failed;
        Err(CefError::InitFailed(
            "CEF not yet integrated (stub mode)".to_string(),
        ))
    }

    /// Initialize CEF (stub - always disabled when feature not enabled)
    #[cfg(not(feature = "cef-ui"))]
    pub fn initialize(&mut self, _device: &wgpu::Device) -> Result<(), CefError> {
        self.status = CefStatus::Disabled;
        Ok(())
    }

    /// Shutdown CEF cleanly
    pub fn shutdown(&mut self) {
        // TODO: Actual CEF shutdown after spike
        self.status = CefStatus::Uninitialized;
    }

    /// Navigate to a local HTML page
    ///
    /// # Errors
    ///
    /// Returns error if CEF not ready or path is invalid.
    pub fn navigate(&mut self, page: &str) -> Result<(), CefError> {
        if self.status != CefStatus::Ready {
            return Err(CefError::NotInitialized);
        }

        // Validate path (no "..", no absolute paths)
        if page.contains("..") || page.starts_with('/') || page.starts_with('\\') {
            return Err(CefError::InvalidPath(page.to_string()));
        }

        // TODO: Actual navigation after spike
        Ok(())
    }

    /// Reload current page
    #[allow(clippy::needless_return)]
    pub fn reload(&mut self) {
        if self.status != CefStatus::Ready {
            return;
        }
        // TODO: Actual reload after spike
    }

    /// Handle window resize
    #[cfg(feature = "cef-ui")]
    pub fn resize(&mut self, _device: &wgpu::Device, _width: u32, _height: u32) {
        // TODO: Actual resize after spike
    }

    /// Handle window resize (stub when feature disabled)
    #[cfg(not(feature = "cef-ui"))]
    pub fn resize(&mut self, _width: u32, _height: u32) {
        // No-op when CEF disabled
    }

    /// Process CEF message loop (call each frame)
    pub fn process_messages(&mut self) {
        // TODO: Actual message processing after spike
    }

    /// Update texture from pending paint (call before render)
    #[cfg(feature = "cef-ui")]
    pub fn update_texture(&mut self, _queue: &wgpu::Queue) {
        // TODO: Actual texture update after spike
    }

    /// Update texture (stub when feature disabled)
    #[cfg(not(feature = "cef-ui"))]
    pub fn update_texture(&mut self) {
        // No-op when CEF disabled
    }

    /// Set input focus
    pub fn set_focus(&mut self, focus: InputFocus) {
        self.focus = focus;
    }

    /// Get current focus state
    pub fn focus(&self) -> InputFocus {
        self.focus
    }

    /// Get current status
    pub fn status(&self) -> CefStatus {
        self.status
    }

    /// Check if CEF is ready
    pub fn is_ready(&self) -> bool {
        self.status == CefStatus::Ready
    }

    /// Check if fallback should be used
    pub fn should_fallback(&self) -> bool {
        matches!(
            self.status,
            CefStatus::Disabled | CefStatus::Failed | CefStatus::Uninitialized
        )
    }

    /// Get configuration
    pub fn config(&self) -> &CefConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cef_shell_new_enabled() {
        let config = CefConfig::default();
        assert!(config.enabled);

        let shell = CefShell::new(config);
        assert_eq!(shell.status(), CefStatus::Uninitialized);
        assert!(!shell.is_ready());
        assert!(shell.should_fallback()); // Uninitialized = fallback
    }

    #[test]
    fn test_cef_shell_new_disabled() {
        let config = CefConfig {
            enabled: false,
            ..Default::default()
        };

        let shell = CefShell::new(config);
        assert_eq!(shell.status(), CefStatus::Disabled);
        assert!(!shell.is_ready());
        assert!(shell.should_fallback());
    }

    #[test]
    fn test_cef_shell_focus() {
        let config = CefConfig::default();
        let mut shell = CefShell::new(config);

        assert_eq!(shell.focus(), InputFocus::Game);

        shell.set_focus(InputFocus::CefUI);
        assert_eq!(shell.focus(), InputFocus::CefUI);

        shell.set_focus(InputFocus::Game);
        assert_eq!(shell.focus(), InputFocus::Game);
    }

    #[test]
    fn test_navigate_requires_ready() {
        let config = CefConfig::default();
        let mut shell = CefShell::new(config);

        // Should fail because not initialized
        let result = shell.navigate("index.html");
        assert!(matches!(result, Err(CefError::NotInitialized)));
    }

    #[test]
    fn test_navigate_rejects_invalid_paths() {
        let config = CefConfig::default();
        let mut shell = CefShell::new(config);

        // Manually set to Ready to test path validation
        shell.status = CefStatus::Ready;

        // Path traversal should be rejected
        assert!(matches!(
            shell.navigate("../secret.html"),
            Err(CefError::InvalidPath(_))
        ));

        // Absolute path should be rejected
        assert!(matches!(
            shell.navigate("/etc/passwd"),
            Err(CefError::InvalidPath(_))
        ));

        // Valid relative path should work
        assert!(shell.navigate("menu/main.html").is_ok());
    }
}
