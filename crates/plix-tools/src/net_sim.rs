//! Network condition simulator

use std::time::Duration;

use rand::Rng;

/// Network simulation configuration
#[derive(Debug, Clone)]
pub struct NetSimConfig {
    /// Added latency in milliseconds
    pub latency_ms: u32,
    /// Latency jitter in milliseconds
    pub jitter_ms: u32,
    /// Packet loss percentage (0-100)
    pub loss_percent: u8,
    /// Packet reordering percentage (0-100)
    pub reorder_percent: u8,
    /// Packet duplication percentage (0-100)
    pub duplicate_percent: u8,
}

impl Default for NetSimConfig {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            jitter_ms: 0,
            loss_percent: 0,
            reorder_percent: 0,
            duplicate_percent: 0,
        }
    }
}

/// Network condition simulator
#[derive(Debug)]
pub struct NetworkSimulator {
    config: NetSimConfig,
    rng: rand::rngs::ThreadRng,
}

impl NetworkSimulator {
    /// Create a new simulator with the given config
    pub fn new(config: NetSimConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    /// Calculate delay for a packet
    pub fn calculate_delay(&mut self) -> Duration {
        let base_latency = self.config.latency_ms as i32;
        let jitter_range = self.config.jitter_ms as i32;

        let jitter = if jitter_range > 0 {
            self.rng.gen_range(-jitter_range..=jitter_range)
        } else {
            0
        };

        let delay = (base_latency + jitter).max(0) as u64;
        Duration::from_millis(delay)
    }

    /// Determine if a packet should be dropped
    pub fn should_drop(&mut self) -> bool {
        if self.config.loss_percent == 0 {
            return false;
        }
        self.rng.gen_range(0..100) < self.config.loss_percent
    }

    /// Determine if a packet should be duplicated
    pub fn should_duplicate(&mut self) -> bool {
        if self.config.duplicate_percent == 0 {
            return false;
        }
        self.rng.gen_range(0..100) < self.config.duplicate_percent
    }

    /// Determine if a packet should be reordered
    pub fn should_reorder(&mut self) -> bool {
        if self.config.reorder_percent == 0 {
            return false;
        }
        self.rng.gen_range(0..100) < self.config.reorder_percent
    }

    /// Process a packet and return (delay, drop, duplicate)
    pub fn process_packet(&mut self) -> (Duration, bool, bool) {
        let delay = self.calculate_delay();
        let drop = self.should_drop();
        let duplicate = !drop && self.should_duplicate();

        (delay, drop, duplicate)
    }

    /// Get current config
    pub fn config(&self) -> &NetSimConfig {
        &self.config
    }

    /// Update config
    pub fn set_config(&mut self, config: NetSimConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_simulation() {
        let mut sim = NetworkSimulator::new(NetSimConfig::default());

        let (delay, drop, dup) = sim.process_packet();
        assert_eq!(delay, Duration::ZERO);
        assert!(!drop);
        assert!(!dup);
    }

    #[test]
    fn test_latency() {
        let config = NetSimConfig {
            latency_ms: 100,
            jitter_ms: 0,
            ..Default::default()
        };
        let mut sim = NetworkSimulator::new(config);

        let (delay, _, _) = sim.process_packet();
        assert_eq!(delay, Duration::from_millis(100));
    }

    #[test]
    fn test_packet_loss() {
        let config = NetSimConfig {
            loss_percent: 100,
            ..Default::default()
        };
        let mut sim = NetworkSimulator::new(config);

        // Should always drop with 100% loss
        assert!(sim.should_drop());
    }
}
