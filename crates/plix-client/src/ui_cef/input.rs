//! CEF UI input focus handling (Feature 030)
//!
//! This module defines the InputFocus state machine that tracks whether
//! input events should be routed to the game or the CEF UI.

/// Input focus state - determines where input events are routed
///
/// # State Transitions
///
/// | From | Event | To |
/// |------|-------|-----|
/// | Game | Mouse click on CEF UI area | CefUI |
/// | CefUI | Escape key pressed | Game |
/// | CefUI | Click outside UI area | Game |
/// | CefUI | UI closed/hidden | Game |
/// | Any | CEF crash/fallback | Game |
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
    #[inline]
    pub fn is_cef_focused(&self) -> bool {
        matches!(self, InputFocus::CefUI)
    }

    /// Check if game should receive input
    #[inline]
    pub fn is_game_focused(&self) -> bool {
        matches!(self, InputFocus::Game)
    }

    /// Toggle focus state
    pub fn toggle(&mut self) {
        *self = match self {
            InputFocus::Game => InputFocus::CefUI,
            InputFocus::CefUI => InputFocus::Game,
        };
    }

    /// Set focus to game (e.g., on ESC or click outside UI)
    pub fn release_cef_focus(&mut self) {
        *self = InputFocus::Game;
    }

    /// Set focus to CEF UI (e.g., on click on UI area)
    pub fn give_cef_focus(&mut self) {
        *self = InputFocus::CefUI;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_game() {
        let focus = InputFocus::default();
        assert_eq!(focus, InputFocus::Game);
        assert!(focus.is_game_focused());
        assert!(!focus.is_cef_focused());
    }

    #[test]
    fn test_cef_focus() {
        let focus = InputFocus::CefUI;
        assert!(focus.is_cef_focused());
        assert!(!focus.is_game_focused());
    }

    #[test]
    fn test_toggle() {
        let mut focus = InputFocus::Game;

        focus.toggle();
        assert_eq!(focus, InputFocus::CefUI);

        focus.toggle();
        assert_eq!(focus, InputFocus::Game);
    }

    #[test]
    fn test_release_cef_focus() {
        let mut focus = InputFocus::CefUI;
        focus.release_cef_focus();
        assert_eq!(focus, InputFocus::Game);

        // Should be idempotent
        focus.release_cef_focus();
        assert_eq!(focus, InputFocus::Game);
    }

    #[test]
    fn test_give_cef_focus() {
        let mut focus = InputFocus::Game;
        focus.give_cef_focus();
        assert_eq!(focus, InputFocus::CefUI);

        // Should be idempotent
        focus.give_cef_focus();
        assert_eq!(focus, InputFocus::CefUI);
    }
}
