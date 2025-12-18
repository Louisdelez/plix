//! Plix Launcher - Automatic game updater and launcher

use clap::Parser;
use plix_launcher::config::LauncherConfig;
use plix_launcher::download::file::download_files_to_temp;
use plix_launcher::error::{ExitCode, LauncherError};
use plix_launcher::install::atomic::{cleanup_temp, install_files, update_current_link};
use plix_launcher::install::structure::LauncherDirs;
use plix_launcher::launch::platform::{ensure_executable, launch_game};
use plix_launcher::manifest::{fetch_manifest, validate_manifest};
use plix_launcher::version::{check_for_updates, LocalState, UpdateStatus};
use std::process;
use tracing_subscriber::EnvFilter;

/// Plix game launcher with automatic updates
#[derive(Parser, Debug)]
#[command(name = "plix-launcher")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Check for updates without downloading or launching
    #[arg(short = 'c', long, conflicts_with_all = ["update", "launch"])]
    pub check: bool,

    /// Download and install updates without launching
    #[arg(short = 'u', long, conflicts_with_all = ["check", "launch"])]
    pub update: bool,

    /// Launch game without checking for updates
    #[arg(short = 'l', long, conflicts_with_all = ["check", "update"])]
    pub launch: bool,

    /// Simulate operations without making changes
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Enable verbose logging
    #[arg(short = 'v', long, env = "PLIX_LAUNCHER_VERBOSE")]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(short = 'q', long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Keep launcher open after game exits
    #[arg(long)]
    pub stay_open: bool,

    /// Override manifest URL
    #[arg(long, env = "PLIX_MANIFEST_URL")]
    pub manifest_url: Option<String>,

    /// Use custom config file
    #[arg(long)]
    pub config: Option<std::path::PathBuf>,

    /// Override data directory
    #[arg(long, env = "PLIX_DATA_DIR")]
    pub data_dir: Option<std::path::PathBuf>,

    /// Arguments to pass to the game
    #[arg(last = true)]
    pub game_args: Vec<String>,
}

fn main() {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.verbose {
        EnvFilter::new("debug")
    } else if args.quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    tracing::info!("Plix Launcher v{}", env!("CARGO_PKG_VERSION"));

    match run(args) {
        Ok(()) => process::exit(ExitCode::Success as i32),
        Err(e) => {
            tracing::error!("{}", e);
            process::exit(ExitCode::from(&e) as i32);
        }
    }
}

fn run(args: Args) -> Result<(), LauncherError> {
    // Set up directories
    let dirs = if let Some(data_dir) = &args.data_dir {
        LauncherDirs::with_data_dir(data_dir.clone())?
    } else {
        LauncherDirs::new()?
    };
    dirs.ensure_dirs()?;

    // Load config
    let default_config_path = dirs.config_file();
    let config_path = args
        .config
        .as_ref()
        .map(|p| p.as_path())
        .unwrap_or(&default_config_path);
    let config = LauncherConfig::load_or_create(config_path)?;
    tracing::debug!("Config loaded from {}", config_path.display());

    // Load local state
    let mut local_state = LocalState::load(&dirs.state_file())?;
    tracing::debug!("Local state: {:?}", local_state.installed_version);

    // Determine manifest URL
    let manifest_url = args.manifest_url.as_ref().unwrap_or(&config.manifest_url);

    // Handle --launch mode (skip update check)
    if args.launch {
        return launch_existing(&dirs, &args, config.stay_open || args.stay_open);
    }

    // Fetch manifest
    let manifest_result = fetch_manifest(manifest_url, config.timeout_seconds);

    let update_status = match manifest_result {
        Ok(manifest) => {
            validate_manifest(&manifest)?;
            check_for_updates(&local_state, &manifest)?
        }
        Err(e) => {
            tracing::warn!("Could not fetch manifest: {}", e);
            // Offline mode
            if let Some(version) = &local_state.installed_version {
                UpdateStatus::Offline {
                    local_version: version.clone(),
                }
            } else {
                return Err(LauncherError::CannotProceed(
                    "No internet connection and no local version installed".into(),
                ));
            }
        }
    };

    // Display status
    display_status(&update_status);

    // Handle --check mode
    if args.check {
        return Ok(());
    }

    // Handle update/install
    if update_status.needs_download() {
        if args.dry_run {
            tracing::info!("[DRY-RUN] Would download and install update");
        } else {
            perform_update(&update_status, &dirs, &config, &mut local_state)?;
        }
    }

    // Handle --update mode (don't launch)
    if args.update {
        return Ok(());
    }

    // Launch game
    if !args.dry_run {
        let stay_open = config.stay_open || args.stay_open;
        launch_existing(&dirs, &args, stay_open)?;
    } else {
        tracing::info!("[DRY-RUN] Would launch: {}", dirs.game_binary().display());
    }

    Ok(())
}

fn display_status(status: &UpdateStatus) {
    match status {
        UpdateStatus::UpToDate { version } => {
            tracing::info!("Up to date: v{}", version);
        }
        UpdateStatus::UpdateAvailable {
            current,
            latest,
            files_to_update,
            total_download_size,
        } => {
            tracing::info!(
                "Update available: v{} -> v{} ({} files, {} MB)",
                current,
                latest,
                files_to_update.len(),
                total_download_size / 1_000_000
            );
        }
        UpdateStatus::FreshInstall {
            version,
            files,
            total_download_size,
        } => {
            tracing::info!(
                "Fresh install: v{} ({} files, {} MB)",
                version,
                files.len(),
                total_download_size / 1_000_000
            );
        }
        UpdateStatus::Offline { local_version } => {
            tracing::warn!("Offline mode - using local version v{}", local_version);
        }
        UpdateStatus::CannotProceed { reason } => {
            tracing::error!("Cannot proceed: {}", reason);
        }
    }
}

fn perform_update(
    status: &UpdateStatus,
    dirs: &LauncherDirs,
    config: &LauncherConfig,
    local_state: &mut LocalState,
) -> Result<(), LauncherError> {
    let (version, files) = match status {
        UpdateStatus::UpdateAvailable {
            latest,
            files_to_update,
            ..
        } => (latest.clone(), files_to_update.clone()),
        UpdateStatus::FreshInstall { version, files, .. } => (version.clone(), files.clone()),
        _ => return Ok(()), // Nothing to do
    };

    tracing::info!("Downloading {} files...", files.len());

    // Create temp directory
    let temp_dir = dirs.data_dir.join("temp_download");

    // Download files
    let progress_cb = |index: usize, total: usize, path: &str| {
        tracing::info!("  [{}/{}] Downloading {}", index + 1, total, path);
    };

    if let Err(e) = download_files_to_temp(
        &files,
        &temp_dir,
        config.timeout_seconds,
        config.max_retries,
        Some(&progress_cb),
    ) {
        cleanup_temp(&temp_dir)?;
        return Err(e);
    }

    tracing::info!("Installing version {}...", version);

    // Install to version directory
    let version_dir = dirs.version_dir(&version);
    let checksums = install_files(&temp_dir, &version_dir, &files)?;

    // Update current link
    update_current_link(&dirs.current_dir, &version_dir)?;

    // Clean up temp
    cleanup_temp(&temp_dir)?;

    // Update local state
    local_state.update_after_install(
        version,
        dirs.current_dir.to_string_lossy().to_string(),
        checksums,
    );
    local_state.save(&dirs.state_file())?;

    tracing::info!("Installation complete!");

    Ok(())
}

fn launch_existing(dirs: &LauncherDirs, args: &Args, stay_open: bool) -> Result<(), LauncherError> {
    let binary = dirs.game_binary();

    if !binary.exists() {
        return Err(LauncherError::Launch(format!(
            "Game not installed. Run launcher without --launch to install: {}",
            binary.display()
        )));
    }

    // Ensure executable permission
    ensure_executable(&binary)?;

    // Launch
    launch_game(&binary, &args.game_args, stay_open)
}
