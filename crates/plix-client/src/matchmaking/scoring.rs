//! Server scoring algorithm for matchmaking.
//!
//! Scores servers based on multiple criteria to select the best match.
//! Higher scores indicate better servers for the player's request.

use plix_common::server_browser::ServerEntry;

use super::QuickJoinRequest;

/// Scoring constants (from FR-009 to FR-011).
pub mod weights {
    /// Bonus for matching region (+50 points).
    pub const REGION_BONUS: i32 = 50;

    /// Bonus for partially filled server (1-80% capacity) (+30 points).
    pub const CAPACITY_BONUS: i32 = 30;

    /// Bonus for recent heartbeat within 30 seconds (+20 points).
    pub const FRESHNESS_BONUS: i32 = 20;

    /// Freshness threshold in seconds.
    pub const FRESHNESS_THRESHOLD_SECS: u64 = 30;

    /// Bonus per player (up to 80% capacity) (+1 point each).
    pub const PLAYER_BONUS_PER_PLAYER: i32 = 1;

    /// Bonus for ping < 50ms (+40 points).
    pub const PING_BONUS_EXCELLENT: i32 = 40;

    /// Bonus for ping 50-100ms (+20 points).
    pub const PING_BONUS_GOOD: i32 = 20;

    /// Ping threshold for excellent bonus (ms).
    pub const PING_THRESHOLD_EXCELLENT: u32 = 50;

    /// Ping threshold for good bonus (ms).
    pub const PING_THRESHOLD_GOOD: u32 = 100;
}

/// Breakdown of score components for debugging/logging.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScoreBreakdown {
    /// +50 if region matches request.
    pub region_bonus: i32,
    /// +30 if server is 1-80% capacity.
    pub capacity_bonus: i32,
    /// +20 if last_seen < 30 seconds ago.
    pub freshness_bonus: i32,
    /// +1 per player (up to 80% capacity).
    pub player_bonus: i32,
    /// +40 if ping < 50ms, +20 if ping < 100ms (optional).
    pub ping_bonus: i32,
}

impl ScoreBreakdown {
    /// Calculate total score from breakdown.
    pub fn total(&self) -> i32 {
        self.region_bonus
            + self.capacity_bonus
            + self.freshness_bonus
            + self.player_bonus
            + self.ping_bonus
    }
}

/// Scoring result for a server candidate.
#[derive(Debug, Clone)]
pub struct ServerScore {
    /// Reference to the scored server.
    pub server: ServerEntry,
    /// Total computed score.
    pub total_score: i32,
    /// Score breakdown for debugging/logging.
    pub breakdown: ScoreBreakdown,
}

impl ServerScore {
    /// Create a new server score.
    pub fn new(server: ServerEntry, breakdown: ScoreBreakdown) -> Self {
        let total_score = breakdown.total();
        Self {
            server,
            total_score,
            breakdown,
        }
    }
}

/// Calculate score for a single server.
///
/// # Arguments
/// * `server` - The server to score
/// * `request` - The quick join request (for region matching)
/// * `current_time` - Current Unix timestamp for freshness calculation
/// * `ping_ms` - Optional ping measurement in milliseconds
pub fn calculate_score(
    server: &ServerEntry,
    request: &QuickJoinRequest,
    current_time: u64,
    ping_ms: Option<u32>,
) -> ServerScore {
    let mut breakdown = ScoreBreakdown::default();

    // Region bonus (+50 if region matches, "any" always matches)
    if request.region == "any" || server.region.to_lowercase() == request.region.to_lowercase() {
        breakdown.region_bonus = weights::REGION_BONUS;
    }

    // Capacity bonus (+30 if 1-80% full)
    let capacity_percent = if server.max_players > 0 {
        (server.player_count as f32 / server.max_players as f32) * 100.0
    } else {
        0.0
    };
    if server.player_count > 0 && capacity_percent <= 80.0 {
        breakdown.capacity_bonus = weights::CAPACITY_BONUS;
    }

    // Freshness bonus (+20 if last_seen within 30 seconds)
    if current_time.saturating_sub(server.last_seen) <= weights::FRESHNESS_THRESHOLD_SECS {
        breakdown.freshness_bonus = weights::FRESHNESS_BONUS;
    }

    // Player bonus (+1 per player, capped at 80% capacity)
    let max_bonus_players = ((server.max_players as f32 * 0.8).floor()) as u8;
    let bonus_players = server.player_count.min(max_bonus_players);
    breakdown.player_bonus = bonus_players as i32 * weights::PLAYER_BONUS_PER_PLAYER;

    // Ping bonus (optional, +40 if <50ms, +20 if <100ms)
    if let Some(ping) = ping_ms {
        if ping < weights::PING_THRESHOLD_EXCELLENT {
            breakdown.ping_bonus = weights::PING_BONUS_EXCELLENT;
        } else if ping < weights::PING_THRESHOLD_GOOD {
            breakdown.ping_bonus = weights::PING_BONUS_GOOD;
        }
    }

    ServerScore::new(server.clone(), breakdown)
}

/// Score a list of servers and return sorted by score (descending).
///
/// # Arguments
/// * `servers` - Pre-filtered server list
/// * `request` - The quick join request (for region matching)
/// * `current_time` - Current Unix timestamp for freshness calculation
pub fn score_servers(
    servers: &[ServerEntry],
    request: &QuickJoinRequest,
    current_time: u64,
) -> Vec<ServerScore> {
    let mut scored: Vec<ServerScore> = servers
        .iter()
        .map(|s| calculate_score(s, request, current_time, None))
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.total_score.cmp(&a.total_score));

    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_server(name: &str, region: &str, players: u8, max: u8, last_seen: u64) -> ServerEntry {
        ServerEntry {
            server_id: format!("id_{}", name),
            name: name.to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: region.to_string(),
            tags: vec![],
            player_count: players,
            max_players: max,
            game_modes: vec!["tdm".to_string()],
            protocol_version: "0.1.0".to_string(),
            last_seen,
        }
    }

    #[test]
    fn test_region_bonus() {
        let server = make_server("EU Server", "eu", 5, 16, 1000);
        let request = QuickJoinRequest::new("tdm", "eu");

        let score = calculate_score(&server, &request, 1000, None);
        assert_eq!(score.breakdown.region_bonus, weights::REGION_BONUS);
    }

    #[test]
    fn test_region_bonus_any() {
        let server = make_server("US Server", "us", 5, 16, 1000);
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, None);
        assert_eq!(score.breakdown.region_bonus, weights::REGION_BONUS);
    }

    #[test]
    fn test_no_region_bonus_mismatch() {
        let server = make_server("US Server", "us", 5, 16, 1000);
        let request = QuickJoinRequest::new("tdm", "eu");

        let score = calculate_score(&server, &request, 1000, None);
        assert_eq!(score.breakdown.region_bonus, 0);
    }

    #[test]
    fn test_capacity_bonus_partially_filled() {
        let server = make_server("Test", "eu", 8, 16, 1000); // 50% full
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, None);
        assert_eq!(score.breakdown.capacity_bonus, weights::CAPACITY_BONUS);
    }

    #[test]
    fn test_no_capacity_bonus_empty() {
        let server = make_server("Test", "eu", 0, 16, 1000); // empty
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, None);
        assert_eq!(score.breakdown.capacity_bonus, 0);
    }

    #[test]
    fn test_no_capacity_bonus_over_80_percent() {
        let server = make_server("Test", "eu", 14, 16, 1000); // 87.5% full
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, None);
        assert_eq!(score.breakdown.capacity_bonus, 0);
    }

    #[test]
    fn test_freshness_bonus_recent() {
        let server = make_server("Test", "eu", 5, 16, 995);
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, None); // 5 seconds ago
        assert_eq!(score.breakdown.freshness_bonus, weights::FRESHNESS_BONUS);
    }

    #[test]
    fn test_no_freshness_bonus_stale() {
        let server = make_server("Test", "eu", 5, 16, 900);
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, None); // 100 seconds ago
        assert_eq!(score.breakdown.freshness_bonus, 0);
    }

    #[test]
    fn test_player_bonus() {
        let server = make_server("Test", "eu", 10, 20, 1000);
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, None);
        // 10 players, max bonus at 80% of 20 = 16 players, so 10 * 1 = 10
        assert_eq!(score.breakdown.player_bonus, 10);
    }

    #[test]
    fn test_player_bonus_capped() {
        let server = make_server("Test", "eu", 18, 20, 1000);
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, None);
        // 18 players but capped at 80% of 20 = 16
        assert_eq!(score.breakdown.player_bonus, 16);
    }

    #[test]
    fn test_ping_bonus_excellent() {
        let server = make_server("Test", "eu", 5, 16, 1000);
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, Some(30));
        assert_eq!(score.breakdown.ping_bonus, weights::PING_BONUS_EXCELLENT);
    }

    #[test]
    fn test_ping_bonus_good() {
        let server = make_server("Test", "eu", 5, 16, 1000);
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, Some(75));
        assert_eq!(score.breakdown.ping_bonus, weights::PING_BONUS_GOOD);
    }

    #[test]
    fn test_no_ping_bonus_high_ping() {
        let server = make_server("Test", "eu", 5, 16, 1000);
        let request = QuickJoinRequest::new("tdm", "any");

        let score = calculate_score(&server, &request, 1000, Some(150));
        assert_eq!(score.breakdown.ping_bonus, 0);
    }

    #[test]
    fn test_total_score_calculation() {
        let breakdown = ScoreBreakdown {
            region_bonus: 50,
            capacity_bonus: 30,
            freshness_bonus: 20,
            player_bonus: 10,
            ping_bonus: 40,
        };
        assert_eq!(breakdown.total(), 150);
    }

    #[test]
    fn test_score_servers_sorted() {
        let servers = vec![
            make_server("Low Score", "us", 0, 16, 900), // No bonuses
            make_server("High Score", "eu", 10, 16, 1000), // Multiple bonuses
            make_server("Medium Score", "eu", 5, 16, 950), // Some bonuses
        ];
        let request = QuickJoinRequest::new("tdm", "eu");

        let scored = score_servers(&servers, &request, 1000);

        // Verify sorted by score descending
        assert!(scored[0].total_score >= scored[1].total_score);
        assert!(scored[1].total_score >= scored[2].total_score);
        assert_eq!(scored[0].server.name, "High Score");
    }
}
