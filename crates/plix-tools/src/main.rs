//! Plix Tools - Development utilities
//!
//! Usage: plix-tools <COMMAND> [OPTIONS]
//!
//! Commands:
//!   bot             Spawn headless bot clients
//!   validate-arena  Validate arena file

// Include shadow-rs generated build info
mod shadow {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::{error, info};

use plix_common::build_info::BuildInfo;
use plix_tools::bot::BotSpawner;

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

/// Plix development and testing tools
#[derive(Parser, Debug)]
#[command(name = "plix-tools")]
#[command(about = "Development and testing tools for Plix")]
#[command(version = shadow::CLAP_LONG_VERSION)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", global = true)]
    log_level: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Spawn headless bot clients for load testing
    Bot {
        /// Server address (e.g., 127.0.0.1:7777)
        #[arg(long)]
        server: String,

        /// Number of bots to spawn
        #[arg(long, default_value = "8")]
        count: u8,

        /// Duration in seconds
        #[arg(long, default_value = "60")]
        duration: u32,
    },

    /// Validate arena file
    ValidateArena {
        /// Arena file path or name (from assets/arenas/)
        arena: String,

        /// Assets directory
        #[arg(long, default_value = "assets")]
        assets_dir: PathBuf,
    },
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

    // Log version info on startup
    let build_info = get_build_info();
    info!(
        version = %build_info.version,
        commit = %build_info.commit_sha_short,
        target = %build_info.target_triple,
        "Starting plix-tools {}",
        build_info.display_version()
    );

    match args.command {
        Commands::Bot {
            server,
            count,
            duration,
        } => {
            let addr: SocketAddr = match server.parse() {
                Ok(a) => a,
                Err(e) => {
                    error!(error = %e, "Invalid server address");
                    return;
                }
            };

            let spawner = BotSpawner::new(addr, count as usize, duration);
            let result = spawner.run().await;

            info!(
                bots = result.bots,
                duration = ?result.duration,
                packets_sent = result.packets_sent,
                packets_recv = result.packets_recv,
                pps_sent = format!("{:.1}", result.pps_sent()),
                pps_recv = format!("{:.1}", result.pps_recv()),
                "Load test results"
            );
        }
        Commands::ValidateArena { arena, assets_dir } => {
            let path = if arena.ends_with(".toml") {
                PathBuf::from(&arena)
            } else {
                assets_dir.join("arenas").join(format!("{}.toml", arena))
            };

            info!(path = %path.display(), "Validating arena");

            match plix_arena::load_arena(&path) {
                Ok(loaded) => {
                    info!(
                        name = %loaded.definition.metadata.name,
                        size = ?loaded.size(),
                        spawns = loaded.spawn_point_count(),
                        "Arena valid"
                    );

                    // Run validation
                    if let Err(errors) = plix_arena::validate_arena(&loaded.definition) {
                        for err in errors {
                            error!(error = %err, "Validation error");
                        }
                    } else {
                        info!("All validation checks passed");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to load arena");
                }
            }
        }
    }
}
