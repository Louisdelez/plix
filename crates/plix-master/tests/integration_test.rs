//! Integration tests for the plix master server.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use plix_common::server_browser::ServerListResponse;
use plix_master::{create_app, HeartbeatRequest, ServerRegistry};
use tower::ServiceExt;

fn create_test_app() -> axum::Router {
    let registry = ServerRegistry::new(60, 100);
    create_app(registry)
}

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["server_count"], 0);
}

#[tokio::test]
async fn test_servers_endpoint_empty_initially() {
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
    assert_eq!(list.total, 0);
}

#[tokio::test]
async fn test_heartbeat_invalid_json_returns_400() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/heartbeat")
                .header("content-type", "application/json")
                .body(Body::from("not valid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 4xx error for invalid JSON
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn test_heartbeat_validation_empty_name() {
    let app = create_test_app();

    let request = HeartbeatRequest {
        name: "".to_string(), // Empty name - invalid
        host: "192.168.1.100".to_string(),
        port: 7777,
        region: "eu-west".to_string(),
        tags: vec![],
        player_count: 0,
        max_players: 10,
        game_modes: vec![],
        protocol_version: "0.1.0".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/heartbeat")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.1")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_heartbeat_validation_name_too_long() {
    let app = create_test_app();

    let request = HeartbeatRequest {
        name: "a".repeat(65), // Over 64 char limit
        host: "192.168.1.100".to_string(),
        port: 7777,
        region: "eu-west".to_string(),
        tags: vec![],
        player_count: 0,
        max_players: 10,
        game_modes: vec![],
        protocol_version: "0.1.0".to_string(),
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/heartbeat")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.1")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
