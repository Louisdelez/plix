//! Plix Master Server - Server directory for multiplayer server discovery.
//!
//! This binary runs the master server that maintains a directory of active
//! game servers. Game servers send periodic heartbeats to register themselves,
//! and clients query the master to discover available servers.

use clap::Parser;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use plix_master::{create_app, ServerRegistry};

/// Plix Master Server - Server browser directory service
#[derive(Parser, Debug)]
#[command(name = "plix-master")]
#[command(
    about = "Master server for plix server browser - maintains directory of active game servers"
)]
struct Args {
    /// Address to bind the HTTP server to
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    bind: SocketAddr,

    /// Time-to-live for server entries in seconds (servers not sending heartbeats within this time are removed)
    #[arg(long, default_value = "60")]
    ttl: u64,

    /// Maximum heartbeat requests per IP per minute
    #[arg(long, default_value = "10")]
    rate_limit: u32,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: Level,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.log_level)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!(
        bind = %args.bind,
        ttl_secs = args.ttl,
        rate_limit = args.rate_limit,
        "Starting plix master server"
    );

    // Create server registry with configured TTL and rate limit
    let registry = ServerRegistry::new(args.ttl, args.rate_limit);

    // Create the application router
    let app = create_app(registry);

    // Start the HTTP server
    let listener = tokio::net::TcpListener::bind(args.bind)
        .await
        .expect("Failed to bind to address");

    info!("Master server listening on {}", args.bind);

    axum::serve(listener, app).await.expect("Server error");
}
