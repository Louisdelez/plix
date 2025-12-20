//! Scoreboard Client (Feature 032)
//!
//! Manages client-side scoreboard state including visibility,
//! player data caching, and update throttling.

use plix_common::protocol::WorldSnapshot;
use serde::Serialize;
use std::time::Instant;

/// Maximum number of players in scoreboard
pub const MAX_SCOREBOARD_ROWS: usize = 64;

/// A single player row in the scoreboard
#[derive(Debug, Clone, Serialize)]
pub struct ScoreboardRow {
    /// Player display name
    pub name: String,
    /// Player ping in milliseconds
    pub ping_ms: u32,
    /// Player score (mode-dependent, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<u16>,
    /// Kill count (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kills: Option<u16>,
    /// Death count (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaths: Option<u16>,
    /// Team identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
}

impl ScoreboardRow {
    /// Create a new scoreboard row
    pub fn new(name: impl Into<String>, ping_ms: u32) -> Self {
        Self {
            name: name.into(),
            ping_ms,
            score: None,
            kills: None,
            deaths: None,
            team: None,
        }
    }

    /// Create a row with full stats
    pub fn with_stats(
        name: impl Into<String>,
        ping_ms: u32,
        kills: u16,
        deaths: u16,
        team: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            ping_ms,
            score: Some(kills), // Score = kills for now
            kills: Some(kills),
            deaths: Some(deaths),
            team,
        }
    }
}

/// Complete scoreboard state snapshot
#[derive(Debug, Clone, Serialize)]
pub struct ScoreboardState {
    /// Server/match name
    pub server_name: String,
    /// Player rows
    pub rows: Vec<ScoreboardRow>,
}

impl Default for ScoreboardState {
    fn default() -> Self {
        Self {
            server_name: String::new(),
            rows: Vec::new(),
        }
    }
}

impl ScoreboardState {
    /// Create a new scoreboard state
    pub fn new(server_name: impl Into<String>) -> Self {
        Self {
            server_name: server_name.into(),
            rows: Vec::with_capacity(MAX_SCOREBOARD_ROWS),
        }
    }

    /// Add a player row
    pub fn add_row(&mut self, row: ScoreboardRow) {
        if self.rows.len() < MAX_SCOREBOARD_ROWS {
            self.rows.push(row);
        }
    }

    /// Sort rows by score (descending)
    pub fn sort_by_score(&mut self) {
        self.rows
            .sort_by(|a, b| b.score.unwrap_or(0).cmp(&a.score.unwrap_or(0)));
    }

    /// Sort rows by team, then by score within team
    pub fn sort_by_team_and_score(&mut self) {
        self.rows.sort_by(|a, b| {
            // First sort by team (None last)
            match (&a.team, &b.team) {
                (Some(t1), Some(t2)) => {
                    let team_cmp = t1.cmp(t2);
                    if team_cmp != std::cmp::Ordering::Equal {
                        return team_cmp;
                    }
                }
                (Some(_), None) => return std::cmp::Ordering::Less,
                (None, Some(_)) => return std::cmp::Ordering::Greater,
                (None, None) => {}
            }
            // Then sort by score (descending)
            b.score.unwrap_or(0).cmp(&a.score.unwrap_or(0))
        });
    }
}

/// Scoreboard client state manager
#[derive(Debug)]
pub struct ScoreboardClient {
    /// Whether scoreboard is currently visible
    visible: bool,
    /// Cached scoreboard state
    cached_state: Option<ScoreboardState>,
    /// Time of last update
    last_update: Option<Instant>,
}

impl Default for ScoreboardClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoreboardClient {
    /// Create a new scoreboard client
    pub fn new() -> Self {
        Self {
            visible: false,
            cached_state: None,
            last_update: None,
        }
    }

    /// Check if scoreboard is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show the scoreboard
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the scoreboard
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Get the cached state (if any)
    pub fn cached_state(&self) -> Option<&ScoreboardState> {
        self.cached_state.as_ref()
    }

    /// Update scoreboard state from a world snapshot
    ///
    /// Only updates if the scoreboard is visible.
    /// Returns the new state if updated, None otherwise.
    pub fn update_from_snapshot(&mut self, snapshot: &WorldSnapshot) -> Option<&ScoreboardState> {
        if !self.visible {
            return None;
        }

        let mut state = ScoreboardState::new(&snapshot.match_state.arena_name);

        // Build rows from player snapshots and match scores
        for player in &snapshot.players {
            // Find player score in match_state
            let player_score = snapshot
                .match_state
                .player_scores
                .iter()
                .find(|ps| ps.player_id == player.id);

            let (kills, deaths) = player_score
                .map(|ps| (ps.kills, ps.deaths))
                .unwrap_or((0, 0));

            // TODO: Get actual ping from server
            let ping_ms = 0; // Placeholder

            let row = ScoreboardRow::with_stats(
                &player.display_name,
                ping_ms,
                kills,
                deaths,
                None, // TODO: Team support
            );

            state.add_row(row);
        }

        // Sort by score
        state.sort_by_score();

        self.cached_state = Some(state);
        self.last_update = Some(Instant::now());

        self.cached_state.as_ref()
    }

    /// Check if update is needed (for throttling)
    ///
    /// Returns true if scoreboard is visible and hasn't been updated recently.
    pub fn needs_update(&self, max_age_ms: u64) -> bool {
        if !self.visible {
            return false;
        }

        match self.last_update {
            None => true,
            Some(last) => last.elapsed().as_millis() as u64 >= max_age_ms,
        }
    }

    /// Clear cached state
    pub fn clear_cache(&mut self) {
        self.cached_state = None;
        self.last_update = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::protocol::{MatchState, PlayerScore, PlayerSnapshot};
    use plix_common::types::PlayerId;
    use plix_common::{Rotation, Vec3};
    use std::thread::sleep;
    use std::time::Duration;

    fn make_player_snapshot(id: u16, name: &str) -> PlayerSnapshot {
        PlayerSnapshot {
            id: PlayerId(id),
            position: Vec3::ZERO,
            rotation: Rotation::default(),
            health: 100,
            is_dead: false,
            animation: Default::default(),
            spectate_target: None,
            display_name: name.to_string(),
        }
    }

    fn make_player_score(id: u16, name: &str, kills: u16, deaths: u16) -> PlayerScore {
        PlayerScore {
            player_id: PlayerId(id),
            name: name.to_string(),
            kills,
            deaths,
        }
    }

    #[test]
    fn test_scoreboard_row_new() {
        let row = ScoreboardRow::new("Alice", 45);
        assert_eq!(row.name, "Alice");
        assert_eq!(row.ping_ms, 45);
        assert!(row.score.is_none());
    }

    #[test]
    fn test_scoreboard_row_with_stats() {
        let row = ScoreboardRow::with_stats("Bob", 30, 10, 5, Some("red".to_string()));
        assert_eq!(row.name, "Bob");
        assert_eq!(row.kills, Some(10));
        assert_eq!(row.deaths, Some(5));
        assert_eq!(row.team, Some("red".to_string()));
    }

    #[test]
    fn test_scoreboard_state_sort_by_score() {
        let mut state = ScoreboardState::new("Test Server");
        state.add_row(ScoreboardRow::with_stats("Alice", 0, 5, 0, None));
        state.add_row(ScoreboardRow::with_stats("Bob", 0, 10, 0, None));
        state.add_row(ScoreboardRow::with_stats("Charlie", 0, 3, 0, None));

        state.sort_by_score();

        assert_eq!(state.rows[0].name, "Bob"); // Highest score first
        assert_eq!(state.rows[1].name, "Alice");
        assert_eq!(state.rows[2].name, "Charlie");
    }

    #[test]
    fn test_scoreboard_state_sort_by_team_and_score() {
        let mut state = ScoreboardState::new("Test Server");
        state.add_row(ScoreboardRow::with_stats(
            "Alice",
            0,
            5,
            0,
            Some("blue".to_string()),
        ));
        state.add_row(ScoreboardRow::with_stats(
            "Bob",
            0,
            10,
            0,
            Some("red".to_string()),
        ));
        state.add_row(ScoreboardRow::with_stats(
            "Charlie",
            0,
            8,
            0,
            Some("blue".to_string()),
        ));
        state.add_row(ScoreboardRow::with_stats("Dave", 0, 3, 0, None));

        state.sort_by_team_and_score();

        // Blue team first (alphabetically), sorted by score
        assert_eq!(state.rows[0].name, "Charlie"); // blue, 8
        assert_eq!(state.rows[1].name, "Alice"); // blue, 5
        assert_eq!(state.rows[2].name, "Bob"); // red, 10
        assert_eq!(state.rows[3].name, "Dave"); // no team, 3
    }

    #[test]
    fn test_scoreboard_client_visibility() {
        let mut client = ScoreboardClient::new();
        assert!(!client.is_visible());

        client.show();
        assert!(client.is_visible());

        client.hide();
        assert!(!client.is_visible());
    }

    #[test]
    fn test_scoreboard_client_update_requires_visible() {
        let mut client = ScoreboardClient::new();

        let mut snapshot = WorldSnapshot {
            tick: plix_common::Tick(1),
            last_input_seq: plix_common::types::InputSeq(0),
            players: vec![make_player_snapshot(1, "Alice")],
            match_state: MatchState {
                player_scores: vec![make_player_score(1, "Alice", 5, 2)],
                ..Default::default()
            },
            rtt_nonce_echo: 0,
            bots: vec![],
        };

        // Should not update when hidden
        assert!(client.update_from_snapshot(&snapshot).is_none());

        // Should update when visible
        client.show();
        let state = client.update_from_snapshot(&snapshot);
        assert!(state.is_some());
        assert_eq!(state.unwrap().rows.len(), 1);
    }

    #[test]
    fn test_scoreboard_client_needs_update() {
        let mut client = ScoreboardClient::new();

        // Hidden = no update needed
        assert!(!client.needs_update(100));

        client.show();
        // Visible but no cache = needs update
        assert!(client.needs_update(100));

        // After update, shouldn't need immediate update
        let snapshot = WorldSnapshot {
            tick: plix_common::Tick(1),
            last_input_seq: plix_common::types::InputSeq(0),
            players: vec![],
            match_state: MatchState::default(),
            rtt_nonce_echo: 0,
            bots: vec![],
        };
        client.update_from_snapshot(&snapshot);
        assert!(!client.needs_update(100));

        // After time passes, needs update again
        sleep(Duration::from_millis(110));
        assert!(client.needs_update(100));
    }

    #[test]
    fn test_scoreboard_state_serialize() {
        let mut state = ScoreboardState::new("Arena Battle");
        state.add_row(ScoreboardRow::with_stats(
            "Alice",
            25,
            10,
            3,
            Some("red".to_string()),
        ));

        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"server_name\":\"Arena Battle\""));
        assert!(json.contains("\"name\":\"Alice\""));
        assert!(json.contains("\"ping_ms\":25"));
        assert!(json.contains("\"kills\":10"));
        assert!(json.contains("\"team\":\"red\""));
    }
}
