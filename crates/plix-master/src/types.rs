//! Request and response types for the master server API.

use serde::{Deserialize, Serialize};

/// Heartbeat request sent by game servers to register/update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    /// Server display name
    pub name: String,
    /// Server host/IP address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Geographic region (e.g., "eu-west", "us-east")
    pub region: String,
    /// Server tags for filtering (e.g., "competitive", "casual")
    pub tags: Vec<String>,
    /// Current number of connected players
    pub player_count: u8,
    /// Maximum player capacity
    pub max_players: u8,
    /// Supported game modes (e.g., "ffa", "ctf", "tdm")
    pub game_modes: Vec<String>,
    /// Protocol version for compatibility checking
    pub protocol_version: String,
}

/// Heartbeat response returned to game servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    /// Assigned server ID (stable hash of host:port)
    pub server_id: String,
    /// TTL in seconds until next heartbeat is required
    pub ttl_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_request_serialization() {
        let request = HeartbeatRequest {
            name: "Test Server".to_string(),
            host: "192.168.1.100".to_string(),
            port: 7777,
            region: "eu-west".to_string(),
            tags: vec!["competitive".to_string()],
            player_count: 10,
            max_players: 32,
            game_modes: vec!["ctf".to_string(), "ffa".to_string()],
            protocol_version: "0.1.0".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: HeartbeatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, request.name);
        assert_eq!(deserialized.host, request.host);
        assert_eq!(deserialized.port, request.port);
        assert_eq!(deserialized.player_count, request.player_count);
    }

    #[test]
    fn test_heartbeat_response_serialization() {
        let response = HeartbeatResponse {
            server_id: "abc123def456".to_string(),
            ttl_secs: 60,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: HeartbeatResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.server_id, response.server_id);
        assert_eq!(deserialized.ttl_secs, response.ttl_secs);
    }
}
