//! UI state machine for menu navigation

use crate::config::Action;

/// Current UI state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiState {
    /// Normal gameplay - cursor grabbed, crosshair visible
    InGame,

    /// Pause menu open - cursor released, crosshair hidden
    PauseMenu,

    /// Settings menu (sub-menu of Paused)
    Settings,

    /// Keybinds submenu
    Keybinds,

    /// Waiting for key input to rebind an action
    Rebinding(Action),

    /// Confirming swap of conflicting keybind
    ConfirmSwap {
        /// Action being rebound
        action: Action,
        /// New key that was pressed
        new_key: crate::config::Key,
        /// Action that currently has that key
        conflicting_action: Action,
    },
}

impl Default for UiState {
    fn default() -> Self {
        Self::InGame
    }
}

impl UiState {
    /// Returns true if gameplay inputs should be processed
    pub fn should_process_gameplay_input(&self) -> bool {
        matches!(self, UiState::InGame)
    }

    /// Returns true if cursor should be grabbed
    pub fn should_grab_cursor(&self) -> bool {
        matches!(self, UiState::InGame)
    }

    /// Returns true if crosshair should be visible
    pub fn should_show_crosshair(&self) -> bool {
        matches!(self, UiState::InGame)
    }
}
