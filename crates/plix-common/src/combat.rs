//! Combat system configuration
//!
//! Centralized combat parameters shared between client (for animation prediction)
//! and server (for validation).

/// Combat system configuration.
/// Shared between client (for animation prediction) and server (for validation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CombatConfig {
    /// Attack cooldown in ticks (default: 30 = 0.5s at 60Hz)
    pub attack_cooldown_ticks: u32,

    /// Base attack range in blocks (default: 1.8)
    pub attack_range: f32,

    /// Latency tolerance added to attack range (default: 0.15)
    pub attack_range_epsilon: f32,

    /// Knockback velocity impulse in m/s (default: 4.0)
    pub knockback_strength: f32,

    /// Respawn invulnerability duration in ticks (default: 120 = 2.0s at 60Hz)
    pub respawn_invuln_ticks: u32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            attack_cooldown_ticks: 30,  // 0.5 seconds at 60Hz
            attack_range: 1.8,          // blocks
            attack_range_epsilon: 0.15, // blocks
            knockback_strength: 4.0,    // m/s
            respawn_invuln_ticks: 120,  // 2.0 seconds at 60Hz
        }
    }
}

impl CombatConfig {
    /// Get effective attack range including latency tolerance
    #[inline]
    pub fn effective_range(&self) -> f32 {
        self.attack_range + self.attack_range_epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = CombatConfig::default();
        assert_eq!(config.attack_cooldown_ticks, 30);
        assert!((config.attack_range - 1.8).abs() < f32::EPSILON);
        assert!((config.attack_range_epsilon - 0.15).abs() < f32::EPSILON);
        assert!((config.knockback_strength - 4.0).abs() < f32::EPSILON);
        assert_eq!(config.respawn_invuln_ticks, 120);
    }

    #[test]
    fn test_effective_range() {
        let config = CombatConfig::default();
        let expected = 1.8 + 0.15;
        assert!((config.effective_range() - expected).abs() < f32::EPSILON);
    }
}
