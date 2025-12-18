//! Server selection with tie-breaking.

use rand::Rng;
use std::collections::HashSet;
use tracing::info;

use super::ServerScore;

/// Select the best server from scored candidates.
///
/// If multiple servers have the same highest score, selects randomly among them.
///
/// # Arguments
/// * `scored` - Scored servers (should be sorted by score descending)
///
/// # Returns
/// * Selected ServerScore, or None if empty
pub fn select_best_server(scored: &[ServerScore]) -> Option<ServerScore> {
    select_best_server_excluding(scored, &HashSet::new())
}

/// Select the best server, excluding specified server IDs.
///
/// Used during retry to avoid re-selecting failed servers.
///
/// # Arguments
/// * `scored` - Scored servers (should be sorted by score descending)
/// * `excluded` - Set of server IDs to exclude
///
/// # Returns
/// * Selected ServerScore, or None if no valid servers
pub fn select_best_server_excluding(
    scored: &[ServerScore],
    excluded: &HashSet<String>,
) -> Option<ServerScore> {
    // Filter out excluded servers
    let candidates: Vec<&ServerScore> = scored
        .iter()
        .filter(|s| !excluded.contains(&s.server.server_id))
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Find highest score among remaining candidates
    let max_score = candidates[0].total_score;

    // Collect all servers with the highest score
    let tied: Vec<&ServerScore> = candidates
        .iter()
        .filter(|s| s.total_score == max_score)
        .copied()
        .collect();

    if tied.len() == 1 {
        log_selected_server(tied[0]);
        return Some(tied[0].clone());
    }

    // Random selection among tied servers
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..tied.len());
    let selected = tied[index].clone();

    // T036: Log selected server
    log_selected_server(&selected);

    Some(selected)
}

/// Log the selected server for debugging (T036).
fn log_selected_server(server: &ServerScore) {
    info!(
        server_name = %server.server.name,
        server_id = %server.server.server_id,
        host = %server.server.host,
        port = server.server.port,
        score = server.total_score,
        "Server selected for quick join"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matchmaking::scoring::ScoreBreakdown;
    use plix_common::server_browser::ServerEntry;

    fn make_server_score(name: &str, score: i32) -> ServerScore {
        let server = ServerEntry {
            server_id: format!("id_{}", name),
            name: name.to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: "eu".to_string(),
            tags: vec![],
            player_count: 5,
            max_players: 16,
            game_modes: vec!["tdm".to_string()],
            protocol_version: "0.1.0".to_string(),
            last_seen: 0,
        };
        ServerScore {
            server,
            total_score: score,
            breakdown: ScoreBreakdown::default(),
        }
    }

    #[test]
    fn test_select_single_server() {
        let scored = vec![make_server_score("Only One", 100)];

        let selected = select_best_server(&scored);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().server.name, "Only One");
    }

    #[test]
    fn test_select_highest_score() {
        let scored = vec![
            make_server_score("High", 150),
            make_server_score("Medium", 100),
            make_server_score("Low", 50),
        ];

        let selected = select_best_server(&scored);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().server.name, "High");
    }

    #[test]
    fn test_select_empty_list() {
        let scored: Vec<ServerScore> = vec![];

        let selected = select_best_server(&scored);
        assert!(selected.is_none());
    }

    #[test]
    fn test_select_with_exclusion() {
        let scored = vec![
            make_server_score("First", 150),
            make_server_score("Second", 100),
        ];

        let mut excluded = HashSet::new();
        excluded.insert("id_First".to_string());

        let selected = select_best_server_excluding(&scored, &excluded);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().server.name, "Second");
    }

    #[test]
    fn test_select_all_excluded() {
        let scored = vec![
            make_server_score("First", 150),
            make_server_score("Second", 100),
        ];

        let mut excluded = HashSet::new();
        excluded.insert("id_First".to_string());
        excluded.insert("id_Second".to_string());

        let selected = select_best_server_excluding(&scored, &excluded);
        assert!(selected.is_none());
    }

    #[test]
    fn test_tie_breaking_selects_valid_server() {
        // Create multiple servers with same score
        let scored = vec![
            make_server_score("Tied1", 100),
            make_server_score("Tied2", 100),
            make_server_score("Tied3", 100),
        ];

        // Run multiple times to verify we get valid selections
        for _ in 0..10 {
            let selected = select_best_server(&scored);
            assert!(selected.is_some());
            let name = &selected.unwrap().server.name;
            assert!(name == "Tied1" || name == "Tied2" || name == "Tied3");
        }
    }

    #[test]
    fn test_tie_breaking_randomness() {
        // Create multiple servers with same score
        let scored = vec![
            make_server_score("A", 100),
            make_server_score("B", 100),
            make_server_score("C", 100),
        ];

        // Run many times and collect selections
        let mut selections = HashSet::new();
        for _ in 0..100 {
            let selected = select_best_server(&scored);
            selections.insert(selected.unwrap().server.name.clone());
        }

        // With 100 runs and 3 options, we should see at least 2 different selections
        // (extremely unlikely to get the same one 100 times)
        assert!(
            selections.len() >= 2,
            "Expected random distribution, got {:?}",
            selections
        );
    }
}
