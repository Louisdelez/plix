//! Network debug overlay

use std::time::Duration;

/// Network debug info
#[derive(Debug, Default, Clone)]
pub struct NetDebugData {
    /// Round-trip time
    pub rtt: Duration,
    /// Jitter
    pub jitter: Duration,
    /// Packet loss percentage
    pub packet_loss: f32,
    /// Pending inputs count
    pub pending_inputs: usize,
    /// Bytes sent per second
    pub bytes_sent_per_sec: u64,
    /// Bytes received per second
    pub bytes_recv_per_sec: u64,
    /// Server tick
    pub server_tick: u32,
    /// Client tick
    pub client_tick: u32,
    /// Tick offset
    pub tick_offset: i32,
}

/// Network debug overlay
#[derive(Debug, Default)]
pub struct NetDebugOverlay {
    /// Whether overlay is visible
    pub visible: bool,
    /// Current debug data
    pub data: NetDebugData,
}

impl NetDebugOverlay {
    /// Create a new overlay
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Update debug data
    pub fn update(&mut self, data: NetDebugData) {
        self.data = data;
    }

    /// Render overlay (placeholder)
    pub fn render(&self) {
        if !self.visible {
            return;
        }
        // TODO: Implement overlay rendering
        // Would display:
        // RTT: 45ms | Jitter: 5ms | Loss: 0.1%
        // Pending: 3 | Bytes: 1.2KB/s ↑ 15KB/s ↓
        // Tick: S:12345 C:12348 (Δ+3)
    }
}
