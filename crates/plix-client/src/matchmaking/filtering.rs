//! Server filtering for matchmaking.
//!
//! Filters servers based on mandatory criteria (version, capacity) and
//! optional criteria (mode, region) before scoring.

use plix_common::server_browser::ServerEntry;

use super::QuickJoinRequest;

/// Filter servers based on request criteria.
///
/// # Mandatory Filters (always applied)
/// - Incompatible protocol version → excluded
/// - Full servers (player_count >= max_players) → excluded
///
/// # Optional Filters (based on request)
/// - Mode filter: server must have requested mode in game_modes (unless mode="any")
/// - Region filter: server must match requested region (unless region="any")
pub fn filter_servers(
    servers: &[ServerEntry],
    request: &QuickJoinRequest,
    protocol_version: &str,
) -> Vec<ServerEntry> {
    servers
        .iter()
        .filter(|s| filter_by_version(s, protocol_version))
        .filter(|s| filter_by_capacity(s))
        .filter(|s| filter_by_mode(s, &request.mode))
        .filter(|s| filter_by_region(s, &request.region))
        .cloned()
        .collect()
}

/// Filter by protocol version compatibility.
///
/// Servers with incompatible versions are excluded.
pub fn filter_by_version(server: &ServerEntry, client_version: &str) -> bool {
    server.protocol_version == client_version
}

/// Filter by server capacity.
///
/// Full servers (player_count >= max_players) are excluded.
pub fn filter_by_capacity(server: &ServerEntry) -> bool {
    server.player_count < server.max_players
}

/// Filter by game mode.
///
/// If mode is "any", all servers pass.
/// Otherwise, server must have the requested mode in game_modes.
pub fn filter_by_mode(server: &ServerEntry, mode: &str) -> bool {
    if mode.to_lowercase() == "any" {
        return true;
    }

    server
        .game_modes
        .iter()
        .any(|m| m.to_lowercase() == mode.to_lowercase())
}

/// Filter by region.
///
/// If region is "any", all servers pass.
/// Otherwise, server must match the requested region.
pub fn filter_by_region(server: &ServerEntry, region: &str) -> bool {
    if region.to_lowercase() == "any" {
        return true;
    }

    server.region.to_lowercase() == region.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_server(
        name: &str,
        region: &str,
        players: u8,
        max: u8,
        modes: Vec<&str>,
        version: &str,
    ) -> ServerEntry {
        ServerEntry {
            server_id: format!("id_{}", name),
            name: name.to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: region.to_string(),
            tags: vec![],
            player_count: players,
            max_players: max,
            game_modes: modes.into_iter().map(String::from).collect(),
            protocol_version: version.to_string(),
            last_seen: 0,
        }
    }

    // Version filter tests

    #[test]
    fn test_filter_by_version_match() {
        let server = make_server("Test", "eu", 5, 16, vec!["tdm"], "0.1.0");
        assert!(filter_by_version(&server, "0.1.0"));
    }

    #[test]
    fn test_filter_by_version_mismatch() {
        let server = make_server("Test", "eu", 5, 16, vec!["tdm"], "0.1.0");
        assert!(!filter_by_version(&server, "0.2.0"));
    }

    // Capacity filter tests

    #[test]
    fn test_filter_by_capacity_available() {
        let server = make_server("Test", "eu", 10, 16, vec!["tdm"], "0.1.0");
        assert!(filter_by_capacity(&server));
    }

    #[test]
    fn test_filter_by_capacity_full() {
        let server = make_server("Test", "eu", 16, 16, vec!["tdm"], "0.1.0");
        assert!(!filter_by_capacity(&server));
    }

    #[test]
    fn test_filter_by_capacity_overfull() {
        // Edge case: player_count > max_players
        let server = make_server("Test", "eu", 18, 16, vec!["tdm"], "0.1.0");
        assert!(!filter_by_capacity(&server));
    }

    // Mode filter tests

    #[test]
    fn test_filter_by_mode_exact_match() {
        let server = make_server("Test", "eu", 5, 16, vec!["tdm", "ffa"], "0.1.0");
        assert!(filter_by_mode(&server, "tdm"));
    }

    #[test]
    fn test_filter_by_mode_case_insensitive() {
        let server = make_server("Test", "eu", 5, 16, vec!["TDM"], "0.1.0");
        assert!(filter_by_mode(&server, "tdm"));
    }

    #[test]
    fn test_filter_by_mode_any() {
        let server = make_server("Test", "eu", 5, 16, vec!["ctf"], "0.1.0");
        assert!(filter_by_mode(&server, "any"));
    }

    #[test]
    fn test_filter_by_mode_no_match() {
        let server = make_server("Test", "eu", 5, 16, vec!["ctf"], "0.1.0");
        assert!(!filter_by_mode(&server, "tdm"));
    }

    // Region filter tests

    #[test]
    fn test_filter_by_region_exact_match() {
        let server = make_server("Test", "eu", 5, 16, vec!["tdm"], "0.1.0");
        assert!(filter_by_region(&server, "eu"));
    }

    #[test]
    fn test_filter_by_region_case_insensitive() {
        let server = make_server("Test", "EU", 5, 16, vec!["tdm"], "0.1.0");
        assert!(filter_by_region(&server, "eu"));
    }

    #[test]
    fn test_filter_by_region_any() {
        let server = make_server("Test", "asia", 5, 16, vec!["tdm"], "0.1.0");
        assert!(filter_by_region(&server, "any"));
    }

    #[test]
    fn test_filter_by_region_no_match() {
        let server = make_server("Test", "us", 5, 16, vec!["tdm"], "0.1.0");
        assert!(!filter_by_region(&server, "eu"));
    }

    // Combined filter tests

    #[test]
    fn test_filter_servers_all_pass() {
        let servers = vec![
            make_server("Server1", "eu", 5, 16, vec!["tdm"], "0.1.0"),
            make_server("Server2", "eu", 8, 16, vec!["tdm", "ffa"], "0.1.0"),
        ];
        let request = QuickJoinRequest::new("tdm", "eu");

        let filtered = filter_servers(&servers, &request, "0.1.0");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_servers_version_mismatch() {
        let servers = vec![
            make_server("Good", "eu", 5, 16, vec!["tdm"], "0.1.0"),
            make_server("Bad Version", "eu", 5, 16, vec!["tdm"], "0.2.0"),
        ];
        let request = QuickJoinRequest::new("tdm", "eu");

        let filtered = filter_servers(&servers, &request, "0.1.0");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Good");
    }

    #[test]
    fn test_filter_servers_full_excluded() {
        let servers = vec![
            make_server("Available", "eu", 10, 16, vec!["tdm"], "0.1.0"),
            make_server("Full", "eu", 16, 16, vec!["tdm"], "0.1.0"),
        ];
        let request = QuickJoinRequest::new("tdm", "eu");

        let filtered = filter_servers(&servers, &request, "0.1.0");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Available");
    }

    #[test]
    fn test_filter_servers_mode_mismatch() {
        let servers = vec![
            make_server("TDM Server", "eu", 5, 16, vec!["tdm"], "0.1.0"),
            make_server("CTF Server", "eu", 5, 16, vec!["ctf"], "0.1.0"),
        ];
        let request = QuickJoinRequest::new("tdm", "eu");

        let filtered = filter_servers(&servers, &request, "0.1.0");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "TDM Server");
    }

    #[test]
    fn test_filter_servers_region_mismatch() {
        let servers = vec![
            make_server("EU Server", "eu", 5, 16, vec!["tdm"], "0.1.0"),
            make_server("US Server", "us", 5, 16, vec!["tdm"], "0.1.0"),
        ];
        let request = QuickJoinRequest::new("tdm", "eu");

        let filtered = filter_servers(&servers, &request, "0.1.0");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "EU Server");
    }

    #[test]
    fn test_filter_servers_any_mode() {
        let servers = vec![
            make_server("TDM Server", "eu", 5, 16, vec!["tdm"], "0.1.0"),
            make_server("CTF Server", "eu", 5, 16, vec!["ctf"], "0.1.0"),
        ];
        let request = QuickJoinRequest::new("any", "eu");

        let filtered = filter_servers(&servers, &request, "0.1.0");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_servers_any_region() {
        let servers = vec![
            make_server("EU Server", "eu", 5, 16, vec!["tdm"], "0.1.0"),
            make_server("US Server", "us", 5, 16, vec!["tdm"], "0.1.0"),
        ];
        let request = QuickJoinRequest::new("tdm", "any");

        let filtered = filter_servers(&servers, &request, "0.1.0");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_servers_empty_result() {
        let servers = vec![make_server(
            "Wrong Everything",
            "asia",
            16,
            16,
            vec!["ctf"],
            "0.2.0",
        )];
        let request = QuickJoinRequest::new("tdm", "eu");

        let filtered = filter_servers(&servers, &request, "0.1.0");
        assert!(filtered.is_empty());
    }
}
