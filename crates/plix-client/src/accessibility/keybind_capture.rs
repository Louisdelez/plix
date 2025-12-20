//! Keybind capture state machine
//!
//! Handles keybind remapping with 5-second timeout and conflict detection.

use std::time::{Duration, Instant};

use crate::config::{Action, Key, Keybinds};

/// Keybind capture timeout duration
pub const CAPTURE_TIMEOUT: Duration = Duration::from_secs(5);

/// State machine for keybind capture
#[derive(Debug, Clone)]
pub enum KeybindCaptureState {
    /// Not currently capturing
    Idle,

    /// Listening for key input
    Listening {
        /// The action being rebound
        action: Action,
        /// When the capture started
        started_at: Instant,
    },

    /// A conflict was detected, awaiting resolution
    Conflict {
        /// The conflict details
        conflict: KeybindConflict,
    },
}

impl Default for KeybindCaptureState {
    fn default() -> Self {
        Self::Idle
    }
}

impl KeybindCaptureState {
    /// Create a new capture state
    pub fn new() -> Self {
        Self::Idle
    }

    /// Start listening for a key input
    pub fn start_capture(&mut self, action: Action) {
        *self = Self::Listening {
            action,
            started_at: Instant::now(),
        };
    }

    /// Check if we're currently listening
    pub fn is_listening(&self) -> bool {
        matches!(self, Self::Listening { .. })
    }

    /// Check if there's a pending conflict
    pub fn has_conflict(&self) -> bool {
        matches!(self, Self::Conflict { .. })
    }

    /// Get the action being captured (if listening)
    pub fn capturing_action(&self) -> Option<Action> {
        match self {
            Self::Listening { action, .. } => Some(*action),
            _ => None,
        }
    }

    /// Get the pending conflict (if any)
    pub fn pending_conflict(&self) -> Option<&KeybindConflict> {
        match self {
            Self::Conflict { conflict } => Some(conflict),
            _ => None,
        }
    }

    /// Check if the capture has timed out (5 seconds)
    pub fn is_timed_out(&self) -> bool {
        match self {
            Self::Listening { started_at, .. } => started_at.elapsed() >= CAPTURE_TIMEOUT,
            _ => false,
        }
    }

    /// Get remaining time in capture (for UI display)
    pub fn remaining_time(&self) -> Option<Duration> {
        match self {
            Self::Listening { started_at, .. } => {
                let elapsed = started_at.elapsed();
                if elapsed >= CAPTURE_TIMEOUT {
                    Some(Duration::ZERO)
                } else {
                    Some(CAPTURE_TIMEOUT - elapsed)
                }
            }
            _ => None,
        }
    }

    /// Set a conflict state
    pub fn set_conflict(&mut self, conflict: KeybindConflict) {
        *self = Self::Conflict { conflict };
    }

    /// Cancel capture and return to idle
    pub fn cancel(&mut self) {
        *self = Self::Idle;
    }

    /// Resolve conflict and return to idle
    pub fn resolve(&mut self) {
        *self = Self::Idle;
    }
}

/// A keybinding conflict between two actions
#[derive(Debug, Clone)]
pub struct KeybindConflict {
    /// The action the user is trying to rebind
    pub target_action: Action,

    /// The action that currently has the conflicting key
    pub conflicting_action: Action,

    /// The key that is causing the conflict
    pub key: Key,
}

impl KeybindConflict {
    /// Create a new keybind conflict
    pub fn new(target: Action, conflicting: Action, key: Key) -> Self {
        Self {
            target_action: target,
            conflicting_action: conflicting,
            key,
        }
    }
}

/// Detect if a key is already bound to another action
pub fn detect_conflict(
    keybinds: &Keybinds,
    target_action: Action,
    new_key: Key,
) -> Option<KeybindConflict> {
    if let Some(existing_action) = keybinds.action_for_key(new_key) {
        // If the key is bound to a different action, that's a conflict
        if existing_action != target_action {
            return Some(KeybindConflict::new(target_action, existing_action, new_key));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_capture_state_new() {
        let state = KeybindCaptureState::new();
        assert!(matches!(state, KeybindCaptureState::Idle));
        assert!(!state.is_listening());
        assert!(!state.has_conflict());
    }

    #[test]
    fn test_capture_state_start_capture() {
        let mut state = KeybindCaptureState::new();
        state.start_capture(Action::Forward);
        assert!(state.is_listening());
        assert_eq!(state.capturing_action(), Some(Action::Forward));
    }

    #[test]
    fn test_capture_state_cancel() {
        let mut state = KeybindCaptureState::new();
        state.start_capture(Action::Forward);
        assert!(state.is_listening());
        state.cancel();
        assert!(!state.is_listening());
        assert!(matches!(state, KeybindCaptureState::Idle));
    }

    #[test]
    fn test_capture_state_timeout() {
        let mut state = KeybindCaptureState::new();
        state.start_capture(Action::Forward);

        // Should not be timed out immediately
        assert!(!state.is_timed_out());

        // Remaining time should be close to 5 seconds
        let remaining = state.remaining_time().unwrap();
        assert!(remaining <= CAPTURE_TIMEOUT);
        assert!(remaining > Duration::from_secs(4));
    }

    #[test]
    fn test_capture_state_conflict() {
        let mut state = KeybindCaptureState::new();
        let conflict = KeybindConflict::new(Action::Jump, Action::Forward, Key::W);
        state.set_conflict(conflict);

        assert!(state.has_conflict());
        assert!(!state.is_listening());

        let pending = state.pending_conflict().unwrap();
        assert_eq!(pending.target_action, Action::Jump);
        assert_eq!(pending.conflicting_action, Action::Forward);
        assert_eq!(pending.key, Key::W);
    }

    #[test]
    fn test_detect_conflict() {
        let keybinds = Keybinds::default();

        // W is bound to Forward, trying to bind Jump to W should conflict
        let conflict = detect_conflict(&keybinds, Action::Jump, Key::W);
        assert!(conflict.is_some());
        let conflict = conflict.unwrap();
        assert_eq!(conflict.target_action, Action::Jump);
        assert_eq!(conflict.conflicting_action, Action::Forward);
        assert_eq!(conflict.key, Key::W);

        // Binding Forward to W (same action) should not conflict
        let no_conflict = detect_conflict(&keybinds, Action::Forward, Key::W);
        assert!(no_conflict.is_none());

        // Binding to an unbound key should not conflict
        let no_conflict = detect_conflict(&keybinds, Action::Jump, Key::G);
        assert!(no_conflict.is_none());
    }

    #[test]
    fn test_keybind_conflict_new() {
        let conflict = KeybindConflict::new(Action::Jump, Action::Forward, Key::W);
        assert_eq!(conflict.target_action, Action::Jump);
        assert_eq!(conflict.conflicting_action, Action::Forward);
        assert_eq!(conflict.key, Key::W);
    }
}
