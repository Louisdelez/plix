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
//!   --validate-content     Validate content files and exit
//!   --content-strict       Use strict validation (fail on first error)
//!   --migrate              Run data migrations before starting
//!   --dry-run              With --migrate, show what would be migrated without changes
//!   --restore-backup       Restore from a backup file

// Include shadow-rs generated build info
mod shadow {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use tracing::{error, info, warn};

use plix_common::build_info::BuildInfo;
use plix_common::migration::{
    config::{detect_config_version, V0ToV1ConfigMigration},
    MigrationEngine, MigrationVersion, restore_backup, MAX_BACKUPS,
};
use plix_server::content::{ContentLoader, ContentValidator, ValidationMode};
use plix_server::exit_codes::ExitCode as PlixExitCode;
use plix_server::{master_announce::MasterConfig, Server, ServerConfig};

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

/// Plix authoritative game server
///
/// Configuration priority: CLI flags > Environment variables > Defaults
/// Environment variables are prefixed with PLIX_ (e.g., PLIX_PORT, PLIX_SERVER_NAME)
#[derive(Parser, Debug)]
#[command(name = "plix-server")]
#[command(about = "Authoritative multiplayer game server for Plix")]
#[command(version = shadow::CLAP_LONG_VERSION)]
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

    /// Validate content files and exit (use with --content-strict for fail-fast mode)
    #[arg(long)]
    validate_content: bool,

    /// Use strict validation mode (fail on first error, exit code 1)
    /// Without this flag, validation logs warnings but continues
    #[arg(long)]
    content_strict: bool,

    /// Run data migrations before starting the server.
    /// Creates backups before migrating and rotates old backups (keeps 3).
    #[arg(long)]
    migrate: bool,

    /// Dry-run mode: show what migrations would be applied without making changes.
    /// Must be used with --migrate.
    #[arg(long, requires = "migrate")]
    dry_run: bool,

    /// Restore configuration from a backup file and exit.
    /// The backup file will be copied to the config location.
    #[arg(long, value_name = "BACKUP_FILE")]
    restore_backup: Option<PathBuf>,

    /// Config directory path (for migration and config loading)
    #[arg(long, env = "PLIX_CONFIG_DIR")]
    config_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> ExitCode {
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
        "Starting plix-server {}",
        build_info.display_version()
    );

    // Determine config directory
    let config_dir = args.config_dir.clone().unwrap_or_else(|| {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("plix")
    });
    let config_path = config_dir.join("server.toml");

    // Handle restore backup mode
    if let Some(backup_path) = &args.restore_backup {
        return handle_restore_backup(backup_path, &config_path);
    }

    // Handle content validation mode
    if args.validate_content {
        return validate_content_and_exit(&args);
    }

    // Handle migration check/execution
    match handle_migrations(&args, &config_path, &config_dir) {
        Ok(()) => {}
        Err(code) => return code,
    }

    info!(
        port = args.port,
        tickrate = args.tickrate,
        max_players = args.max_players,
        arena = %args.arena,
        "Server configuration"
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
                return ExitCode::FAILURE;
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to start server");
            return ExitCode::FAILURE;
        }
    }

    info!("Server shutdown");
    ExitCode::SUCCESS
}

/// Validate content files and exit with appropriate code
fn validate_content_and_exit(args: &Args) -> ExitCode {
    let content_path = args.assets_dir.join("content");

    info!(path = ?content_path, "Validating content files");

    // Load content
    let loader = ContentLoader::new(&content_path);
    let registry = match loader.load_all() {
        Ok(reg) => {
            let counts = reg.counts();
            info!(
                chapters = counts.chapters,
                quests = counts.quests,
                mobs = counts.mobs,
                loot_tables = counts.loot_tables,
                spawns = counts.spawns,
                dungeons = counts.dungeons,
                npcs = counts.npcs,
                "Content loaded successfully"
            );
            reg
        }
        Err(e) => {
            error!(error = %e, "Failed to load content");
            return ExitCode::FAILURE;
        }
    };

    // Validate content
    let mode = if args.content_strict {
        ValidationMode::Development
    } else {
        ValidationMode::Production
    };

    let validator = ContentValidator::new(mode);
    match validator.validate(&registry) {
        Ok(()) => {
            info!("Content validation passed");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            if args.content_strict {
                for err in &errors {
                    error!(error = %err, "Validation error");
                }
                error!(count = errors.len(), "Content validation failed");
                ExitCode::FAILURE
            } else {
                for err in &errors {
                    warn!(error = %err, "Validation warning");
                }
                warn!(count = errors.len(), "Content validation completed with warnings");
                ExitCode::SUCCESS
            }
        }
    }
}

/// Handle data migrations at server startup.
///
/// Returns Ok(()) if the server can proceed, Err(ExitCode) if it should exit.
fn handle_migrations(
    args: &Args,
    config_path: &PathBuf,
    config_dir: &PathBuf,
) -> Result<(), ExitCode> {
    // Check if config file exists
    if !config_path.exists() {
        // No config file means fresh install, no migration needed
        info!(path = %config_path.display(), "No config file found, will use defaults");
        return Ok(());
    }

    // Read config and detect version
    let content = match std::fs::read(config_path) {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, path = %config_path.display(), "Failed to read config file");
            return Err(ExitCode::FAILURE);
        }
    };

    let current_version = detect_config_version(&content);
    let target_version = MigrationVersion::V1_0_0;

    info!(
        config = %config_path.display(),
        current_version = %current_version,
        target_version = %target_version,
        "Checking configuration version"
    );

    // If already at target version, nothing to do
    if current_version >= target_version {
        info!("Configuration is up to date, no migration needed");
        return Ok(());
    }

    // Migration is needed
    if !args.migrate {
        // Migration required but not requested
        error!(
            current_version = %current_version,
            target_version = %target_version,
            "Data migration required. Your configuration is from an older version."
        );
        error!("To migrate, run: plix-server --migrate");
        error!("To preview changes, run: plix-server --migrate --dry-run");
        error!("Backups are created automatically before migration.");

        // Use our custom exit code
        return Err(std::process::ExitCode::from(PlixExitCode::MigrationRequired.code() as u8));
    }

    // Set up migration engine
    let backup_dir = config_dir.join("backups");
    let mut engine = MigrationEngine::new(&backup_dir);
    engine.register(Box::new(V0ToV1ConfigMigration));

    // Run migration (dry-run or real)
    let result = if args.dry_run {
        info!("Running migration in dry-run mode (no changes will be made)");
        engine.run_migrations_dry_run(config_path, current_version)
    } else {
        info!("Running migration...");
        engine.run_migrations(config_path, current_version)
    };

    match result {
        Ok(report) => {
            if args.dry_run {
                info!(
                    from = %report.from_version,
                    to = %report.to_version,
                    migrations = report.migrations_applied,
                    "Dry-run complete: {} migration(s) would be applied",
                    report.migrations_applied
                );
                info!("Run without --dry-run to apply migrations");
                // Exit after dry-run
                return Err(ExitCode::SUCCESS);
            } else {
                info!(
                    from = %report.from_version,
                    to = %report.to_version,
                    migrations = report.migrations_applied,
                    "Migration complete: {} migration(s) applied",
                    report.migrations_applied
                );
                if let Some(backup) = &report.backup {
                    info!(
                        backup = %backup.path.display(),
                        checksum = %backup.checksum,
                        "Backup created before migration"
                    );
                }
            }
        }
        Err(e) => {
            error!(error = %e, "Migration failed");
            error!("Your original configuration has been preserved.");
            error!("Check the backup directory for recovery: {}", backup_dir.display());
            return Err(std::process::ExitCode::from(PlixExitCode::MigrationFailed.code() as u8));
        }
    }

    Ok(())
}

/// Handle restoring configuration from a backup file.
fn handle_restore_backup(backup_path: &PathBuf, config_path: &PathBuf) -> ExitCode {
    info!(
        backup = %backup_path.display(),
        target = %config_path.display(),
        "Restoring configuration from backup"
    );

    if !backup_path.exists() {
        error!(path = %backup_path.display(), "Backup file not found");
        return ExitCode::FAILURE;
    }

    match restore_backup(backup_path, config_path) {
        Ok(()) => {
            info!("Configuration restored successfully");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!(error = %e, "Failed to restore backup");
            ExitCode::FAILURE
        }
    }
}
