//! Plix Mod CLI - Tools for mod development
//!
//! Commands:
//! - `plix-mod new <name>` - Create a new mod from template
//! - `plix-mod build` - Compile mod to WASM
//! - `plix-mod pack` - Create .plixmod bundle
//! - `plix-mod validate <bundle>` - Validate a mod bundle
//! - `plix-mod install <bundle>` - Install to local cache

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cmd_build;
mod cmd_install;
mod cmd_new;
mod cmd_pack;
mod cmd_validate;

#[derive(Parser)]
#[command(name = "plix-mod")]
#[command(author, version, about = "Plix mod development tools", long_about = None)]
struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress output
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new mod project from a template
    New {
        /// Mod project name (becomes mod ID)
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "chat-filter")]
        template: String,

        /// Output directory (defaults to ./<name>)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Compile the mod to WASM
    Build {
        /// Build in release mode (default: true)
        #[arg(short, long, default_value = "true")]
        release: bool,

        /// Path to Cargo.toml
        #[arg(long)]
        manifest_path: Option<PathBuf>,
    },

    /// Create a .plixmod bundle from a built mod
    Pack {
        /// Output file path (default: ./<id>-<version>.plixmod)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to mod.toml manifest
        #[arg(short, long, default_value = "mod.toml")]
        manifest: PathBuf,

        /// Path to WASM binary (auto-detect if not specified)
        #[arg(short, long)]
        wasm: Option<PathBuf>,

        /// Skip signing (dev only)
        #[arg(long)]
        unsigned: bool,
    },

    /// Validate a mod bundle
    Validate {
        /// Path to .plixmod bundle
        bundle: PathBuf,

        /// Fail on warnings
        #[arg(short, long)]
        strict: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Install a mod bundle to local cache
    Install {
        /// Path to .plixmod bundle
        bundle: PathBuf,

        /// Install to local cache (default)
        #[arg(short, long, default_value = "true")]
        local: bool,

        /// Override cache directory
        #[arg(short, long)]
        cache: Option<PathBuf>,

        /// Overwrite existing installation
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging based on verbosity
    let log_level = if cli.quiet {
        tracing::Level::ERROR
    } else {
        match cli.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    match cli.command {
        Commands::New {
            name,
            template,
            output,
        } => cmd_new::run(&name, &template, output.as_deref()),

        Commands::Build {
            release,
            manifest_path,
        } => cmd_build::run(release, manifest_path.as_deref()),

        Commands::Pack {
            output,
            manifest,
            wasm,
            unsigned,
        } => cmd_pack::run(output.as_deref(), &manifest, wasm.as_deref(), unsigned),

        Commands::Validate {
            bundle,
            strict,
            json,
        } => cmd_validate::run(&bundle, strict, json),

        Commands::Install {
            bundle,
            local: _,
            cache,
            force,
        } => cmd_install::run(&bundle, cache.as_deref(), force),
    }
}
