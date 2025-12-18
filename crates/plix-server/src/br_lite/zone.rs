//! BR Lite zone controller - handles shrinking safe zone

use glam::{Vec2, Vec3};
use tracing::info;

use super::config::{BrLiteConfig, ZonePhase};
use super::state::{PhaseMode, ZoneState};

/// Server tick rate for time conversion
const TICK_RATE: u32 = 60;

/// Controller for the shrinking zone
#[derive(Debug)]
pub struct ZoneController {
    /// Zone phases configuration
    phases: Vec<ZonePhase>,
    /// Initial radius (before any shrinking)
    initial_radius: f32,
    /// Whether this is the final phase
    is_final_phase: bool,
}

impl ZoneController {
    /// Create a new zone controller from config
    pub fn new(config: &BrLiteConfig, arena_size: Vec2) -> Self {
        let initial_radius = config
            .initial_radius
            .unwrap_or_else(|| arena_size.x.min(arena_size.y) / 2.0);

        Self {
            phases: config.phases.clone(),
            initial_radius,
            is_final_phase: false,
        }
    }

    /// Initialize zone state for a new match
    pub fn init_state(&self, center: Vec2) -> ZoneState {
        let first_phase = self.phases.first();
        let damage = first_phase.map(|p| p.damage_per_tick).unwrap_or(5);
        let stable_duration = first_phase.map(|p| p.stable_duration_secs).unwrap_or(60);

        ZoneState {
            center,
            current_radius: self.initial_radius,
            target_radius: self.initial_radius,
            start_radius: self.initial_radius,
            phase_index: 0,
            phase_mode: PhaseMode::Stable,
            phase_timer: stable_duration * TICK_RATE,
            phase_duration_ticks: stable_duration * TICK_RATE,
            damage_per_tick: damage,
        }
    }

    /// Tick the zone controller - update state
    /// Returns true if phase changed (for event broadcasting)
    pub fn tick(&mut self, state: &mut ZoneState) -> bool {
        if self.is_final_phase && state.phase_mode == PhaseMode::Stable {
            // Final phase stable - stay here forever
            return false;
        }

        if state.phase_timer > 0 {
            state.phase_timer -= 1;

            // Interpolate radius during shrink
            if state.phase_mode == PhaseMode::Shrinking && state.phase_duration_ticks > 0 {
                let elapsed = state.phase_duration_ticks - state.phase_timer;
                let progress = elapsed as f32 / state.phase_duration_ticks as f32;
                state.current_radius = lerp(state.start_radius, state.target_radius, progress);
            }

            return false;
        }

        // Timer expired - transition
        self.transition_phase(state)
    }

    /// Transition to next phase/mode
    fn transition_phase(&mut self, state: &mut ZoneState) -> bool {
        match state.phase_mode {
            PhaseMode::Stable => {
                // Stable → Shrinking
                let phase = &self.phases[state.phase_index];
                state.phase_mode = PhaseMode::Shrinking;
                state.phase_timer = phase.shrink_duration_secs * TICK_RATE;
                state.phase_duration_ticks = phase.shrink_duration_secs * TICK_RATE;
                state.start_radius = state.current_radius;

                // Calculate target radius (end_radius is a fraction of initial)
                let target = if phase.end_radius <= 1.0 {
                    // Fraction of initial radius
                    self.initial_radius * phase.end_radius
                } else {
                    // Absolute radius
                    phase.end_radius
                };
                state.target_radius = target;

                info!(
                    target: "br_lite",
                    phase = state.phase_index,
                    from_radius = state.current_radius,
                    to_radius = target,
                    "shrink_start"
                );

                true
            }
            PhaseMode::Shrinking => {
                // Shrinking → Stable (next phase) or final
                state.current_radius = state.target_radius;
                state.phase_index += 1;

                if state.phase_index >= self.phases.len() {
                    // Stay in final phase forever
                    self.is_final_phase = true;
                    state.phase_mode = PhaseMode::Stable;
                    state.phase_timer = u32::MAX;
                    state.phase_duration_ticks = u32::MAX;

                    info!(
                        target: "br_lite",
                        radius = state.current_radius,
                        "final_phase_reached"
                    );
                } else {
                    // Move to next phase
                    let next_phase = &self.phases[state.phase_index];
                    state.phase_mode = PhaseMode::Stable;
                    state.phase_timer = next_phase.stable_duration_secs * TICK_RATE;
                    state.phase_duration_ticks = next_phase.stable_duration_secs * TICK_RATE;
                    state.damage_per_tick = next_phase.damage_per_tick;

                    info!(
                        target: "br_lite",
                        phase = state.phase_index,
                        radius = state.current_radius,
                        damage = state.damage_per_tick,
                        "phase_change"
                    );
                }

                true
            }
        }
    }

    /// Check if a position is inside the safe zone
    pub fn is_in_zone(&self, player_pos: Vec3, state: &ZoneState) -> bool {
        let dx = player_pos.x - state.center.x;
        let dz = player_pos.z - state.center.y; // Vec2.y is Z coordinate
        let dist_sq = dx * dx + dz * dz;
        dist_sq < state.current_radius * state.current_radius
    }

    /// Get initial radius
    pub fn initial_radius(&self) -> f32 {
        self.initial_radius
    }
}

/// Linear interpolation helper
fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> BrLiteConfig {
        BrLiteConfig {
            phases: vec![
                ZonePhase::new(10, 5, 0.8, 5),  // 10s stable, 5s shrink to 80%
                ZonePhase::new(10, 5, 0.5, 10), // 10s stable, 5s shrink to 50%
            ],
            initial_radius: Some(100.0),
            ..Default::default()
        }
    }

    #[test]
    fn test_zone_controller_creation() {
        let config = test_config();
        let controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        assert_eq!(controller.initial_radius, 100.0);
    }

    #[test]
    fn test_zone_state_initialization() {
        let config = test_config();
        let controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let state = controller.init_state(Vec2::new(32.0, 32.0));

        assert_eq!(state.center, Vec2::new(32.0, 32.0));
        assert_eq!(state.current_radius, 100.0);
        assert_eq!(state.phase_mode, PhaseMode::Stable);
        assert_eq!(state.phase_index, 0);
        assert_eq!(state.damage_per_tick, 5);
    }

    #[test]
    fn test_stable_to_shrink_transition() {
        let config = test_config();
        let mut controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let mut state = controller.init_state(Vec2::new(32.0, 32.0));

        // Fast forward through stable phase
        state.phase_timer = 1;
        let changed = controller.tick(&mut state);
        assert!(!changed); // Timer just decremented

        let changed = controller.tick(&mut state);
        assert!(changed); // Now transitioned
        assert_eq!(state.phase_mode, PhaseMode::Shrinking);
        assert_eq!(state.target_radius, 80.0); // 80% of 100
    }

    #[test]
    fn test_radius_interpolation() {
        let config = test_config();
        let mut controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let mut state = controller.init_state(Vec2::new(32.0, 32.0));

        // Transition to shrinking
        state.phase_timer = 0;
        controller.tick(&mut state);

        // Halfway through shrink (5 seconds = 300 ticks, so 150 ticks remaining)
        state.phase_timer = 150;
        controller.tick(&mut state);

        // Should be interpolated to ~90 (halfway between 100 and 80)
        assert!((state.current_radius - 90.0).abs() < 1.0);
    }

    #[test]
    fn test_phase_advancement() {
        let config = test_config();
        let mut controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let mut state = controller.init_state(Vec2::new(32.0, 32.0));

        // Go through first stable phase
        state.phase_timer = 0;
        controller.tick(&mut state);
        assert_eq!(state.phase_mode, PhaseMode::Shrinking);

        // Complete shrinking
        state.phase_timer = 0;
        state.current_radius = 80.0;
        controller.tick(&mut state);

        // Should be in phase 1, stable
        assert_eq!(state.phase_index, 1);
        assert_eq!(state.phase_mode, PhaseMode::Stable);
        assert_eq!(state.damage_per_tick, 10);
    }

    #[test]
    fn test_final_phase() {
        let config = test_config();
        let mut controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let mut state = controller.init_state(Vec2::new(32.0, 32.0));

        // Fast forward to final phase
        state.phase_index = 1;
        state.phase_mode = PhaseMode::Shrinking;
        state.phase_timer = 0;
        state.current_radius = 50.0;
        controller.tick(&mut state);

        // Should be in final phase
        assert!(controller.is_final_phase);
        assert_eq!(state.phase_mode, PhaseMode::Stable);
        assert_eq!(state.phase_timer, u32::MAX);
    }

    #[test]
    fn test_is_in_zone() {
        let config = test_config();
        let controller = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let state = controller.init_state(Vec2::new(32.0, 32.0));

        // Player at center - in zone
        assert!(controller.is_in_zone(Vec3::new(32.0, 5.0, 32.0), &state));

        // Player near edge - in zone
        assert!(controller.is_in_zone(Vec3::new(32.0 + 95.0, 5.0, 32.0), &state));

        // Player outside zone
        assert!(!controller.is_in_zone(Vec3::new(32.0 + 150.0, 5.0, 32.0), &state));
    }

    #[test]
    fn test_determinism() {
        let config = test_config();
        let mut controller1 = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let mut controller2 = ZoneController::new(&config, Vec2::new(64.0, 64.0));
        let mut state1 = controller1.init_state(Vec2::new(32.0, 32.0));
        let mut state2 = controller2.init_state(Vec2::new(32.0, 32.0));

        // Run same number of ticks
        for _ in 0..1000 {
            controller1.tick(&mut state1);
            controller2.tick(&mut state2);
        }

        // Should be identical
        assert_eq!(state1.current_radius, state2.current_radius);
        assert_eq!(state1.phase_index, state2.phase_index);
        assert_eq!(state1.phase_mode, state2.phase_mode);
    }
}
