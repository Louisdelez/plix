//! Load test: 8-16 bots for 60 seconds
//! T076 [US2]
//!
//! NOTE: This is an integration test that requires a running server.
//! Run with: cargo test --package plix-tools --test load_test -- --ignored

use std::net::SocketAddr;
use std::time::Duration;

use plix_tools::bot::BotSpawner;

/// Test 8 bots connected for 60 seconds
/// Requires server running on 127.0.0.1:7777
#[tokio::test]
#[ignore] // Run manually with server
async fn test_8_bots_60_seconds() {
    let server: SocketAddr = "127.0.0.1:7777".parse().unwrap();

    let spawner = BotSpawner::new(server, 8, 60);
    let result = spawner.run().await;

    // Assertions
    assert_eq!(result.bots, 8, "Should track 8 bots");
    assert!(
        result.duration >= Duration::from_secs(60),
        "Should run for at least 60 seconds"
    );

    // Should have sent packets at ~60Hz per bot
    // 8 bots * 60 seconds * 60 Hz = 28800 expected
    // Allow some tolerance for connection overhead
    let expected_min = 8 * 60 * 55; // ~55 Hz minimum
    assert!(
        result.packets_sent >= expected_min,
        "Expected at least {} packets sent, got {}",
        expected_min,
        result.packets_sent
    );

    println!("Load test results:");
    println!("  Duration: {:?}", result.duration);
    println!("  Bots: {}", result.bots);
    println!("  Packets sent: {}", result.packets_sent);
    println!("  Packets recv: {}", result.packets_recv);
    println!("  PPS sent: {:.1}", result.pps_sent());
    println!("  PPS recv: {:.1}", result.pps_recv());
}

/// Test 16 bots connected for 30 seconds (shorter duration, more bots)
/// Requires server running on 127.0.0.1:7777
#[tokio::test]
#[ignore] // Run manually with server
async fn test_16_bots_30_seconds() {
    let server: SocketAddr = "127.0.0.1:7777".parse().unwrap();

    let spawner = BotSpawner::new(server, 16, 30);
    let result = spawner.run().await;

    // Assertions
    assert_eq!(result.bots, 16, "Should track 16 bots");
    assert!(
        result.duration >= Duration::from_secs(30),
        "Should run for at least 30 seconds"
    );

    // 16 bots * 30 seconds * 55 Hz = 26400 expected minimum
    let expected_min = 16 * 30 * 55;
    assert!(
        result.packets_sent >= expected_min,
        "Expected at least {} packets sent, got {}",
        expected_min,
        result.packets_sent
    );

    println!("Load test results:");
    println!("  Duration: {:?}", result.duration);
    println!("  Bots: {}", result.bots);
    println!("  Packets sent: {}", result.packets_sent);
    println!("  Packets recv: {}", result.packets_recv);
    println!("  PPS sent: {:.1}", result.pps_sent());
    println!("  PPS recv: {:.1}", result.pps_recv());
}

/// Quick smoke test with 2 bots for 5 seconds
/// Requires server running on 127.0.0.1:7777
#[tokio::test]
#[ignore] // Run manually with server
async fn test_2_bots_smoke() {
    let server: SocketAddr = "127.0.0.1:7777".parse().unwrap();

    let spawner = BotSpawner::new(server, 2, 5);
    let result = spawner.run().await;

    assert_eq!(result.bots, 2);
    assert!(result.packets_sent > 0, "Should have sent some packets");

    println!(
        "Smoke test: {} packets sent in {:?}",
        result.packets_sent, result.duration
    );
}
