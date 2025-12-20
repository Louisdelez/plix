//! HUD State Publisher (Feature 032)
//!
//! Manages throttled publishing of HUD data (HP, RTT, FPS, crosshair)
//! to the CEF UI overlay.

use serde::Serialize;
use std::time::{Duration, Instant};

/// Minimum interval between HUD updates (66ms = ~15 Hz)
const HUD_UPDATE_INTERVAL_MS: u64 = 66;

/// HUD state snapshot for bridge transmission
#[derive(Debug, Clone, Serialize)]
pub struct HudState {
    /// Current health points (0-100)
    pub hp: u8,
    /// Maximum health points (always 100 for now)
    pub max_hp: u8,
    /// Round-trip time in milliseconds
    pub rtt_ms: u32,
    /// Frames per second (optional, null if disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<u32>,
    /// Whether crosshair should be visible
    #[serde(rename = "crosshairVisible")]
    pub crosshair_visible: bool,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            hp: 100,
            max_hp: 100,
            rtt_ms: 0,
            fps: None,
            crosshair_visible: true,
        }
    }
}

impl HudState {
    /// Create a new HUD state
    pub fn new(hp: u8, rtt_ms: u32, fps: Option<u32>) -> Self {
        Self {
            hp,
            max_hp: 100,
            rtt_ms,
            fps,
            crosshair_visible: true,
        }
    }

    /// Check if HP has changed significantly from another state
    pub fn hp_changed(&self, other: &Self) -> bool {
        self.hp != other.hp
    }
}

/// Throttled HUD state publisher
///
/// Publishes HUD updates at a rate of ~15 Hz, with immediate publish
/// on HP change for responsive damage feedback.
pub struct HudStatePublisher {
    /// Time of last publish
    last_publish: Instant,
    /// Previously published state (for change detection)
    last_state: HudState,
    /// Minimum interval between publishes
    min_interval: Duration,
}

impl Default for HudStatePublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl HudStatePublisher {
    /// Create a new HUD state publisher
    pub fn new() -> Self {
        Self {
            last_publish: Instant::now() - Duration::from_secs(1), // Allow immediate first publish
            last_state: HudState::default(),
            min_interval: Duration::from_millis(HUD_UPDATE_INTERVAL_MS),
        }
    }

    /// Check if HUD state should be published, returning the state if so
    ///
    /// Publishes immediately on HP change, otherwise throttles to ~15 Hz.
    pub fn maybe_publish(&mut self, current: HudState) -> Option<HudState> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_publish);

        // Immediate publish on HP change (damage feedback)
        let hp_changed = current.hp_changed(&self.last_state);

        // Publish if HP changed or interval elapsed
        if hp_changed || elapsed >= self.min_interval {
            self.last_publish = now;
            self.last_state = current.clone();
            Some(current)
        } else {
            None
        }
    }

    /// Force a publish regardless of throttle
    pub fn force_publish(&mut self, current: HudState) -> HudState {
        self.last_publish = Instant::now();
        self.last_state = current.clone();
        current
    }

    /// Get the last published state
    pub fn last_state(&self) -> &HudState {
        &self.last_state
    }

    /// Set the update interval (for testing)
    #[cfg(test)]
    pub fn set_interval(&mut self, interval: Duration) {
        self.min_interval = interval;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_hud_state_default() {
        let state = HudState::default();
        assert_eq!(state.hp, 100);
        assert_eq!(state.max_hp, 100);
        assert_eq!(state.rtt_ms, 0);
        assert!(state.fps.is_none());
        assert!(state.crosshair_visible);
    }

    #[test]
    fn test_hud_state_new() {
        let state = HudState::new(75, 45, Some(60));
        assert_eq!(state.hp, 75);
        assert_eq!(state.rtt_ms, 45);
        assert_eq!(state.fps, Some(60));
    }

    #[test]
    fn test_hud_state_hp_changed() {
        let state1 = HudState::new(100, 0, None);
        let state2 = HudState::new(100, 50, Some(60)); // Same HP, different RTT/FPS
        let state3 = HudState::new(90, 0, None); // Different HP

        assert!(!state1.hp_changed(&state2));
        assert!(state1.hp_changed(&state3));
    }

    #[test]
    fn test_publisher_immediate_first_publish() {
        let mut publisher = HudStatePublisher::new();
        let state = HudState::new(100, 0, None);

        // First publish should always succeed
        let result = publisher.maybe_publish(state);
        assert!(result.is_some());
    }

    #[test]
    fn test_publisher_throttles() {
        let mut publisher = HudStatePublisher::new();
        publisher.set_interval(Duration::from_millis(100));

        let state = HudState::new(100, 0, None);

        // First publish succeeds
        assert!(publisher.maybe_publish(state.clone()).is_some());

        // Immediate second publish should be throttled
        assert!(publisher.maybe_publish(state.clone()).is_none());

        // After interval, should publish again
        sleep(Duration::from_millis(110));
        assert!(publisher.maybe_publish(state).is_some());
    }

    #[test]
    fn test_publisher_immediate_on_hp_change() {
        let mut publisher = HudStatePublisher::new();
        publisher.set_interval(Duration::from_millis(100));

        // First publish
        let state1 = HudState::new(100, 0, None);
        assert!(publisher.maybe_publish(state1).is_some());

        // Immediate HP change should bypass throttle
        let state2 = HudState::new(90, 0, None);
        assert!(publisher.maybe_publish(state2).is_some());

        // Same HP should be throttled
        let state3 = HudState::new(90, 50, None); // Different RTT, same HP
        assert!(publisher.maybe_publish(state3).is_none());
    }

    #[test]
    fn test_publisher_force_publish() {
        let mut publisher = HudStatePublisher::new();
        publisher.set_interval(Duration::from_millis(100));

        let state = HudState::new(100, 0, None);
        publisher.maybe_publish(state.clone());

        // Force publish should work even when throttled
        let forced = publisher.force_publish(state);
        assert_eq!(forced.hp, 100);
    }

    #[test]
    fn test_hud_state_serialize() {
        let state = HudState::new(75, 45, Some(60));
        let json = serde_json::to_string(&state).unwrap();

        assert!(json.contains("\"hp\":75"));
        assert!(json.contains("\"max_hp\":100"));
        assert!(json.contains("\"rtt_ms\":45"));
        assert!(json.contains("\"fps\":60"));
        assert!(json.contains("\"crosshairVisible\":true"));
    }

    #[test]
    fn test_hud_state_serialize_no_fps() {
        let state = HudState::default();
        let json = serde_json::to_string(&state).unwrap();

        // fps should be omitted when None
        assert!(!json.contains("\"fps\""));
    }
}
