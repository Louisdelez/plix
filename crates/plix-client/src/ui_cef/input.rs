//! CEF UI input focus handling (Feature 030, extended in Feature 032)
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
/// | Game | Enter key pressed | ChatTyping |
/// | Game | F8 pressed + click on embed panel | EmbedFocus |
/// | CefUI | Escape key pressed | Game |
/// | CefUI | Click outside UI area | Game |
/// | CefUI | UI closed/hidden | Game |
/// | ChatTyping | Enter/Escape pressed | Game |
/// | ChatTyping | Click outside chat | Game |
/// | EmbedFocus | Escape key pressed | Game |
/// | EmbedFocus | Window focus loss | Game |
/// | Any | CEF crash/fallback | Game |
/// | Any | Window focus loss | Game (recovery) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputFocus {
    /// Game has focus - input goes to gameplay (movement, camera, etc.)
    #[default]
    Game,

    /// CEF UI has focus - input goes to HTML UI (menus)
    CefUI,

    /// Chat input has focus - keyboard goes to chat, mouse to game (Feature 032)
    ///
    /// In this state:
    /// - Keyboard input is routed to chat input field
    /// - Mouse input is blocked from gameplay (no camera movement)
    /// - Escape closes chat and returns to Game
    /// - Enter sends message and returns to Game
    ChatTyping,

    /// Embed panel has focus - keyboard/mouse goes to embed iframe (Feature 033)
    ///
    /// In this state:
    /// - All input is routed to the embed iframe
    /// - Gameplay input is blocked
    /// - Escape returns to Game (panel stays visible)
    /// - Window focus loss returns to Game
    EmbedFocus,
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

    /// Check if chat is being typed (Feature 032)
    #[inline]
    pub fn is_chat_typing(&self) -> bool {
        matches!(self, InputFocus::ChatTyping)
    }

    /// Check if embed panel has focus (Feature 033)
    #[inline]
    pub fn is_embed_focus(&self) -> bool {
        matches!(self, InputFocus::EmbedFocus)
    }

    /// Check if gameplay input should be blocked (menu, chat, or embed active)
    #[inline]
    pub fn blocks_gameplay(&self) -> bool {
        matches!(
            self,
            InputFocus::CefUI | InputFocus::ChatTyping | InputFocus::EmbedFocus
        )
    }

    /// Toggle focus state (between Game and CefUI only)
    pub fn toggle(&mut self) {
        *self = match self {
            InputFocus::Game => InputFocus::CefUI,
            InputFocus::CefUI => InputFocus::Game,
            InputFocus::ChatTyping => InputFocus::Game, // ChatTyping toggle = close
            InputFocus::EmbedFocus => InputFocus::Game, // EmbedFocus toggle = close
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

    /// Set focus to chat typing mode (Feature 032)
    pub fn give_chat_focus(&mut self) {
        *self = InputFocus::ChatTyping;
    }

    /// Release chat focus and return to game (Feature 032)
    pub fn release_chat_focus(&mut self) {
        if *self == InputFocus::ChatTyping {
            *self = InputFocus::Game;
        }
    }

    /// Set focus to embed panel (Feature 033)
    pub fn give_embed_focus(&mut self) {
        *self = InputFocus::EmbedFocus;
    }

    /// Release embed focus and return to game (Feature 033)
    pub fn release_embed_focus(&mut self) {
        if *self == InputFocus::EmbedFocus {
            *self = InputFocus::Game;
        }
    }

    /// Force return to Game state (recovery from stuck states, window focus loss)
    /// (Feature 032 - T016, Feature 033 - T060)
    pub fn force_game_focus(&mut self) {
        *self = InputFocus::Game;
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
        assert!(!focus.is_chat_typing());
        assert!(!focus.is_embed_focus());
        assert!(!focus.blocks_gameplay());
    }

    #[test]
    fn test_cef_focus() {
        let focus = InputFocus::CefUI;
        assert!(focus.is_cef_focused());
        assert!(!focus.is_game_focused());
        assert!(!focus.is_chat_typing());
        assert!(!focus.is_embed_focus());
        assert!(focus.blocks_gameplay());
    }

    #[test]
    fn test_chat_typing_focus() {
        let focus = InputFocus::ChatTyping;
        assert!(!focus.is_cef_focused());
        assert!(!focus.is_game_focused());
        assert!(focus.is_chat_typing());
        assert!(!focus.is_embed_focus());
        assert!(focus.blocks_gameplay());
    }

    #[test]
    fn test_embed_focus() {
        let focus = InputFocus::EmbedFocus;
        assert!(!focus.is_cef_focused());
        assert!(!focus.is_game_focused());
        assert!(!focus.is_chat_typing());
        assert!(focus.is_embed_focus());
        assert!(focus.blocks_gameplay());
    }

    #[test]
    fn test_toggle() {
        let mut focus = InputFocus::Game;

        focus.toggle();
        assert_eq!(focus, InputFocus::CefUI);

        focus.toggle();
        assert_eq!(focus, InputFocus::Game);

        // Toggle from ChatTyping should return to Game
        focus = InputFocus::ChatTyping;
        focus.toggle();
        assert_eq!(focus, InputFocus::Game);

        // Toggle from EmbedFocus should return to Game
        focus = InputFocus::EmbedFocus;
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

    #[test]
    fn test_give_chat_focus() {
        let mut focus = InputFocus::Game;
        focus.give_chat_focus();
        assert_eq!(focus, InputFocus::ChatTyping);

        // Should be idempotent
        focus.give_chat_focus();
        assert_eq!(focus, InputFocus::ChatTyping);
    }

    #[test]
    fn test_release_chat_focus() {
        let mut focus = InputFocus::ChatTyping;
        focus.release_chat_focus();
        assert_eq!(focus, InputFocus::Game);

        // Should not change if not in ChatTyping
        let mut focus = InputFocus::CefUI;
        focus.release_chat_focus();
        assert_eq!(focus, InputFocus::CefUI);
    }

    #[test]
    fn test_give_embed_focus() {
        let mut focus = InputFocus::Game;
        focus.give_embed_focus();
        assert_eq!(focus, InputFocus::EmbedFocus);

        // Should be idempotent
        focus.give_embed_focus();
        assert_eq!(focus, InputFocus::EmbedFocus);
    }

    #[test]
    fn test_release_embed_focus() {
        let mut focus = InputFocus::EmbedFocus;
        focus.release_embed_focus();
        assert_eq!(focus, InputFocus::Game);

        // Should not change if not in EmbedFocus
        let mut focus = InputFocus::CefUI;
        focus.release_embed_focus();
        assert_eq!(focus, InputFocus::CefUI);
    }

    #[test]
    fn test_force_game_focus() {
        let mut focus = InputFocus::CefUI;
        focus.force_game_focus();
        assert_eq!(focus, InputFocus::Game);

        focus = InputFocus::ChatTyping;
        focus.force_game_focus();
        assert_eq!(focus, InputFocus::Game);

        focus = InputFocus::EmbedFocus;
        focus.force_game_focus();
        assert_eq!(focus, InputFocus::Game);

        // Should be idempotent
        focus.force_game_focus();
        assert_eq!(focus, InputFocus::Game);
    }
}
