//! Performance test scenarios
//!
//! Defines reproducible scenarios for performance profiling.

use std::time::Duration;

/// Performance test scenario
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfScenario {
    /// Empty server with minimal world - baseline overhead measurement
    Idle,
    /// Rapid chunk loading/unloading with block edits - meshing stress test
    WorldChurn,
    /// Multiple simulated players with rapid movement - network bandwidth test
    NetStress,
}

impl PerfScenario {
    /// Parse scenario from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "idle" => Some(Self::Idle),
            "world_churn" | "worldchurn" => Some(Self::WorldChurn),
            "net_stress" | "netstress" => Some(Self::NetStress),
            _ => None,
        }
    }

    /// Get scenario name for reports
    pub fn name(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::WorldChurn => "world_churn",
            Self::NetStress => "net_stress",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Idle => "Empty server with minimal world - baseline overhead",
            Self::WorldChurn => "Rapid chunk loading/unloading - meshing stress",
            Self::NetStress => "Multiple players with rapid movement - network bandwidth",
        }
    }

    /// Get default duration for this scenario
    pub fn default_duration(&self) -> Duration {
        Duration::from_secs(60)
    }

    /// Get default player count for this scenario
    pub fn default_player_count(&self) -> u32 {
        match self {
            Self::Idle => 0,
            Self::WorldChurn => 0,
            Self::NetStress => 16,
        }
    }

    /// Get scenario configuration
    pub fn config(&self) -> ScenarioConfig {
        match self {
            Self::Idle => ScenarioConfig {
                name: self.name(),
                description: self.description(),
                duration: self.default_duration(),
                player_count: 0,
                chunk_count: 1,
                block_edits_per_sec: 0,
                movement_updates_per_sec: 0,
            },
            Self::WorldChurn => ScenarioConfig {
                name: self.name(),
                description: self.description(),
                duration: self.default_duration(),
                player_count: 0,
                chunk_count: 100,
                block_edits_per_sec: 60, // 1 per tick
                movement_updates_per_sec: 0,
            },
            Self::NetStress => ScenarioConfig {
                name: self.name(),
                description: self.description(),
                duration: self.default_duration(),
                player_count: 16,
                chunk_count: 10,
                block_edits_per_sec: 0,
                movement_updates_per_sec: 960, // 60 per player per second
            },
        }
    }

    /// List all available scenarios
    pub fn all() -> &'static [Self] {
        &[Self::Idle, Self::WorldChurn, Self::NetStress]
    }
}

impl std::fmt::Display for PerfScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Scenario configuration
#[derive(Debug, Clone)]
pub struct ScenarioConfig {
    /// Scenario name
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Duration of the test
    pub duration: Duration,
    /// Number of simulated players
    pub player_count: u32,
    /// Number of chunks to load
    pub chunk_count: u32,
    /// Block edits per second
    pub block_edits_per_sec: u32,
    /// Movement updates per second
    pub movement_updates_per_sec: u32,
}

impl ScenarioConfig {
    /// Get total actions per tick at 60 TPS
    pub fn actions_per_tick(&self) -> u32 {
        let tps = 60u32;
        (self.block_edits_per_sec + self.movement_updates_per_sec) / tps
    }
}

/// Scenario action to perform during a test
#[derive(Debug, Clone)]
pub enum ScenarioAction {
    /// Wait for a duration
    Wait { duration: Duration },
    /// Spawn simulated players
    SpawnPlayers { count: u32 },
    /// Simulate player movement
    MovePlayer { player_id: u32, direction: [f32; 3] },
    /// Edit a block
    EditBlock { pos: [i32; 3], block_id: u8 },
    /// Force chunk loading
    LoadChunks { count: u32 },
    /// Force chunk unloading
    UnloadChunks { count: u32 },
}

/// Scenario runner state
#[derive(Debug)]
pub struct ScenarioRunner {
    scenario: PerfScenario,
    config: ScenarioConfig,
    elapsed: Duration,
    actions_executed: u64,
}

impl ScenarioRunner {
    /// Create a new scenario runner
    pub fn new(scenario: PerfScenario) -> Self {
        let config = scenario.config();
        Self {
            scenario,
            config,
            elapsed: Duration::ZERO,
            actions_executed: 0,
        }
    }

    /// Create with custom duration
    pub fn with_duration(scenario: PerfScenario, duration: Duration) -> Self {
        let mut config = scenario.config();
        config.duration = duration;
        Self {
            scenario,
            config,
            elapsed: Duration::ZERO,
            actions_executed: 0,
        }
    }

    /// Check if the scenario is complete
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.config.duration
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        self.config.duration.saturating_sub(self.elapsed)
    }

    /// Get progress as percentage (0.0 - 1.0)
    pub fn progress(&self) -> f64 {
        let total = self.config.duration.as_secs_f64();
        if total == 0.0 {
            1.0
        } else {
            (self.elapsed.as_secs_f64() / total).min(1.0)
        }
    }

    /// Advance the scenario by one tick
    pub fn tick(&mut self, tick_duration: Duration) -> Vec<ScenarioAction> {
        self.elapsed += tick_duration;

        if self.is_complete() {
            return Vec::new();
        }

        let mut actions = Vec::new();

        // Generate actions based on scenario
        match self.scenario {
            PerfScenario::Idle => {
                // No actions - just measure baseline
            }
            PerfScenario::WorldChurn => {
                // Simulate block edits every tick
                if self.actions_executed % 60 < self.config.block_edits_per_sec as u64 {
                    let tick = self.actions_executed as i32;
                    actions.push(ScenarioAction::EditBlock {
                        pos: [tick % 16, 64, tick % 16],
                        block_id: ((tick % 5) + 1) as u8,
                    });
                }
            }
            PerfScenario::NetStress => {
                // Simulate rapid movement for each player
                for player_id in 0..self.config.player_count {
                    let angle = (self.actions_executed as f32 * 0.1 + player_id as f32).sin();
                    actions.push(ScenarioAction::MovePlayer {
                        player_id,
                        direction: [angle, 0.0, angle.cos()],
                    });
                }
            }
        }

        self.actions_executed += 1;
        actions
    }

    /// Get the scenario type
    pub fn scenario(&self) -> PerfScenario {
        self.scenario
    }

    /// Get the scenario configuration
    pub fn config(&self) -> &ScenarioConfig {
        &self.config
    }

    /// Get total actions executed
    pub fn actions_executed(&self) -> u64 {
        self.actions_executed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_from_str() {
        assert_eq!(PerfScenario::from_str("idle"), Some(PerfScenario::Idle));
        assert_eq!(
            PerfScenario::from_str("world_churn"),
            Some(PerfScenario::WorldChurn)
        );
        assert_eq!(
            PerfScenario::from_str("net_stress"),
            Some(PerfScenario::NetStress)
        );
        assert_eq!(PerfScenario::from_str("invalid"), None);
    }

    #[test]
    fn test_scenario_names() {
        assert_eq!(PerfScenario::Idle.name(), "idle");
        assert_eq!(PerfScenario::WorldChurn.name(), "world_churn");
        assert_eq!(PerfScenario::NetStress.name(), "net_stress");
    }

    #[test]
    fn test_scenario_configs() {
        let idle = PerfScenario::Idle.config();
        assert_eq!(idle.player_count, 0);
        assert_eq!(idle.block_edits_per_sec, 0);

        let churn = PerfScenario::WorldChurn.config();
        assert_eq!(churn.chunk_count, 100);
        assert!(churn.block_edits_per_sec > 0);

        let stress = PerfScenario::NetStress.config();
        assert!(stress.player_count > 0);
        assert!(stress.movement_updates_per_sec > 0);
    }

    #[test]
    fn test_runner_progress() {
        let mut runner = ScenarioRunner::with_duration(PerfScenario::Idle, Duration::from_secs(10));

        assert!(!runner.is_complete());
        assert!((runner.progress() - 0.0).abs() < 0.001);

        runner.tick(Duration::from_secs(5));
        assert!(!runner.is_complete());
        assert!((runner.progress() - 0.5).abs() < 0.001);

        runner.tick(Duration::from_secs(5));
        assert!(runner.is_complete());
        assert!((runner.progress() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_world_churn_actions() {
        let mut runner = ScenarioRunner::new(PerfScenario::WorldChurn);
        let actions = runner.tick(Duration::from_millis(16));

        // Should have some block edit actions
        assert!(actions.iter().any(|a| matches!(a, ScenarioAction::EditBlock { .. })));
    }

    #[test]
    fn test_net_stress_actions() {
        let mut runner = ScenarioRunner::new(PerfScenario::NetStress);
        let actions = runner.tick(Duration::from_millis(16));

        // Should have movement actions for each player
        let move_count = actions
            .iter()
            .filter(|a| matches!(a, ScenarioAction::MovePlayer { .. }))
            .count();
        assert_eq!(move_count as u32, runner.config().player_count);
    }
}
