//! Training bot implementation

use glam::Vec3;
use plix_common::time::Tick;
use plix_common::types::BotId;

use super::config::BotBehaviorType;

/// Training bot entity
#[derive(Debug, Clone)]
pub struct TrainingBot {
    /// Unique identifier
    pub id: BotId,
    /// Current world position
    pub position: Vec3,
    /// Current health (0-100)
    pub health: u8,
    /// Is bot dead (waiting to respawn)
    pub is_dead: bool,
    /// Tick when bot can respawn (None if alive)
    pub respawn_tick: Option<Tick>,
    /// Initial spawn position (for reset)
    pub spawn_point: Vec3,
    /// Runtime behavior state
    pub behavior: BotBehavior,
}

impl TrainingBot {
    /// Create a new bot at spawn point
    pub fn new(id: BotId, spawn_point: Vec3, behavior_type: BotBehaviorType) -> Self {
        Self {
            id,
            position: spawn_point,
            health: 100,
            is_dead: false,
            respawn_tick: None,
            spawn_point,
            behavior: BotBehavior::new(behavior_type, spawn_point),
        }
    }

    /// Take damage, returns true if killed
    pub fn take_damage(&mut self, amount: u8, respawn_tick: Tick) -> bool {
        if self.is_dead {
            return false;
        }
        self.health = self.health.saturating_sub(amount);
        if self.health == 0 {
            self.is_dead = true;
            self.respawn_tick = Some(respawn_tick);
            true
        } else {
            false
        }
    }

    /// Respawn bot at spawn point
    pub fn respawn(&mut self) {
        self.position = self.spawn_point;
        self.health = 100;
        self.is_dead = false;
        self.respawn_tick = None;
        self.behavior.reset(self.spawn_point);
    }

    /// Update bot position based on behavior
    pub fn update(&mut self, current_tick: Tick) {
        if !self.is_dead {
            let new_pos = self
                .behavior
                .update(self.position, self.spawn_point, current_tick);
            self.position = new_pos;
        }
    }
}

/// Runtime behavior state (varies by behavior type)
#[derive(Debug, Clone)]
pub enum BotBehavior {
    /// Stationary - no movement
    Dummy,

    /// Random slow movement around spawn point
    Roam {
        /// Current movement direction (normalized)
        direction: Vec3,
        /// Next direction change tick
        change_tick: Tick,
        /// Units per tick (default: 0.05)
        move_speed: f32,
        /// Max distance from spawn (default: 5.0)
        max_radius: f32,
    },

    /// Side-to-side oscillation around spawn point
    Strafe {
        /// Current oscillation phase (0..2π)
        phase: f32,
        /// Center point (spawn position)
        center: Vec3,
        /// Oscillation amplitude (default: 3.0)
        radius: f32,
        /// Phase increment per tick (default: 0.05)
        speed: f32,
    },
}

impl BotBehavior {
    /// Create behavior from type
    pub fn new(behavior_type: BotBehaviorType, spawn_point: Vec3) -> Self {
        match behavior_type {
            BotBehaviorType::Dummy => BotBehavior::Dummy,
            BotBehaviorType::Roam => BotBehavior::Roam {
                direction: Vec3::ZERO,
                change_tick: Tick(0),
                move_speed: 0.05,
                max_radius: 5.0,
            },
            BotBehaviorType::Strafe => BotBehavior::Strafe {
                phase: 0.0,
                center: spawn_point,
                radius: 3.0,
                speed: 0.05,
            },
        }
    }

    /// Reset behavior state (on respawn/session reset)
    pub fn reset(&mut self, spawn_point: Vec3) {
        match self {
            BotBehavior::Dummy => {}
            BotBehavior::Roam {
                direction,
                change_tick,
                ..
            } => {
                *direction = Vec3::ZERO;
                *change_tick = Tick(0);
            }
            BotBehavior::Strafe { phase, center, .. } => {
                *phase = 0.0;
                *center = spawn_point;
            }
        }
    }

    /// Update position based on behavior, returns new position
    pub fn update(&mut self, current_pos: Vec3, spawn_point: Vec3, current_tick: Tick) -> Vec3 {
        match self {
            BotBehavior::Dummy => current_pos,

            BotBehavior::Roam {
                direction,
                change_tick,
                move_speed,
                max_radius,
            } => {
                // Change direction periodically
                if current_tick.0 >= change_tick.0 {
                    // Use tick as simple seed for direction variation
                    let angle = (current_tick.0 as f32 * 0.1) % (2.0 * std::f32::consts::PI);
                    *direction = Vec3::new(angle.cos(), 0.0, angle.sin()).normalize_or_zero();
                    *change_tick = Tick(current_tick.0 + 60); // Change every second at 60Hz
                }

                let new_pos = current_pos + *direction * *move_speed;

                // Clamp to max radius from spawn
                let delta = new_pos - spawn_point;
                let horizontal_delta = Vec3::new(delta.x, 0.0, delta.z);
                if horizontal_delta.length() > *max_radius {
                    let clamped = spawn_point + horizontal_delta.normalize_or_zero() * *max_radius;
                    Vec3::new(clamped.x, new_pos.y, clamped.z)
                } else {
                    new_pos
                }
            }

            BotBehavior::Strafe {
                phase,
                center,
                radius,
                speed,
            } => {
                *phase += *speed;
                if *phase > 2.0 * std::f32::consts::PI {
                    *phase -= 2.0 * std::f32::consts::PI;
                }

                // Oscillate left-right (X axis) around center
                let offset = phase.sin() * *radius;
                Vec3::new(center.x + offset, center.y, center.z)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_creation() {
        let spawn = Vec3::new(10.0, 2.0, 10.0);
        let bot = TrainingBot::new(BotId(0), spawn, BotBehaviorType::Dummy);

        assert_eq!(bot.id, BotId(0));
        assert_eq!(bot.position, spawn);
        assert_eq!(bot.health, 100);
        assert!(!bot.is_dead);
        assert!(bot.respawn_tick.is_none());
        assert_eq!(bot.spawn_point, spawn);
    }

    #[test]
    fn test_bot_take_damage() {
        let spawn = Vec3::new(10.0, 2.0, 10.0);
        let mut bot = TrainingBot::new(BotId(0), spawn, BotBehaviorType::Dummy);

        // Partial damage
        let killed = bot.take_damage(30, Tick(100));
        assert!(!killed);
        assert_eq!(bot.health, 70);
        assert!(!bot.is_dead);

        // Lethal damage
        let killed = bot.take_damage(70, Tick(200));
        assert!(killed);
        assert_eq!(bot.health, 0);
        assert!(bot.is_dead);
        assert_eq!(bot.respawn_tick, Some(Tick(200)));
    }

    #[test]
    fn test_bot_respawn() {
        let spawn = Vec3::new(10.0, 2.0, 10.0);
        let mut bot = TrainingBot::new(BotId(0), spawn, BotBehaviorType::Dummy);

        // Kill bot and move position
        bot.take_damage(100, Tick(100));
        bot.position = Vec3::new(20.0, 2.0, 20.0);

        // Respawn
        bot.respawn();
        assert_eq!(bot.position, spawn);
        assert_eq!(bot.health, 100);
        assert!(!bot.is_dead);
        assert!(bot.respawn_tick.is_none());
    }

    #[test]
    fn test_dummy_behavior_no_movement() {
        let spawn = Vec3::new(10.0, 2.0, 10.0);
        let mut behavior = BotBehavior::new(BotBehaviorType::Dummy, spawn);

        // Multiple ticks should not change position
        for tick in 0..100 {
            let new_pos = behavior.update(spawn, spawn, Tick(tick));
            assert_eq!(new_pos, spawn, "Dummy should not move");
        }
    }

    #[test]
    fn test_strafe_behavior_oscillates() {
        let spawn = Vec3::new(10.0, 2.0, 10.0);
        let mut behavior = BotBehavior::new(BotBehaviorType::Strafe, spawn);

        let mut positions = Vec::new();
        let mut pos = spawn;

        // Collect positions over time
        for tick in 0..200 {
            pos = behavior.update(pos, spawn, Tick(tick));
            positions.push(pos);
        }

        // Verify oscillation occurred (X position changed)
        let x_values: Vec<f32> = positions.iter().map(|p| p.x).collect();
        let min_x = x_values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_x = x_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        assert!(max_x - min_x > 1.0, "Strafe should oscillate X position");
    }

    #[test]
    fn test_roam_behavior_stays_in_radius() {
        let spawn = Vec3::new(10.0, 2.0, 10.0);
        let mut behavior = BotBehavior::new(BotBehaviorType::Roam, spawn);

        let mut pos = spawn;
        let max_radius = 5.0;

        // Run many ticks
        for tick in 0..1000 {
            pos = behavior.update(pos, spawn, Tick(tick));

            // Check horizontal distance from spawn
            let delta = pos - spawn;
            let horizontal_dist = (delta.x * delta.x + delta.z * delta.z).sqrt();
            assert!(
                horizontal_dist <= max_radius + 0.1,
                "Roam should stay within radius: dist={}, max={}",
                horizontal_dist,
                max_radius
            );
        }
    }

    #[test]
    fn test_behavior_reset() {
        let spawn = Vec3::new(10.0, 2.0, 10.0);

        // Test Roam reset
        let mut roam = BotBehavior::Roam {
            direction: Vec3::new(1.0, 0.0, 0.0),
            change_tick: Tick(500),
            move_speed: 0.05,
            max_radius: 5.0,
        };
        roam.reset(spawn);
        if let BotBehavior::Roam {
            direction,
            change_tick,
            ..
        } = roam
        {
            assert_eq!(direction, Vec3::ZERO);
            assert_eq!(change_tick, Tick(0));
        }

        // Test Strafe reset
        let mut strafe = BotBehavior::Strafe {
            phase: std::f32::consts::PI,
            center: Vec3::new(100.0, 100.0, 100.0),
            radius: 3.0,
            speed: 0.05,
        };
        strafe.reset(spawn);
        if let BotBehavior::Strafe { phase, center, .. } = strafe {
            assert_eq!(phase, 0.0);
            assert_eq!(center, spawn);
        }
    }
}
