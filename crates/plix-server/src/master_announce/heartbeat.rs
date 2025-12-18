//! Heartbeat task for master server announcements.
//!
//! Sends periodic heartbeat requests to the master server to keep this
//! game server's entry active in the server directory.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::interval;
use tracing::{debug, info, warn};

use super::config::MasterConfig;

/// Heartbeat request payload sent to the master server.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HeartbeatRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub region: String,
    pub tags: Vec<String>,
    pub player_count: u8,
    pub max_players: u8,
    pub game_modes: Vec<String>,
    pub protocol_version: String,
}

/// Heartbeat response from the master server.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HeartbeatResponse {
    pub server_id: String,
    pub ttl_secs: u64,
}

/// Shared state for player count updates.
pub struct HeartbeatState {
    player_count: AtomicU8,
}

impl HeartbeatState {
    /// Create new heartbeat state.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            player_count: AtomicU8::new(0),
        })
    }

    /// Update the current player count.
    pub fn set_player_count(&self, count: u8) {
        self.player_count.store(count, Ordering::Relaxed);
    }

    /// Get the current player count.
    pub fn get_player_count(&self) -> u8 {
        self.player_count.load(Ordering::Relaxed)
    }
}

impl Default for HeartbeatState {
    fn default() -> Self {
        Self {
            player_count: AtomicU8::new(0),
        }
    }
}

/// Async task that sends periodic heartbeats to the master server.
pub struct HeartbeatTask {
    config: MasterConfig,
    host: String,
    port: u16,
    max_players: u8,
    state: Arc<HeartbeatState>,
    client: reqwest::Client,
    shutdown_rx: watch::Receiver<bool>,
}

impl HeartbeatTask {
    /// Create a new heartbeat task.
    pub fn new(
        config: MasterConfig,
        host: String,
        port: u16,
        max_players: u8,
        state: Arc<HeartbeatState>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            host,
            port,
            max_players,
            state,
            client,
            shutdown_rx,
        }
    }

    /// Build the heartbeat request payload.
    pub fn build_request(&self) -> HeartbeatRequest {
        HeartbeatRequest {
            name: self.config.name.clone(),
            host: self.host.clone(),
            port: self.port,
            region: self.config.region.clone(),
            tags: self.config.tags.clone(),
            player_count: self.state.get_player_count(),
            max_players: self.max_players,
            game_modes: self.config.game_modes.clone(),
            protocol_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Send a single heartbeat to the master server.
    async fn send_heartbeat(&self) -> Result<HeartbeatResponse, HeartbeatError> {
        let url = format!("{}/heartbeat", self.config.url);
        let request = self.build_request();

        debug!(
            url = %url,
            name = %request.name,
            players = %format!("{}/{}", request.player_count, request.max_players),
            "Sending heartbeat"
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(HeartbeatError::Network)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(HeartbeatError::ServerError(status.as_u16(), body));
        }

        let heartbeat_response: HeartbeatResponse =
            response.json().await.map_err(HeartbeatError::ParseError)?;

        Ok(heartbeat_response)
    }

    /// Run the heartbeat task loop.
    pub async fn run(mut self) {
        if !self.config.enabled {
            info!("Master server announcement disabled");
            return;
        }

        if self.config.url.is_empty() {
            warn!("Master server URL not configured, skipping heartbeat");
            return;
        }

        info!(
            master_url = %self.config.url,
            server_name = %self.config.name,
            interval_secs = self.config.heartbeat_interval_secs,
            "Starting heartbeat task"
        );

        let mut ticker = interval(Duration::from_secs(self.config.heartbeat_interval_secs));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut server_id: Option<String> = None;

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    match self.send_heartbeat().await {
                        Ok(response) => {
                            if server_id.is_none() {
                                info!(
                                    server_id = %response.server_id,
                                    ttl_secs = response.ttl_secs,
                                    "Registered with master server"
                                );
                            } else {
                                debug!(
                                    server_id = %response.server_id,
                                    ttl_secs = response.ttl_secs,
                                    "Heartbeat sent successfully"
                                );
                            }
                            server_id = Some(response.server_id);
                        }
                        Err(e) => {
                            warn!(error = %e, "Heartbeat failed");
                        }
                    }
                }
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        info!("Heartbeat task shutting down");
                        break;
                    }
                }
            }
        }
    }
}

/// Errors that can occur during heartbeat operations.
#[derive(Debug, thiserror::Error)]
pub enum HeartbeatError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Server error ({0}): {1}")]
    ServerError(u16, String),

    #[error("Failed to parse response: {0}")]
    ParseError(reqwest::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_state_default() {
        let state = HeartbeatState::default();
        assert_eq!(state.get_player_count(), 0);
    }

    #[test]
    fn test_heartbeat_state_update() {
        let state = HeartbeatState::new();
        assert_eq!(state.get_player_count(), 0);

        state.set_player_count(5);
        assert_eq!(state.get_player_count(), 5);

        state.set_player_count(10);
        assert_eq!(state.get_player_count(), 10);
    }

    #[test]
    fn test_build_request() {
        let config = MasterConfig::new("http://localhost:8080", "Test Server")
            .with_region("eu-west")
            .with_tags(vec!["test".to_string()])
            .with_game_modes(vec!["ffa".to_string(), "ctf".to_string()]);

        let state = HeartbeatState::new();
        state.set_player_count(3);

        let (tx, rx) = watch::channel(false);
        let task = HeartbeatTask::new(config, "192.168.1.100".to_string(), 7777, 16, state, rx);

        let request = task.build_request();
        assert_eq!(request.name, "Test Server");
        assert_eq!(request.host, "192.168.1.100");
        assert_eq!(request.port, 7777);
        assert_eq!(request.region, "eu-west");
        assert_eq!(request.tags, vec!["test"]);
        assert_eq!(request.player_count, 3);
        assert_eq!(request.max_players, 16);
        assert_eq!(request.game_modes, vec!["ffa", "ctf"]);

        drop(tx); // Clean up
    }

    #[test]
    fn test_build_request_protocol_version() {
        let config = MasterConfig::new("http://localhost:8080", "Test");
        let state = HeartbeatState::new();
        let (_tx, rx) = watch::channel(false);

        let task = HeartbeatTask::new(config, "127.0.0.1".to_string(), 7777, 8, state, rx);

        let request = task.build_request();
        // Protocol version should be the crate version
        assert!(!request.protocol_version.is_empty());
    }
}
