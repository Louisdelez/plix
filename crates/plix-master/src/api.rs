//! HTTP API routes for the master server.
//!
//! Provides endpoints for:
//! - POST /heartbeat - Server registration and updates
//! - GET /servers - List active servers
//! - GET /health - Health check

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use plix_common::server_browser::ServerListResponse;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tracing::{info, warn};

use crate::state::ServerRegistry;
use crate::types::{HeartbeatRequest, HeartbeatResponse};
use crate::validation::validate_heartbeat;

/// Create the application router with all routes.
pub fn create_app(registry: Arc<ServerRegistry>) -> Router {
    Router::new()
        .route("/heartbeat", post(handle_heartbeat))
        .route("/servers", get(handle_list_servers))
        .route("/health", get(handle_health))
        .with_state(registry)
}

/// Extract client IP from X-Forwarded-For header or ConnectInfo.
fn extract_client_ip(headers: &HeaderMap, connect_info: Option<SocketAddr>) -> IpAddr {
    // Check X-Forwarded-For header first (for proxied requests)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            // Take the first IP in the chain (original client)
            if let Some(first_ip) = value.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Fall back to connection info
    connect_info
        .map(|addr| addr.ip())
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
}

/// Handle POST /heartbeat - register or update a server.
async fn handle_heartbeat(
    State(registry): State<Arc<ServerRegistry>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    headers: HeaderMap,
    Json(request): Json<HeartbeatRequest>,
) -> Result<Json<HeartbeatResponse>, (StatusCode, String)> {
    let ip = extract_client_ip(&headers, connect_info.map(|ci| ci.0));

    // Check rate limit
    if !registry.check_rate_limit(ip).await {
        warn!(ip = %ip, "Rate limit exceeded for heartbeat request");
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Max 10 requests per minute.".to_string(),
        ));
    }

    // Validate request fields
    if let Err(e) = validate_heartbeat(&request) {
        warn!(ip = %ip, error = %e, "Invalid heartbeat request");
        return Err((StatusCode::BAD_REQUEST, format!("Validation error: {}", e)));
    }

    // Register the server
    let server_id = registry.register(request.clone()).await;

    info!(
        server_id = %server_id,
        name = %request.name,
        host = %request.host,
        port = request.port,
        players = %format!("{}/{}", request.player_count, request.max_players),
        "Server heartbeat registered"
    );

    Ok(Json(HeartbeatResponse {
        server_id,
        ttl_secs: registry.ttl_secs(),
    }))
}

/// Handle GET /servers - list all active servers.
async fn handle_list_servers(
    State(registry): State<Arc<ServerRegistry>>,
) -> Json<ServerListResponse> {
    let servers = registry.get_active_servers().await;
    let total = servers.len();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    info!(count = total, "Server list requested");

    Json(ServerListResponse {
        servers,
        total,
        timestamp,
    })
}

/// Handle GET /health - health check endpoint.
async fn handle_health(State(registry): State<Arc<ServerRegistry>>) -> Json<HealthResponse> {
    let server_count = registry.active_count().await;

    Json(HealthResponse {
        status: "ok".to_string(),
        server_count,
    })
}

/// Health check response.
#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
    server_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        let registry = ServerRegistry::new(60, 100);
        create_app(registry)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_servers_endpoint_empty() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/servers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list: ServerListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.servers.is_empty());
    }
}
