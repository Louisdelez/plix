//! Server registry state management.
//!
//! Provides in-memory storage for server entries with TTL-based expiration
//! and rate limiting per IP address.

use plix_common::server_browser::ServerEntry;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::rate_limit::RateLimiter;
use crate::types::HeartbeatRequest;

/// In-memory server registry.
///
/// Stores server entries keyed by server_id (hash of host:port).
/// Entries automatically expire after TTL seconds without a heartbeat.
pub struct ServerRegistry {
    /// Map of server_id -> ServerEntry
    servers: RwLock<HashMap<String, ServerEntry>>,
    /// Rate limiter for heartbeat requests
    rate_limiter: RateLimiter,
    /// TTL for server entries in seconds
    ttl_secs: u64,
}

impl ServerRegistry {
    /// Create a new server registry with the specified TTL and rate limit.
    pub fn new(ttl_secs: u64, rate_limit_per_minute: u32) -> Arc<Self> {
        Arc::new(Self {
            servers: RwLock::new(HashMap::new()),
            rate_limiter: RateLimiter::new(rate_limit_per_minute),
            ttl_secs,
        })
    }

    /// Generate a stable server_id from host and port.
    ///
    /// Uses a hash of "host:port" to create a deterministic, stable identifier.
    /// The same host:port will always produce the same server_id.
    pub fn generate_server_id(host: &str, port: u16) -> String {
        let mut hasher = DefaultHasher::new();
        host.hash(&mut hasher);
        port.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Check if a request from the given IP is within rate limits.
    ///
    /// Returns true if the request is allowed, false if rate limited.
    pub async fn check_rate_limit(&self, ip: IpAddr) -> bool {
        self.rate_limiter.check(ip).await
    }

    /// Register or update a server entry from a heartbeat request.
    ///
    /// Returns the assigned server_id.
    pub async fn register(&self, request: HeartbeatRequest) -> String {
        let server_id = Self::generate_server_id(&request.host, request.port);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = ServerEntry {
            server_id: server_id.clone(),
            name: request.name,
            host: request.host,
            port: request.port,
            region: request.region,
            tags: request.tags,
            player_count: request.player_count,
            max_players: request.max_players,
            game_modes: request.game_modes,
            protocol_version: request.protocol_version,
            last_seen: now,
        };

        let mut servers = self.servers.write().await;
        servers.insert(server_id.clone(), entry);

        server_id
    }

    /// Get all active (non-expired) server entries.
    ///
    /// Entries with last_seen older than TTL are filtered out.
    /// This performs lazy cleanup - expired entries are not removed from storage,
    /// just filtered from the response.
    pub async fn get_active_servers(&self) -> Vec<ServerEntry> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let servers = self.servers.read().await;
        servers
            .values()
            .filter(|entry| now.saturating_sub(entry.last_seen) < self.ttl_secs)
            .cloned()
            .collect()
    }

    /// Get the count of active (non-expired) servers.
    pub async fn active_count(&self) -> usize {
        self.get_active_servers().await.len()
    }

    /// Get the TTL in seconds.
    pub fn ttl_secs(&self) -> u64 {
        self.ttl_secs
    }

    /// Clean up expired entries from storage.
    ///
    /// This is optional - entries are already filtered on read.
    /// Call this periodically to free memory if needed.
    pub async fn cleanup_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut servers = self.servers.write().await;
        servers.retain(|_, entry| now.saturating_sub(entry.last_seen) < self.ttl_secs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_server_id_deterministic() {
        let id1 = ServerRegistry::generate_server_id("192.168.1.100", 7777);
        let id2 = ServerRegistry::generate_server_id("192.168.1.100", 7777);
        assert_eq!(id1, id2, "Same host:port should produce same ID");
    }

    #[test]
    fn test_generate_server_id_different_host() {
        let id1 = ServerRegistry::generate_server_id("192.168.1.100", 7777);
        let id2 = ServerRegistry::generate_server_id("192.168.1.101", 7777);
        assert_ne!(id1, id2, "Different hosts should produce different IDs");
    }

    #[test]
    fn test_generate_server_id_different_port() {
        let id1 = ServerRegistry::generate_server_id("192.168.1.100", 7777);
        let id2 = ServerRegistry::generate_server_id("192.168.1.100", 7778);
        assert_ne!(id1, id2, "Different ports should produce different IDs");
    }

    #[test]
    fn test_generate_server_id_format() {
        let id = ServerRegistry::generate_server_id("localhost", 8080);
        assert_eq!(id.len(), 16, "Server ID should be 16 hex characters");
        assert!(
            id.chars().all(|c| c.is_ascii_hexdigit()),
            "Server ID should be hex"
        );
    }

    #[tokio::test]
    async fn test_register_and_get_servers() {
        let registry = ServerRegistry::new(60, 100);

        let request = HeartbeatRequest {
            name: "Test Server".to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: "local".to_string(),
            tags: vec!["test".to_string()],
            player_count: 5,
            max_players: 10,
            game_modes: vec!["ffa".to_string()],
            protocol_version: "0.1.0".to_string(),
        };

        let server_id = registry.register(request).await;

        let servers = registry.get_active_servers().await;
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].server_id, server_id);
        assert_eq!(servers[0].name, "Test Server");
        assert_eq!(servers[0].player_count, 5);
    }

    #[tokio::test]
    async fn test_update_existing_server() {
        let registry = ServerRegistry::new(60, 100);

        let request1 = HeartbeatRequest {
            name: "Test Server".to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: "local".to_string(),
            tags: vec![],
            player_count: 5,
            max_players: 10,
            game_modes: vec![],
            protocol_version: "0.1.0".to_string(),
        };

        let id1 = registry.register(request1).await;

        // Update with new player count
        let request2 = HeartbeatRequest {
            name: "Test Server".to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: "local".to_string(),
            tags: vec![],
            player_count: 10,
            max_players: 10,
            game_modes: vec![],
            protocol_version: "0.1.0".to_string(),
        };

        let id2 = registry.register(request2).await;

        assert_eq!(id1, id2, "Same host:port should produce same ID");

        let servers = registry.get_active_servers().await;
        assert_eq!(servers.len(), 1, "Should still be one server");
        assert_eq!(
            servers[0].player_count, 10,
            "Player count should be updated"
        );
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        // Create registry with 1 second TTL
        let registry = ServerRegistry::new(1, 100);

        let request = HeartbeatRequest {
            name: "Test Server".to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: "local".to_string(),
            tags: vec![],
            player_count: 0,
            max_players: 10,
            game_modes: vec![],
            protocol_version: "0.1.0".to_string(),
        };

        registry.register(request).await;

        // Should be visible immediately
        let servers = registry.get_active_servers().await;
        assert_eq!(servers.len(), 1);

        // Wait for TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be filtered out now
        let servers = registry.get_active_servers().await;
        assert_eq!(servers.len(), 0, "Expired server should not be returned");
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let registry = ServerRegistry::new(1, 100);

        let request = HeartbeatRequest {
            name: "Test Server".to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: "local".to_string(),
            tags: vec![],
            player_count: 0,
            max_players: 10,
            game_modes: vec![],
            protocol_version: "0.1.0".to_string(),
        };

        registry.register(request).await;

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Cleanup
        registry.cleanup_expired().await;

        // Verify storage is actually cleaned
        let servers = registry.servers.read().await;
        assert!(
            servers.is_empty(),
            "Expired entries should be removed from storage"
        );
    }
}
