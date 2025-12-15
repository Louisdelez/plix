//! Network debug overlay
//!
//! F3-toggled overlay showing network stats, refreshed at 2Hz.

use std::time::{Duration, Instant};

/// Refresh interval for overlay (2Hz = 500ms)
pub const OVERLAY_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

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
    /// Frames per second
    pub fps: f32,
    /// Local player ID
    pub player_id: u16,
}

/// Network debug overlay with 2Hz refresh caching
#[derive(Debug)]
pub struct NetDebugOverlay {
    /// Whether overlay is visible
    pub visible: bool,
    /// Current debug data
    pub data: NetDebugData,
    /// Last time the display was updated
    last_update: Instant,
    /// Cached formatted lines for display
    cached_lines: Vec<String>,
}

impl Default for NetDebugOverlay {
    fn default() -> Self {
        Self {
            visible: false,
            data: NetDebugData::default(),
            last_update: Instant::now(),
            cached_lines: Vec::new(),
        }
    }
}

impl NetDebugOverlay {
    /// Create a new overlay
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            // Force refresh when becoming visible
            self.last_update = Instant::now()
                .checked_sub(OVERLAY_REFRESH_INTERVAL)
                .unwrap_or_else(Instant::now);
        }
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Update debug data
    pub fn update(&mut self, data: NetDebugData) {
        self.data = data;
    }

    /// Check if it's time to refresh the display (2Hz)
    pub fn should_update(&self) -> bool {
        self.last_update.elapsed() >= OVERLAY_REFRESH_INTERVAL
    }

    /// Build cached display lines from current data
    ///
    /// Call this when `should_update()` returns true.
    pub fn build_cached_lines(&mut self) {
        self.last_update = Instant::now();
        self.cached_lines.clear();

        let d = &self.data;

        // Line 1: Player and FPS
        self.cached_lines
            .push(format!("Player {} | FPS: {:.0}", d.player_id, d.fps));

        // Line 2: Network stats
        self.cached_lines.push(format!(
            "RTT: {:.0}ms | Jitter: {:.0}ms | Loss: {:.1}%",
            d.rtt.as_secs_f64() * 1000.0,
            d.jitter.as_secs_f64() * 1000.0,
            d.packet_loss
        ));

        // Line 3: Tick info
        let offset_sign = if d.tick_offset >= 0 { "+" } else { "" };
        self.cached_lines.push(format!(
            "Tick S:{} C:{} ({}{})",
            d.server_tick, d.client_tick, offset_sign, d.tick_offset
        ));

        // Line 4: Bandwidth and pending inputs
        self.cached_lines.push(format!(
            "Pending: {} | {:.1}KB/s up, {:.1}KB/s down",
            d.pending_inputs,
            d.bytes_sent_per_sec as f64 / 1024.0,
            d.bytes_recv_per_sec as f64 / 1024.0,
        ));
    }

    /// Get the cached display lines
    pub fn lines(&self) -> &[String] {
        &self.cached_lines
    }

    /// Render overlay
    ///
    /// This returns the lines to render; actual GPU rendering is done by the HUD system.
    pub fn render_lines(&mut self) -> Option<&[String]> {
        if !self.visible {
            return None;
        }

        // Update cache if needed
        if self.should_update() {
            self.build_cached_lines();
        }

        Some(&self.cached_lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle() {
        let mut overlay = NetDebugOverlay::new();
        assert!(!overlay.is_visible());

        overlay.toggle();
        assert!(overlay.is_visible());

        overlay.toggle();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_build_cached_lines() {
        let mut overlay = NetDebugOverlay::new();
        overlay.data = NetDebugData {
            rtt: Duration::from_millis(50),
            jitter: Duration::from_millis(5),
            packet_loss: 0.5,
            fps: 60.0,
            player_id: 42,
            server_tick: 1000,
            client_tick: 1003,
            tick_offset: 3,
            ..Default::default()
        };

        overlay.build_cached_lines();
        assert_eq!(overlay.lines().len(), 4);
        assert!(overlay.lines()[0].contains("Player 42"));
        assert!(overlay.lines()[1].contains("50ms"));
    }
}
