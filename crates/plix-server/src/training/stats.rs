//! Training statistics tracking

use plix_common::time::Tick;

/// Training session statistics
#[derive(Debug, Clone, Default)]
pub struct TrainingStats {
    /// Total hits on bots
    pub hits: u32,
    /// Total bot kills
    pub kills: u32,
    /// Total attack attempts
    pub attacks: u32,
    /// When session started
    pub session_start: Tick,
}

impl TrainingStats {
    /// Create new stats with given start tick
    pub fn new(start_tick: Tick) -> Self {
        Self {
            hits: 0,
            kills: 0,
            attacks: 0,
            session_start: start_tick,
        }
    }

    /// Record an attack attempt
    pub fn record_attack(&mut self) {
        self.attacks += 1;
    }

    /// Record a hit on a bot
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a bot kill
    pub fn record_kill(&mut self) {
        self.kills += 1;
    }

    /// Calculate accuracy percentage (0.0 - 100.0)
    pub fn accuracy(&self) -> f32 {
        if self.attacks == 0 {
            0.0
        } else {
            (self.hits as f32 / self.attacks as f32) * 100.0
        }
    }

    /// Calculate session duration in seconds
    pub fn duration_secs(&self, current_tick: Tick, tick_rate: u8) -> f32 {
        let elapsed_ticks = current_tick.0.saturating_sub(self.session_start.0);
        elapsed_ticks as f32 / tick_rate as f32
    }

    /// Reset all stats
    pub fn reset(&mut self, start_tick: Tick) {
        self.hits = 0;
        self.kills = 0;
        self.attacks = 0;
        self.session_start = start_tick;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_creation() {
        let stats = TrainingStats::new(Tick(100));
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.kills, 0);
        assert_eq!(stats.attacks, 0);
        assert_eq!(stats.session_start, Tick(100));
    }

    #[test]
    fn test_record_attack() {
        let mut stats = TrainingStats::new(Tick(0));
        stats.record_attack();
        stats.record_attack();
        stats.record_attack();
        assert_eq!(stats.attacks, 3);
    }

    #[test]
    fn test_record_hit() {
        let mut stats = TrainingStats::new(Tick(0));
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.hits, 2);
    }

    #[test]
    fn test_record_kill() {
        let mut stats = TrainingStats::new(Tick(0));
        stats.record_kill();
        assert_eq!(stats.kills, 1);
    }

    #[test]
    fn test_accuracy_no_attacks() {
        let stats = TrainingStats::new(Tick(0));
        assert_eq!(stats.accuracy(), 0.0);
    }

    #[test]
    fn test_accuracy_all_hits() {
        let mut stats = TrainingStats::new(Tick(0));
        stats.attacks = 10;
        stats.hits = 10;
        assert_eq!(stats.accuracy(), 100.0);
    }

    #[test]
    fn test_accuracy_partial_hits() {
        let mut stats = TrainingStats::new(Tick(0));
        stats.attacks = 10;
        stats.hits = 5;
        assert_eq!(stats.accuracy(), 50.0);
    }

    #[test]
    fn test_accuracy_no_hits() {
        let mut stats = TrainingStats::new(Tick(0));
        stats.attacks = 10;
        stats.hits = 0;
        assert_eq!(stats.accuracy(), 0.0);
    }

    #[test]
    fn test_duration_secs() {
        let stats = TrainingStats::new(Tick(0));

        // At 60Hz, 60 ticks = 1 second
        let duration = stats.duration_secs(Tick(60), 60);
        assert!((duration - 1.0).abs() < 0.001);

        // At 60Hz, 300 ticks = 5 seconds
        let duration = stats.duration_secs(Tick(300), 60);
        assert!((duration - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_duration_secs_with_offset() {
        let stats = TrainingStats::new(Tick(100));

        // Started at tick 100, now at tick 160 (60 ticks elapsed = 1 second @ 60Hz)
        let duration = stats.duration_secs(Tick(160), 60);
        assert!((duration - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_reset() {
        let mut stats = TrainingStats::new(Tick(0));
        stats.hits = 50;
        stats.kills = 10;
        stats.attacks = 100;

        stats.reset(Tick(500));

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.kills, 0);
        assert_eq!(stats.attacks, 0);
        assert_eq!(stats.session_start, Tick(500));
    }

    #[test]
    fn test_stats_default() {
        let stats = TrainingStats::default();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.kills, 0);
        assert_eq!(stats.attacks, 0);
        assert_eq!(stats.session_start, Tick(0));
    }
}
