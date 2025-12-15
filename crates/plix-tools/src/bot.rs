//! Headless bot client for load testing

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use plix_common::protocol::{ClientMessage, PlayerInput, ServerMessage, PROTOCOL_VERSION};
use plix_common::time::Tick;
use plix_common::types::{InputSeq, PlayerId};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tokio::net::UdpSocket;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Bot client configuration
#[derive(Debug, Clone)]
pub struct BotConfig {
    /// Server address
    pub server: SocketAddr,
    /// Bot name prefix
    pub name_prefix: String,
    /// Movement randomization
    pub random_movement: bool,
}

/// Headless bot client
#[derive(Debug)]
pub struct BotClient {
    /// Configuration
    config: BotConfig,
    /// Bot index
    index: usize,
    /// Current tick
    tick: Tick,
    /// Input sequence
    input_seq: InputSeq,
    /// Random generator (Send-safe)
    rng: StdRng,
    /// Player ID (after connect)
    player_id: Option<PlayerId>,
}

impl BotClient {
    /// Create a new bot client
    pub fn new(config: BotConfig, index: usize) -> Self {
        Self {
            config,
            index,
            tick: Tick::ZERO,
            input_seq: InputSeq(0),
            rng: StdRng::from_entropy(),
            player_id: None,
        }
    }

    /// Get bot name
    pub fn name(&self) -> String {
        format!("{}_{}", self.config.name_prefix, self.index)
    }

    /// Generate connect message
    pub fn connect_message(&self) -> ClientMessage {
        ClientMessage::Connect {
            protocol_version: PROTOCOL_VERSION,
            name: self.name(),
        }
    }

    /// Generate input for current tick
    pub fn generate_input(&mut self) -> PlayerInput {
        let (forward, right) = if self.config.random_movement {
            (
                self.rng.gen_range(-1.0..=1.0),
                self.rng.gen_range(-1.0..=1.0),
            )
        } else {
            // Simple patrol pattern
            let phase = (self.tick.0 / 60) % 4;
            match phase {
                0 => (1.0, 0.0),  // Forward
                1 => (0.0, 1.0),  // Right
                2 => (-1.0, 0.0), // Back
                _ => (0.0, -1.0), // Left
            }
        };

        let input = PlayerInput {
            seq: self.input_seq,
            tick: self.tick,
            move_forward: forward,
            move_right: right,
            jump: self.rng.gen_bool(0.01), // Occasionally jump
            crouch: false,
            attack: self.rng.gen_bool(0.1), // Occasionally attack
            yaw: self
                .rng
                .gen_range(-std::f32::consts::PI..std::f32::consts::PI),
            pitch: 0.0,
        };

        self.input_seq = self.input_seq.next();
        self.tick = self.tick.next();

        input
    }

    /// Run bot client loop
    pub async fn run(
        mut self,
        running: Arc<AtomicBool>,
        packets_sent: Arc<AtomicU64>,
        packets_recv: Arc<AtomicU64>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Bind socket
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(self.config.server).await?;

        // Send connect
        let connect_msg = self.connect_message();
        let data = plix_common::protocol::encode(&connect_msg)?;
        socket.send(&data).await?;
        packets_sent.fetch_add(1, Ordering::Relaxed);

        // Wait for response
        let mut buf = vec![0u8; 1500];
        let timeout = Duration::from_secs(5);

        let len = tokio::time::timeout(timeout, socket.recv(&mut buf)).await??;
        packets_recv.fetch_add(1, Ordering::Relaxed);

        let response: ServerMessage = plix_common::protocol::decode(&buf[..len])?;
        match response {
            ServerMessage::Connected {
                player_id, tick, ..
            } => {
                self.player_id = Some(player_id);
                self.tick = tick;
                debug!(bot = %self.name(), player_id = player_id.0, "Bot connected");
            }
            ServerMessage::Rejected { reason } => {
                warn!(bot = %self.name(), reason = %reason, "Bot rejected");
                return Ok(());
            }
            _ => {
                warn!(bot = %self.name(), "Unexpected response");
                return Ok(());
            }
        }

        // Main loop - send inputs at 60Hz
        let mut tick_interval = interval(Duration::from_secs_f64(1.0 / 60.0));

        while running.load(Ordering::Relaxed) {
            tick_interval.tick().await;

            // Generate and send input
            let input = self.generate_input();
            let msg = ClientMessage::Input(input);
            let data = plix_common::protocol::encode(&msg)?;

            if let Err(e) = socket.send(&data).await {
                debug!(bot = %self.name(), error = %e, "Send failed");
                break;
            }
            packets_sent.fetch_add(1, Ordering::Relaxed);

            // Try to receive (non-blocking)
            match tokio::time::timeout(Duration::from_millis(1), socket.recv(&mut buf)).await {
                Ok(Ok(_)) => {
                    packets_recv.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
        }

        // Disconnect
        let msg = ClientMessage::Disconnect;
        let data = plix_common::protocol::encode(&msg)?;
        let _ = socket.send(&data).await;

        debug!(bot = %self.name(), "Bot disconnected");
        Ok(())
    }
}

/// Bot spawner for load testing
#[derive(Debug, Clone)]
pub struct BotSpawner {
    /// Server address
    pub server: SocketAddr,
    /// Number of bots
    pub count: usize,
    /// Test duration
    pub duration: Duration,
    /// Bot name prefix
    pub name_prefix: String,
}

impl BotSpawner {
    /// Create a new bot spawner
    pub fn new(server: SocketAddr, count: usize, duration_secs: u32) -> Self {
        Self {
            server,
            count,
            duration: Duration::from_secs(duration_secs as u64),
            name_prefix: "Bot".to_string(),
        }
    }

    /// Run the load test
    pub async fn run(&self) -> LoadTestResult {
        info!(
            server = %self.server,
            count = self.count,
            duration = ?self.duration,
            "Starting load test"
        );

        let running = Arc::new(AtomicBool::new(true));
        let packets_sent = Arc::new(AtomicU64::new(0));
        let packets_recv = Arc::new(AtomicU64::new(0));

        let start = Instant::now();

        // Spawn bots
        let mut handles = Vec::new();
        for i in 0..self.count {
            let config = BotConfig {
                server: self.server,
                name_prefix: self.name_prefix.clone(),
                random_movement: true,
            };
            let bot = BotClient::new(config, i);
            let running = running.clone();
            let sent = packets_sent.clone();
            let recv = packets_recv.clone();

            let handle = tokio::spawn(async move {
                if let Err(e) = bot.run(running, sent, recv).await {
                    error!(error = %e, "Bot error");
                }
            });
            handles.push(handle);

            // Stagger bot spawns
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Run for duration
        tokio::time::sleep(self.duration).await;

        // Stop bots
        running.store(false, Ordering::Relaxed);

        // Wait for bots to finish
        for handle in handles {
            let _ = handle.await;
        }

        let elapsed = start.elapsed();
        let sent = packets_sent.load(Ordering::Relaxed);
        let recv = packets_recv.load(Ordering::Relaxed);

        info!(
            elapsed = ?elapsed,
            packets_sent = sent,
            packets_recv = recv,
            "Load test complete"
        );

        LoadTestResult {
            duration: elapsed,
            bots: self.count,
            packets_sent: sent,
            packets_recv: recv,
        }
    }
}

/// Load test results
#[derive(Debug, Clone)]
pub struct LoadTestResult {
    /// Actual test duration
    pub duration: Duration,
    /// Number of bots
    pub bots: usize,
    /// Total packets sent
    pub packets_sent: u64,
    /// Total packets received
    pub packets_recv: u64,
}

impl LoadTestResult {
    /// Get packets per second sent
    pub fn pps_sent(&self) -> f64 {
        self.packets_sent as f64 / self.duration.as_secs_f64()
    }

    /// Get packets per second received
    pub fn pps_recv(&self) -> f64 {
        self.packets_recv as f64 / self.duration.as_secs_f64()
    }
}
