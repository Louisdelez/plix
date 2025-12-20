//! Plix Headless Server - Production-ready dedicated server binary
//!
//! A minimal, headless server binary optimized for deployment:
//! - No GUI dependencies
//! - Proper signal handling (SIGINT/SIGTERM)
//! - Graceful shutdown with timeout
//! - Structured exit codes for systemd/Docker
//! - Config validation with clear error messages
//!
//! Usage:
//!   plix-server-headless [OPTIONS]
//!   plix-server-headless --config server.toml

// Include shadow-rs generated build info
mod shadow {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

use std::path::PathBuf;

use clap::Parser;
use tracing::{error, info, warn};

use plix_common::build_info::BuildInfo;
use plix_server::config::{ConfigError, ServerConfig};
use plix_server::exit_codes::ExitCode;
use plix_server::shutdown::GracefulShutdown;
use plix_server::Server;

/// Get build information from shadow-rs constants
fn get_build_info() -> BuildInfo {
    BuildInfo::from_shadow(
        shadow::PKG_VERSION,
        shadow::COMMIT_HASH,
        shadow::SHORT_COMMIT,
        shadow::BUILD_TIME,
        shadow::RUST_VERSION,
        shadow::BRANCH,
        shadow::GIT_CLEAN,
        shadow::BUILD_TARGET,
    )
}

/// Plix headless dedicated server
#[derive(Parser, Debug)]
#[command(name = "plix-server-headless")]
#[command(about = "Headless dedicated server for Plix")]
#[command(version = shadow::CLAP_LONG_VERSION)]
struct Args {
    /// Path to server configuration file (TOML format)
    #[arg(long, short = 'c')]
    config: Option<PathBuf>,

    /// IP address to bind to
    #[arg(long, default_value = "0.0.0.0")]
    bind: String,

    /// UDP port for game traffic
    #[arg(long, default_value = "7777")]
    port: u16,

    /// Maximum number of players
    #[arg(long, default_value = "32")]
    max_players: u8,

    /// Arena to load
    #[arg(long, default_value = "ffa_small")]
    arena: String,

    /// Server tick rate (Hz)
    #[arg(long, default_value = "30")]
    tick_rate: u8,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Graceful shutdown timeout in seconds
    #[arg(long, default_value = "5")]
    shutdown_timeout: u64,

    /// Path to assets directory
    #[arg(long)]
    assets_dir: Option<PathBuf>,

    /// Validate config and exit without starting server
    #[arg(long)]
    validate: bool,
}

fn main() {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("plix={}", args.log_level).parse().unwrap()),
        )
        .init();

    // Log version info on startup
    let build_info = get_build_info();
    info!(
        version = %build_info.version,
        commit = %build_info.commit_sha_short,
        target = %build_info.target_triple,
        "Starting plix-server-headless {}",
        build_info.display_version()
    );

    // Load and validate configuration
    let config = match load_config(&args) {
        Ok(c) => c,
        Err(errors) => {
            for err in &errors {
                error!("{}", err);
            }
            ExitCode::Misuse.exit();
        }
    };

    // Validate-only mode
    if args.validate {
        info!("Configuration validation passed");
        std::process::exit(0);
    }

    // Run the server
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async {
        run_server(config, args.shutdown_timeout).await;
    });
}

/// Load configuration from file or CLI arguments
fn load_config(args: &Args) -> Result<ServerConfig, Vec<ConfigError>> {
    let mut config = if let Some(config_path) = &args.config {
        info!(path = ?config_path, "Loading configuration from file");

        match ServerConfig::load_from_file(config_path) {
            Ok(c) => c,
            Err(e) => {
                return Err(vec![e]);
            }
        }
    } else {
        ServerConfig::default()
    };

    // CLI arguments override config file
    config.network.bind_address = args.bind.clone();
    config.network.port = args.port;
    config.network.max_players = args.max_players;
    config.game.arena = args.arena.clone();
    config.game.tick_rate = args.tick_rate;
    config.persistence.shutdown_timeout_secs = args.shutdown_timeout;

    // Validate configuration
    config.validate()?;

    Ok(config)
}

/// Run the server with graceful shutdown handling
async fn run_server(config: ServerConfig, shutdown_timeout: u64) {
    let shutdown = GracefulShutdown::new(shutdown_timeout);

    // Start the server
    let addr = format!("{}:{}", config.network.bind_address, config.network.port);
    info!(addr = %addr, "Starting server");

    // Create server instance
    let server_result = Server::new(
        addr.parse().unwrap(),
        &config.game.arena,
        None,
        config.game.tick_rate,
    );

    let mut server = match server_result {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create server: {}", e);

            // Determine appropriate exit code based on error type
            let error_str = e.to_string().to_lowercase();
            if error_str.contains("address") || error_str.contains("bind") || error_str.contains("port") {
                ExitCode::BindFailed.exit();
            } else if error_str.contains("arena") || error_str.contains("asset") {
                ExitCode::AssetLoadFailed.exit();
            } else {
                ExitCode::GeneralError.exit();
            }
        }
    };

    // Run server with shutdown handling
    tokio::select! {
        result = server.run() => {
            match result {
                Ok(()) => {
                    info!("Server stopped normally");
                }
                Err(e) => {
                    error!("Server error: {}", e);
                    ExitCode::GeneralError.exit();
                }
            }
        }
        code = shutdown.wait_for_shutdown() => {
            info!("Initiating graceful shutdown...");

            // Attempt graceful shutdown
            let shutdown_result = tokio::time::timeout(
                std::time::Duration::from_secs(shutdown_timeout),
                server.shutdown()
            ).await;

            match shutdown_result {
                Ok(Ok(())) => {
                    info!("Graceful shutdown complete");
                    std::process::exit(code.code());
                }
                Ok(Err(e)) => {
                    error!("Shutdown error: {}", e);
                    ExitCode::GeneralError.exit();
                }
                Err(_) => {
                    warn!("Shutdown timeout exceeded, forcing exit");
                    ExitCode::ShutdownTimeout.exit();
                }
            }
        }
    }

    std::process::exit(0);
}
