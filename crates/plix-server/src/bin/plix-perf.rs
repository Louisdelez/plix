//! Plix Performance Harness
//!
//! Runs reproducible performance scenarios and generates structured reports.
//!
//! # Usage
//!
//! ```bash
//! cargo run --release --features perf --bin plix-perf -- --scenario idle --duration 60
//! ```

use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use tracing::{info, warn};

use plix_server::perf::{
    config::PerfConfig,
    metrics::{Histogram, SubsystemMetrics},
    reporter::{Metadata, PerfReport, TickStats},
    scenarios::{PerfScenario, ScenarioRunner},
    TickBudget,
};

/// Plix performance profiling harness
#[derive(Parser, Debug)]
#[command(name = "plix-perf")]
#[command(about = "Run reproducible performance scenarios and generate reports")]
#[command(version)]
struct Args {
    /// Scenario to run (idle, world_churn, net_stress)
    #[arg(long, default_value = "idle")]
    scenario: String,

    /// Duration in seconds
    #[arg(long, default_value = "60")]
    duration: u32,

    /// Output report file path
    #[arg(long, default_value = "perf_report.json")]
    output: PathBuf,

    /// Tick rate (TPS)
    #[arg(long, default_value = "60")]
    tick_rate: u8,

    /// Number of simulated players (for net_stress scenario)
    #[arg(long)]
    players: Option<u32>,

    /// P95 threshold for failure (optional, for CI)
    #[arg(long)]
    threshold_p95: Option<f64>,

    /// Print report to console instead of file
    #[arg(long)]
    no_file: bool,

    /// Pretty-print JSON output
    #[arg(long)]
    pretty: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
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

    info!(
        scenario = %args.scenario,
        duration_secs = args.duration,
        tick_rate = args.tick_rate,
        "Starting performance harness"
    );

    // Parse scenario
    let scenario = match PerfScenario::from_str(&args.scenario) {
        Some(s) => s,
        None => {
            eprintln!(
                "Invalid scenario '{}'. Valid options: idle, world_churn, net_stress",
                args.scenario
            );
            std::process::exit(1);
        }
    };

    // Validate config
    let config = PerfConfig::for_scenario(scenario.name(), args.duration)
        .with_tick_rate(args.tick_rate)
        .with_report_path(args.output.clone());

    if let Err(e) = config.validate() {
        eprintln!("Invalid configuration: {}", e);
        std::process::exit(1);
    }

    // Run the scenario
    let report = run_scenario(&config, scenario, args.players);

    // Output report
    if args.no_file {
        let json = if args.pretty {
            serde_json::to_string_pretty(&report).unwrap()
        } else {
            serde_json::to_string(&report).unwrap()
        };
        println!("{}", json);
    } else {
        if let Err(e) = report.write_to_file(&args.output) {
            eprintln!("Failed to write report: {}", e);
            std::process::exit(1);
        }
        info!(path = %args.output.display(), "Report written");
    }

    // Print summary
    print_summary(&report);

    // Check threshold for CI
    if let Some(threshold) = args.threshold_p95 {
        if report.tick_stats.p95_ms > threshold {
            eprintln!(
                "FAIL: p95 ({:.2}ms) exceeds threshold ({:.2}ms)",
                report.tick_stats.p95_ms, threshold
            );
            std::process::exit(1);
        }
        info!(
            p95_ms = format!("{:.2}", report.tick_stats.p95_ms),
            threshold_ms = format!("{:.2}", threshold),
            "PASS: p95 within threshold"
        );
    }
}

/// Run the profiling scenario
fn run_scenario(
    config: &PerfConfig,
    scenario: PerfScenario,
    player_count: Option<u32>,
) -> PerfReport {
    let tick_duration = Duration::from_secs_f64(config.budgets.tick_target_ms / 1000.0);
    let total_duration = Duration::from_secs(config.duration_secs as u64);

    // Initialize collectors
    let mut tick_histogram = Histogram::with_defaults();
    let mut subsystem_metrics = SubsystemMetrics::new();
    let mut budget = TickBudget::new(config.budgets.clone());

    // Create scenario runner
    let mut runner = ScenarioRunner::with_duration(scenario, total_duration);

    info!(
        scenario = scenario.name(),
        duration_secs = config.duration_secs,
        tick_budget_ms = format!("{:.2}", config.budgets.tick_target_ms),
        "Running scenario"
    );

    let start = Instant::now();
    let mut tick_count = 0u64;

    // Main profiling loop
    while !runner.is_complete() && start.elapsed() < total_duration {
        let tick_start = Instant::now();

        // Execute scenario actions
        let actions = runner.tick(tick_duration);

        // Simulate subsystem work based on scenario
        simulate_subsystems(&mut subsystem_metrics, scenario, actions.len());

        // Record tick timing
        let tick_elapsed = tick_start.elapsed();
        tick_histogram.record_duration(tick_elapsed);

        // Check budget
        if budget.record_tick(tick_elapsed) {
            warn!(
                tick = tick_count,
                elapsed_ms = format!("{:.2}", tick_elapsed.as_secs_f64() * 1000.0),
                "Tick overrun"
            );
        }

        tick_count += 1;

        // Sleep to maintain tick rate
        let elapsed = tick_start.elapsed();
        if let Some(sleep_time) = tick_duration.checked_sub(elapsed) {
            std::thread::sleep(sleep_time);
        }

        // Progress logging every 10 seconds
        if tick_count % (config.budgets.tick_target_ms as u64 * 10000 / 1000) == 0 {
            info!(
                progress = format!("{:.0}%", runner.progress() * 100.0),
                ticks = tick_count,
                "Progress"
            );
        }
    }

    let actual_duration = start.elapsed();
    info!(
        ticks = tick_count,
        duration_secs = format!("{:.1}", actual_duration.as_secs_f64()),
        "Scenario complete"
    );

    // Build report
    let metadata = Metadata::new(scenario.name(), config.duration_secs, config.budgets.tick_target_ms as u8 / 60 * 60)
        .with_player_count(player_count.unwrap_or(scenario.default_player_count()));

    let tick_stats = TickStats::from_histogram(&tick_histogram.stats(), Some(budget.overrun_count()));

    let mut report = PerfReport::new(metadata, tick_stats)
        .with_subsystem_stats(subsystem_metrics.all_stats())
        .with_alloc_stats_disabled();

    report
}

/// Simulate subsystem work for timing measurements
fn simulate_subsystems(metrics: &mut SubsystemMetrics, scenario: PerfScenario, action_count: usize) {
    // Simulate different subsystem loads based on scenario
    match scenario {
        PerfScenario::Idle => {
            // Minimal work
            metrics.record("simulation", simulate_work(0.5));
            metrics.record("net_encode", simulate_work(0.1));
        }
        PerfScenario::WorldChurn => {
            // Heavy meshing work
            metrics.record("simulation", simulate_work(1.0));
            metrics.record("meshing", simulate_work(2.0 + action_count as f64 * 0.5));
            metrics.record("streaming", simulate_work(1.5));
        }
        PerfScenario::NetStress => {
            // Heavy network work
            metrics.record("simulation", simulate_work(2.0));
            metrics.record("net_encode", simulate_work(1.5 + action_count as f64 * 0.1));
            metrics.record("net_decode", simulate_work(1.0));
            metrics.record("net_flush", simulate_work(0.5));
        }
    }
}

/// Simulate work and return approximate time spent in ms
fn simulate_work(target_ms: f64) -> f64 {
    let start = Instant::now();
    let target = Duration::from_secs_f64(target_ms / 1000.0);

    // Busy loop to simulate work (not just sleep)
    let mut sum = 0u64;
    while start.elapsed() < target {
        for i in 0..1000 {
            sum = sum.wrapping_add(i);
        }
    }

    // Prevent optimization
    std::hint::black_box(sum);

    start.elapsed().as_secs_f64() * 1000.0
}

/// Print summary to console
fn print_summary(report: &PerfReport) {
    println!("\n=== Performance Report Summary ===");
    println!("Scenario: {}", report.metadata.scenario);
    println!("Duration: {}s", report.metadata.duration_secs);
    println!("Build: {}", report.metadata.build_mode);
    println!();
    println!("Tick Statistics:");
    println!("  Count: {}", report.tick_stats.count);
    println!("  Avg:   {:.2}ms", report.tick_stats.avg_ms);
    if let Some(p50) = report.tick_stats.p50_ms {
        println!("  p50:   {:.2}ms", p50);
    }
    println!("  p95:   {:.2}ms", report.tick_stats.p95_ms);
    println!("  p99:   {:.2}ms", report.tick_stats.p99_ms);
    if let Some(max) = report.tick_stats.max_ms {
        println!("  Max:   {:.2}ms", max);
    }
    if let Some(overruns) = report.tick_stats.overruns {
        println!("  Overruns: {}", overruns);
    }

    if !report.subsystem_stats.is_empty() {
        println!();
        println!("Subsystem Breakdown:");
        for (name, stats) in &report.subsystem_stats {
            println!(
                "  {}: avg={:.2}ms p95={:.2}ms",
                name, stats.avg_ms, stats.p95_ms
            );
        }
    }

    println!("==================================\n");
}
