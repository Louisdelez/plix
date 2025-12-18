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
//!   --master-url <URL>     Master server URL for server browser registration
//!   --server-name <NAME>   Display name in server browser [default: Plix Server]
//!   --region <REGION>      Geographic region [default: unknown]

use std::path::PathBuf;

use clap::Parser;
use tracing::{error, info};

use plix_server::{master_announce::MasterConfig, Server, ServerConfig};

/// Plix authoritative game server
///
/// Configuration priority: CLI flags > Environment variables > Defaults
/// Environment variables are prefixed with PLIX_ (e.g., PLIX_PORT, PLIX_SERVER_NAME)
#[derive(Parser, Debug)]
#[command(name = "plix-server")]
#[command(about = "Authoritative multiplayer game server for Plix")]
#[command(version)]
struct Args {
    /// UDP port to listen on
    #[arg(long, default_value = "7777", env = "PLIX_PORT")]
    port: u16,

    /// Server tick rate (20-60 Hz)
    #[arg(long, default_value = "60", env = "PLIX_TICKRATE")]
    tickrate: u8,

    /// Maximum concurrent players
    #[arg(long, default_value = "16", env = "PLIX_MAX_PLAYERS")]
    max_players: u8,

    /// Arena name (from assets/arenas/)
    #[arg(long, default_value = "test_arena", env = "PLIX_ARENA")]
    arena: String,

    /// Assets directory path
    #[arg(long, default_value = "assets", env = "PLIX_ASSETS_DIR")]
    assets_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", env = "PLIX_LOG_LEVEL")]
    log_level: String,

    /// Enable world persistence (saves block edits between restarts)
    #[arg(long, default_value = "false", env = "PLIX_PERSISTENCE")]
    persistence: bool,

    /// World ID for persistence (defaults to server_<arena>)
    #[arg(long, env = "PLIX_WORLD_ID")]
    world_id: Option<String>,

    /// Auto-save interval in seconds (default: 300 = 5 min)
    #[arg(long, default_value = "300", env = "PLIX_AUTOSAVE_INTERVAL")]
    autosave_interval: u64,

    /// Master server URL for server browser registration (e.g., http://localhost:8080)
    #[arg(long, env = "PLIX_MASTER_URL")]
    master_url: Option<String>,

    /// Server display name in server browser
    #[arg(long, default_value = "Plix Server", env = "PLIX_SERVER_NAME")]
    server_name: String,

    /// Geographic region for server browser (e.g., eu-west, us-east)
    #[arg(long, default_value = "unknown", env = "PLIX_REGION")]
    region: String,

    /// Server tags for filtering (comma-separated, e.g., "competitive,ranked")
    #[arg(long, value_delimiter = ',', env = "PLIX_TAGS")]
    tags: Vec<String>,

    /// Game modes supported (comma-separated, e.g., "ffa,ctf")
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "ffa",
        env = "PLIX_GAME_MODE"
    )]
    game_modes: Vec<String>,
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

    // Build master server config
    let master_config = if let Some(master_url) = args.master_url {
        MasterConfig {
            enabled: true,
            url: master_url,
            name: args.server_name,
            region: args.region,
            tags: args.tags,
            game_modes: args.game_modes,
            heartbeat_interval_secs: 20,
        }
    } else {
        MasterConfig::default()
    };

    // Create server config
    let config = ServerConfig {
        port: args.port,
        tick_rate: args.tickrate,
        max_players: args.max_players,
        arena_name: args.arena,
        assets_dir: args.assets_dir,
        persistence_enabled: args.persistence,
        world_id: args.world_id,
        auto_save_interval_secs: args.autosave_interval,
        block_physics: plix_common::block_physics::BlockPhysicsConfig::default(),
        master_config,
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
