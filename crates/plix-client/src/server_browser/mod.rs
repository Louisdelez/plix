//! Server browser module for discovering and connecting to game servers.
//!
//! Provides functionality to:
//! - Fetch server list from master server
//! - Display formatted server list
//! - Connect to servers by index
//! - Cache server entries for quick access

pub mod fetch;

use plix_common::server_browser::ServerEntry;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cached server browser state.
///
/// Stores the last fetched server list for quick access by index.
pub struct BrowserState {
    /// Cached server entries from last fetch
    servers: Arc<RwLock<Vec<ServerEntry>>>,
    /// Master server URL
    master_url: String,
}

impl BrowserState {
    /// Create a new browser state with the given master URL.
    pub fn new(master_url: impl Into<String>) -> Self {
        Self {
            servers: Arc::new(RwLock::new(Vec::new())),
            master_url: master_url.into(),
        }
    }

    /// Get the master server URL.
    pub fn master_url(&self) -> &str {
        &self.master_url
    }

    /// Refresh the server list from the master server.
    ///
    /// Returns the number of servers found, or an error.
    pub async fn refresh(&self) -> Result<usize, fetch::FetchError> {
        let response = fetch::fetch_servers(&self.master_url).await?;
        let count = response.servers.len();
        let mut servers = self.servers.write().await;
        *servers = response.servers;
        Ok(count)
    }

    /// Get a copy of the current server list.
    pub async fn servers(&self) -> Vec<ServerEntry> {
        self.servers.read().await.clone()
    }

    /// Get a server by 1-based index.
    ///
    /// Returns None if index is out of range.
    pub async fn get_server(&self, index: usize) -> Option<ServerEntry> {
        let servers = self.servers.read().await;
        if index == 0 || index > servers.len() {
            return None;
        }
        servers.get(index - 1).cloned()
    }

    /// Get the number of cached servers.
    pub async fn count(&self) -> usize {
        self.servers.read().await.len()
    }
}

/// Format the server list for console display.
pub fn format_server_list(servers: &[ServerEntry]) -> String {
    if servers.is_empty() {
        return "No servers found.".to_string();
    }

    let mut output = format!("Found {} server(s):\n", servers.len());
    output.push_str("─".repeat(60).as_str());
    output.push('\n');

    for (i, server) in servers.iter().enumerate() {
        let players = format!("{}/{}", server.player_count, server.max_players);
        let tags = if server.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", server.tags.join(", "))
        };

        output.push_str(&format!(
            "{:>2}. {} ({}) - {} players{}\n",
            i + 1,
            server.name,
            server.region,
            players,
            tags
        ));
    }

    output.push_str("─".repeat(60).as_str());
    output.push_str("\nUse /connect <number> to join a server.");

    output
}

/// Truncate and sanitize a string for safe display.
///
/// Removes control characters and truncates to max length.
pub fn sanitize_display_string(s: &str, max_len: usize) -> String {
    let sanitized: String = s.chars().filter(|c| !c.is_control()).collect();

    if sanitized.len() > max_len {
        format!("{}...", &sanitized[..max_len])
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_server(name: &str, players: u8, max: u8, region: &str) -> ServerEntry {
        ServerEntry {
            server_id: format!("id_{}", name),
            name: name.to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: region.to_string(),
            tags: vec![],
            player_count: players,
            max_players: max,
            game_modes: vec!["ffa".to_string()],
            protocol_version: "0.1.0".to_string(),
            last_seen: 0,
        }
    }

    #[test]
    fn test_format_empty_list() {
        let output = format_server_list(&[]);
        assert_eq!(output, "No servers found.");
    }

    #[test]
    fn test_format_server_list() {
        let servers = vec![
            make_test_server("Test Server 1", 5, 16, "eu-west"),
            make_test_server("Test Server 2", 0, 32, "us-east"),
        ];

        let output = format_server_list(&servers);
        assert!(output.contains("Found 2 server(s)"));
        assert!(output.contains("1. Test Server 1"));
        assert!(output.contains("2. Test Server 2"));
        assert!(output.contains("5/16 players"));
        assert!(output.contains("0/32 players"));
        assert!(output.contains("/connect"));
    }

    #[test]
    fn test_format_server_with_tags() {
        let mut server = make_test_server("Tagged Server", 10, 20, "eu");
        server.tags = vec!["competitive".to_string(), "ranked".to_string()];

        let output = format_server_list(&[server]);
        assert!(output.contains("[competitive, ranked]"));
    }

    #[test]
    fn test_sanitize_display_string() {
        assert_eq!(sanitize_display_string("hello", 10), "hello");
        assert_eq!(sanitize_display_string("hello world", 5), "hello...");
        assert_eq!(sanitize_display_string("a\nb\tc", 10), "abc");
    }

    #[tokio::test]
    async fn test_browser_state_get_server() {
        let state = BrowserState::new("http://localhost:8080");

        // Manually set servers for testing
        {
            let mut servers = state.servers.write().await;
            *servers = vec![
                make_test_server("Server A", 1, 10, "eu"),
                make_test_server("Server B", 2, 20, "us"),
            ];
        }

        // Index 0 is invalid (1-based indexing)
        assert!(state.get_server(0).await.is_none());

        // Index 1 should return first server
        let server = state.get_server(1).await.unwrap();
        assert_eq!(server.name, "Server A");

        // Index 2 should return second server
        let server = state.get_server(2).await.unwrap();
        assert_eq!(server.name, "Server B");

        // Index 3 is out of range
        assert!(state.get_server(3).await.is_none());
    }
}
