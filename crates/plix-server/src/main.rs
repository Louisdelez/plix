//! Plix Server - Entry point
//!
//! Usage: plix-server [OPTIONS]
//!
//! Options:
//!   --port <PORT>          UDP port to listen on [default: 7777]
//!   --tickrate <RATE>      Server tick rate (20-60) [default: 60]
//!   --max-players <N>      Maximum concurrent players [default: 16]
//!   --arena <NAME>         Arena name from assets/arenas/ [default: test_arena]
//!   --log-level <LEVEL>    Log verbosity [default: info]

use std::path::PathBuf;

use clap::Parser;
use tracing::{error, info};

use plix_server::{Server, ServerConfig};

/// Plix authoritative game server
#[derive(Parser, Debug)]
#[command(name = "plix-server")]
#[command(about = "Authoritative multiplayer game server for Plix")]
#[command(version)]
struct Args {
    /// UDP port to listen on
    #[arg(long, default_value = "7777")]
    port: u16,

    /// Server tick rate (20-60 Hz)
    #[arg(long, default_value = "60")]
    tickrate: u8,

    /// Maximum concurrent players
    #[arg(long, default_value = "16")]
    max_players: u8,

    /// Arena name (from assets/arenas/)
    #[arg(long, default_value = "test_arena")]
    arena: String,

    /// Assets directory path
    #[arg(long, default_value = "assets")]
    assets_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("plix={}", args.log_level).parse().unwrap()),
        )
        .init();

    info!(
        port = args.port,
        tickrate = args.tickrate,
        max_players = args.max_players,
        arena = %args.arena,
        "Starting Plix server"
    );

    // Create server config
    let config = ServerConfig {
        port: args.port,
        tick_rate: args.tickrate,
        max_players: args.max_players,
        arena_name: args.arena,
        assets_dir: args.assets_dir,
    };

    // Create and run server
    match Server::new(config).await {
        Ok(server) => {
            if let Err(e) = server.run().await {
                error!(error = %e, "Server error");
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to start server");
        }
    }

    info!("Server shutdown");
}
