//! Quick join request and result types.

use plix_common::server_browser::ServerEntry;
use tracing::{info, warn};

use super::filtering::filter_servers;
use super::scoring::score_servers;
use super::selection::select_best_server;

/// Valid game mode values for quick join (case-insensitive).
pub const VALID_MODES: &[&str] = &["tdm", "ffa", "ctf", "br", "training", "any"];

/// Valid region values for quick join (case-insensitive).
pub const VALID_REGIONS: &[&str] = &["eu", "us", "asia", "any"];

/// A request to quickly join a game server.
#[derive(Debug, Clone)]
pub struct QuickJoinRequest {
    /// Requested game mode (e.g., "tdm", "ffa", "any")
    pub mode: String,
    /// Requested region (e.g., "eu", "us", "any")
    pub region: String,
}

impl QuickJoinRequest {
    /// Create a new quick join request.
    ///
    /// Mode and region are normalized to lowercase.
    pub fn new(mode: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            mode: mode.into().to_lowercase(),
            region: region.into().to_lowercase(),
        }
    }

    /// Check if the mode is valid.
    pub fn is_valid_mode(mode: &str) -> bool {
        VALID_MODES.contains(&mode.to_lowercase().as_str())
    }

    /// Check if the region is valid.
    pub fn is_valid_region(region: &str) -> bool {
        VALID_REGIONS.contains(&region.to_lowercase().as_str())
    }

    /// Create a request with expanded region (fallback step 1).
    pub fn with_any_region(&self) -> Self {
        Self {
            mode: self.mode.clone(),
            region: "any".to_string(),
        }
    }

    /// Create a request with expanded mode and region (fallback step 2).
    pub fn with_any_mode_and_region(&self) -> Self {
        Self {
            mode: "any".to_string(),
            region: "any".to_string(),
        }
    }

    /// Log the quick join request (T021).
    pub fn log(&self) {
        info!(
            mode = %self.mode,
            region = %self.region,
            "Quick join request initiated"
        );
    }
}

impl Default for QuickJoinRequest {
    fn default() -> Self {
        Self {
            mode: "tdm".to_string(),
            region: "any".to_string(),
        }
    }
}

/// Result of a quick join attempt.
#[derive(Debug, Clone)]
pub struct QuickJoinResult {
    /// Selected server (if successful)
    pub selected_server: Option<ServerEntry>,
    /// Whether fallback criteria were used
    pub fallback_used: bool,
    /// Reason for fallback (if applicable)
    pub fallback_reason: Option<String>,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Number of connection attempts made
    pub attempts: u8,
}

impl QuickJoinResult {
    /// Maximum connection attempts before giving up.
    pub const MAX_ATTEMPTS: u8 = 3;

    /// Create a successful result.
    pub fn success(server: ServerEntry) -> Self {
        Self {
            selected_server: Some(server),
            fallback_used: false,
            fallback_reason: None,
            error_message: None,
            attempts: 1,
        }
    }

    /// Create a successful result with fallback.
    pub fn success_with_fallback(server: ServerEntry, reason: impl Into<String>) -> Self {
        Self {
            selected_server: Some(server),
            fallback_used: true,
            fallback_reason: Some(reason.into()),
            error_message: None,
            attempts: 1,
        }
    }

    /// Create a failed result.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            selected_server: None,
            fallback_used: false,
            fallback_reason: None,
            error_message: Some(error.into()),
            attempts: 0,
        }
    }

    /// Create a failed result after retries.
    pub fn failure_after_retries(error: impl Into<String>, attempts: u8) -> Self {
        Self {
            selected_server: None,
            fallback_used: false,
            fallback_reason: None,
            error_message: Some(error.into()),
            attempts,
        }
    }

    /// Check if the result is successful.
    pub fn is_success(&self) -> bool {
        self.selected_server.is_some() && self.error_message.is_none()
    }
}

/// Select a server with fallback cascade (T037-T040).
///
/// Tries in order:
/// 1. Exact mode + region match
/// 2. Exact mode + any region (fallback step 1)
/// 3. Any mode + any region (fallback step 2)
///
/// Returns QuickJoinResult with server selection and fallback information.
pub fn select_server(
    servers: &[ServerEntry],
    request: &QuickJoinRequest,
    protocol_version: &str,
    current_time: u64,
) -> QuickJoinResult {
    // Log the request
    request.log();

    // T037: Try exact match first
    let filtered = filter_servers(servers, request, protocol_version);
    info!(
        candidate_count = filtered.len(),
        mode = %request.mode,
        region = %request.region,
        "Filtered servers for exact match"
    );

    if !filtered.is_empty() {
        let scored = score_servers(&filtered, request, current_time);
        if let Some(selected) = select_best_server(&scored) {
            return QuickJoinResult::success(selected.server);
        }
    }

    // T038: Fallback step 1 - expand region to "any"
    if request.region != "any" {
        let fallback_request = request.with_any_region();
        let filtered = filter_servers(servers, &fallback_request, protocol_version);
        info!(
            candidate_count = filtered.len(),
            mode = %fallback_request.mode,
            region = %fallback_request.region,
            "Filtered servers after region fallback"
        );

        if !filtered.is_empty() {
            let scored = score_servers(&filtered, &fallback_request, current_time);
            if let Some(selected) = select_best_server(&scored) {
                let reason = format!(
                    "No servers found in '{}' region, expanded to any region",
                    request.region
                );
                // T040: Set fallback_used and fallback_reason
                return QuickJoinResult::success_with_fallback(selected.server, reason);
            }
        }
    }

    // T039: Fallback step 2 - expand mode to "any"
    if request.mode != "any" {
        let fallback_request = request.with_any_mode_and_region();
        let filtered = filter_servers(servers, &fallback_request, protocol_version);
        info!(
            candidate_count = filtered.len(),
            mode = %fallback_request.mode,
            region = %fallback_request.region,
            "Filtered servers after mode fallback"
        );

        if !filtered.is_empty() {
            let scored = score_servers(&filtered, &fallback_request, current_time);
            if let Some(selected) = select_best_server(&scored) {
                let reason = format!(
                    "No servers found for '{}' mode in requested region, expanded to any mode",
                    request.mode
                );
                // T040: Set fallback_used and fallback_reason
                return QuickJoinResult::success_with_fallback(selected.server, reason);
            }
        }
    }

    // T041: No servers available after all fallbacks
    warn!(
        mode = %request.mode,
        region = %request.region,
        "No servers available after fallback cascade"
    );
    QuickJoinResult::failure("No servers available")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_new_normalizes_case() {
        let req = QuickJoinRequest::new("TDM", "EU");
        assert_eq!(req.mode, "tdm");
        assert_eq!(req.region, "eu");
    }

    #[test]
    fn test_valid_modes() {
        assert!(QuickJoinRequest::is_valid_mode("tdm"));
        assert!(QuickJoinRequest::is_valid_mode("TDM"));
        assert!(QuickJoinRequest::is_valid_mode("ffa"));
        assert!(QuickJoinRequest::is_valid_mode("ctf"));
        assert!(QuickJoinRequest::is_valid_mode("br"));
        assert!(QuickJoinRequest::is_valid_mode("training"));
        assert!(QuickJoinRequest::is_valid_mode("any"));
        assert!(!QuickJoinRequest::is_valid_mode("invalid"));
    }

    #[test]
    fn test_valid_regions() {
        assert!(QuickJoinRequest::is_valid_region("eu"));
        assert!(QuickJoinRequest::is_valid_region("EU"));
        assert!(QuickJoinRequest::is_valid_region("us"));
        assert!(QuickJoinRequest::is_valid_region("asia"));
        assert!(QuickJoinRequest::is_valid_region("any"));
        assert!(!QuickJoinRequest::is_valid_region("invalid"));
    }

    #[test]
    fn test_fallback_requests() {
        let req = QuickJoinRequest::new("tdm", "eu");

        let fallback1 = req.with_any_region();
        assert_eq!(fallback1.mode, "tdm");
        assert_eq!(fallback1.region, "any");

        let fallback2 = req.with_any_mode_and_region();
        assert_eq!(fallback2.mode, "any");
        assert_eq!(fallback2.region, "any");
    }

    #[test]
    fn test_result_success() {
        let server = ServerEntry {
            server_id: "test".to_string(),
            name: "Test Server".to_string(),
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

        let result = QuickJoinResult::success(server.clone());
        assert!(result.is_success());
        assert!(!result.fallback_used);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_result_failure() {
        let result = QuickJoinResult::failure("No servers available");
        assert!(!result.is_success());
        assert_eq!(
            result.error_message,
            Some("No servers available".to_string())
        );
    }

    // T043: Tests for fallback cascade
    fn make_server(
        name: &str,
        region: &str,
        mode: &str,
        players: u8,
        protocol_version: &str,
    ) -> ServerEntry {
        ServerEntry {
            server_id: format!("id_{}", name),
            name: name.to_string(),
            host: "127.0.0.1".to_string(),
            port: 7777,
            region: region.to_string(),
            tags: vec![],
            player_count: players,
            max_players: 16,
            game_modes: vec![mode.to_string()],
            protocol_version: protocol_version.to_string(),
            last_seen: 1000,
        }
    }

    #[test]
    fn test_select_server_exact_match() {
        let servers = vec![
            make_server("EU TDM", "eu", "tdm", 5, "0.1.0"),
            make_server("US TDM", "us", "tdm", 3, "0.1.0"),
        ];
        let request = QuickJoinRequest::new("tdm", "eu");

        let result = select_server(&servers, &request, "0.1.0", 1000);

        assert!(result.is_success());
        assert!(!result.fallback_used);
        assert_eq!(result.selected_server.unwrap().name, "EU TDM");
    }

    #[test]
    fn test_select_server_region_fallback() {
        // No servers in EU, but US has servers
        let servers = vec![make_server("US TDM", "us", "tdm", 5, "0.1.0")];
        let request = QuickJoinRequest::new("tdm", "eu");

        let result = select_server(&servers, &request, "0.1.0", 1000);

        assert!(result.is_success());
        assert!(result.fallback_used);
        assert!(result.fallback_reason.is_some());
        assert!(result.fallback_reason.unwrap().contains("region"));
        assert_eq!(result.selected_server.unwrap().name, "US TDM");
    }

    #[test]
    fn test_select_server_mode_fallback() {
        // No TDM servers, but FFA servers exist
        let servers = vec![make_server("US FFA", "us", "ffa", 5, "0.1.0")];
        let request = QuickJoinRequest::new("tdm", "eu");

        let result = select_server(&servers, &request, "0.1.0", 1000);

        assert!(result.is_success());
        assert!(result.fallback_used);
        assert!(result.fallback_reason.is_some());
        assert!(result.fallback_reason.unwrap().contains("mode"));
        assert_eq!(result.selected_server.unwrap().name, "US FFA");
    }

    // T044: Test for empty server list handling
    #[test]
    fn test_select_server_empty_list() {
        let servers: Vec<ServerEntry> = vec![];
        let request = QuickJoinRequest::new("tdm", "eu");

        let result = select_server(&servers, &request, "0.1.0", 1000);

        assert!(!result.is_success());
        assert_eq!(
            result.error_message,
            Some("No servers available".to_string())
        );
    }

    #[test]
    fn test_select_server_no_compatible_servers() {
        // All servers have wrong protocol version
        let servers = vec![make_server("Old Server", "eu", "tdm", 5, "0.0.1")];
        let request = QuickJoinRequest::new("tdm", "eu");

        let result = select_server(&servers, &request, "0.1.0", 1000);

        assert!(!result.is_success());
        assert_eq!(
            result.error_message,
            Some("No servers available".to_string())
        );
    }

    #[test]
    fn test_select_server_prefers_exact_match_over_fallback() {
        // Both exact match and fallback available - should prefer exact
        let servers = vec![
            make_server("EU TDM", "eu", "tdm", 3, "0.1.0"),
            make_server("US TDM", "us", "tdm", 10, "0.1.0"), // More players but wrong region
        ];
        let request = QuickJoinRequest::new("tdm", "eu");

        let result = select_server(&servers, &request, "0.1.0", 1000);

        assert!(result.is_success());
        assert!(!result.fallback_used); // Should NOT use fallback
        assert_eq!(result.selected_server.unwrap().name, "EU TDM");
    }
}
