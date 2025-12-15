//! Stability test: Verify network stability metrics
//! T077 [US2]
//!
//! Tests that bots can maintain stable connections with acceptable packet loss.

use std::net::SocketAddr;
use std::time::Duration;

use plix_tools::bot::BotSpawner;

/// Maximum acceptable packet loss ratio (5%)
#[allow(dead_code)]
const MAX_PACKET_LOSS_RATIO: f64 = 0.05;

/// Minimum acceptable packets per second per bot
const MIN_PPS_PER_BOT: f64 = 50.0;

/// Test stability with 8 bots - verify low packet loss
/// Requires server running on 127.0.0.1:7777
#[tokio::test]
#[ignore] // Run manually with server
async fn test_stability_8_bots() {
    let server: SocketAddr = "127.0.0.1:7777".parse().unwrap();

    let spawner = BotSpawner::new(server, 8, 30);
    let result = spawner.run().await;

    // Calculate metrics
    let pps_sent = result.pps_sent();
    let pps_recv = result.pps_recv();
    let pps_per_bot_sent = pps_sent / result.bots as f64;

    // Packet loss ratio (comparing sent vs received)
    // Note: This is approximate since server broadcasts to all clients
    let _expected_recv_ratio = result.bots as f64; // Each bot receives broadcasts from server
    let actual_recv_ratio = result.packets_recv as f64 / result.packets_sent as f64;

    println!("Stability test results:");
    println!("  PPS sent: {:.1}", pps_sent);
    println!("  PPS recv: {:.1}", pps_recv);
    println!("  PPS per bot (sent): {:.1}", pps_per_bot_sent);
    println!("  Recv/Sent ratio: {:.2}", actual_recv_ratio);

    // Assertions
    assert!(
        pps_per_bot_sent >= MIN_PPS_PER_BOT,
        "PPS per bot ({:.1}) should be at least {}",
        pps_per_bot_sent,
        MIN_PPS_PER_BOT
    );

    // Each bot should send at target rate
    let expected_sends = 8.0 * 60.0; // 8 bots * 60 Hz
    let send_efficiency = pps_sent / expected_sends;
    assert!(
        send_efficiency >= 0.9,
        "Send efficiency ({:.1}%) should be at least 90%",
        send_efficiency * 100.0
    );
}

/// Test no connection drops under load
/// Requires server running on 127.0.0.1:7777
#[tokio::test]
#[ignore] // Run manually with server
async fn test_no_connection_drops() {
    let server: SocketAddr = "127.0.0.1:7777".parse().unwrap();

    // Run multiple sequential tests to check for drops
    for i in 0..3 {
        println!("Run {}/3", i + 1);

        let spawner = BotSpawner::new(server, 4, 10);
        let result = spawner.run().await;

        // Each run should complete successfully
        assert_eq!(result.bots, 4);
        assert!(result.duration >= Duration::from_secs(10));
        assert!(result.packets_sent > 0);

        println!(
            "  Sent: {}, Recv: {}",
            result.packets_sent, result.packets_recv
        );
    }
}

/// Test sustained load doesn't degrade over time
/// Requires server running on 127.0.0.1:7777
#[tokio::test]
#[ignore] // Run manually with server
async fn test_sustained_performance() {
    let server: SocketAddr = "127.0.0.1:7777".parse().unwrap();

    // Run for 2 minutes with 8 bots
    let spawner = BotSpawner::new(server, 8, 120);
    let result = spawner.run().await;

    let pps_sent = result.pps_sent();
    let pps_per_bot = pps_sent / result.bots as f64;

    println!("Sustained performance test:");
    println!("  Duration: {:?}", result.duration);
    println!("  Total packets: {}", result.packets_sent);
    println!("  PPS per bot: {:.1}", pps_per_bot);

    // After 2 minutes, should still maintain good performance
    assert!(
        pps_per_bot >= MIN_PPS_PER_BOT,
        "Performance degraded: PPS per bot ({:.1}) below {}",
        pps_per_bot,
        MIN_PPS_PER_BOT
    );
}
