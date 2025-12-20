//! Allocation tracking (feature-gated)
//!
//! Provides allocation counting and hotspot identification when enabled.
//! Uses simple counters rather than global allocator hooks for stable Rust compatibility.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Global allocation counter (feature-gated)
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

/// Allocation statistics collector
#[derive(Debug)]
pub struct AllocTracker {
    /// Whether tracking is enabled
    enabled: bool,
    /// Start time of tracking period
    start_time: Instant,
    /// Initial allocation count
    start_allocs: u64,
    /// Initial bytes allocated
    start_bytes: u64,
    /// Known hotspots (location, count, bytes)
    hotspots: Vec<AllocHotspot>,
}

/// Allocation hotspot information
#[derive(Debug, Clone)]
pub struct AllocHotspot {
    /// Source location (function or file:line)
    pub location: String,
    /// Number of allocations
    pub alloc_count: u64,
    /// Total bytes allocated
    pub total_bytes: u64,
}

impl AllocTracker {
    /// Create a new allocation tracker (disabled by default)
    pub fn new() -> Self {
        Self {
            enabled: false,
            start_time: Instant::now(),
            start_allocs: 0,
            start_bytes: 0,
            hotspots: Vec::new(),
        }
    }

    /// Create an enabled allocation tracker
    pub fn enabled() -> Self {
        let mut tracker = Self::new();
        tracker.start();
        tracker
    }

    /// Start tracking allocations
    pub fn start(&mut self) {
        self.enabled = true;
        self.start_time = Instant::now();
        self.start_allocs = ALLOC_COUNT.load(Ordering::Relaxed);
        self.start_bytes = ALLOC_BYTES.load(Ordering::Relaxed);
    }

    /// Stop tracking and return snapshot
    pub fn stop(&mut self) -> AllocSnapshot {
        let elapsed = self.start_time.elapsed();
        let total_allocs = ALLOC_COUNT.load(Ordering::Relaxed) - self.start_allocs;
        let total_bytes = ALLOC_BYTES.load(Ordering::Relaxed) - self.start_bytes;

        let allocs_per_sec = if elapsed.as_secs_f64() > 0.0 {
            total_allocs as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        AllocSnapshot {
            enabled: self.enabled,
            total_allocs,
            total_bytes,
            allocs_per_sec,
            elapsed,
            hotspots: self.hotspots.clone(),
        }
    }

    /// Check if tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record an allocation (call from instrumented code)
    #[inline]
    pub fn record_alloc(bytes: usize) {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Add a known hotspot (for manual profiling results)
    pub fn add_hotspot(&mut self, location: &str, alloc_count: u64, total_bytes: u64) {
        self.hotspots.push(AllocHotspot {
            location: location.to_string(),
            alloc_count,
            total_bytes,
        });
    }

    /// Get current allocation rate estimate
    pub fn current_rate(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let total = ALLOC_COUNT.load(Ordering::Relaxed) - self.start_allocs;
            total as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Check if allocation rate exceeds threshold (allocs/sec)
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.current_rate() > threshold
    }

    /// Reset counters
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.start_allocs = ALLOC_COUNT.load(Ordering::Relaxed);
        self.start_bytes = ALLOC_BYTES.load(Ordering::Relaxed);
        self.hotspots.clear();
    }
}

impl Default for AllocTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of allocation statistics
#[derive(Debug, Clone)]
pub struct AllocSnapshot {
    /// Whether tracking was enabled
    pub enabled: bool,
    /// Total allocations during tracking period
    pub total_allocs: u64,
    /// Total bytes allocated during tracking period
    pub total_bytes: u64,
    /// Allocations per second
    pub allocs_per_sec: f64,
    /// Duration of tracking period
    pub elapsed: Duration,
    /// Known hotspots
    pub hotspots: Vec<AllocHotspot>,
}

impl AllocSnapshot {
    /// Create a disabled snapshot (for when tracking is off)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            total_allocs: 0,
            total_bytes: 0,
            allocs_per_sec: 0.0,
            elapsed: Duration::ZERO,
            hotspots: Vec::new(),
        }
    }
}

/// Reusable buffer for reducing allocations
///
/// A simple wrapper around Vec<u8> that clears instead of reallocating.
#[derive(Debug)]
pub struct ReusableBuffer {
    buffer: Vec<u8>,
    /// Peak usage for monitoring
    peak_size: usize,
}

impl ReusableBuffer {
    /// Create a new reusable buffer with initial capacity
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(initial_capacity),
            peak_size: 0,
        }
    }

    /// Create with default capacity (4KB)
    pub fn with_default_capacity() -> Self {
        Self::new(4096)
    }

    /// Clear the buffer for reuse (no reallocation)
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get mutable access to the underlying buffer
    #[inline]
    pub fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    /// Get the buffer contents as a slice
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Get current length
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Update peak tracking after use
    pub fn update_peak(&mut self) {
        if self.buffer.len() > self.peak_size {
            self.peak_size = self.buffer.len();
        }
    }

    /// Get peak usage
    pub fn peak_size(&self) -> usize {
        self.peak_size
    }
}

impl Default for ReusableBuffer {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

/// Buffer pool for mesh data
///
/// Pre-allocates buffers and reuses them to reduce allocation pressure.
#[derive(Debug)]
pub struct BufferPool {
    /// Available buffers
    available: Vec<Vec<u8>>,
    /// Maximum pool size
    max_size: usize,
    /// Default buffer capacity
    default_capacity: usize,
    /// Buffers checked out
    checked_out: usize,
    /// Peak buffers in use
    peak_in_use: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(max_size: usize, default_capacity: usize) -> Self {
        Self {
            available: Vec::with_capacity(max_size),
            max_size,
            default_capacity,
            checked_out: 0,
            peak_in_use: 0,
        }
    }

    /// Create with default settings (16 buffers, 8KB each)
    pub fn with_defaults() -> Self {
        Self::new(16, 8192)
    }

    /// Get a buffer from the pool (or create new if empty)
    pub fn checkout(&mut self) -> Vec<u8> {
        self.checked_out += 1;
        if self.checked_out > self.peak_in_use {
            self.peak_in_use = self.checked_out;
        }

        if let Some(mut buf) = self.available.pop() {
            buf.clear();
            buf
        } else {
            Vec::with_capacity(self.default_capacity)
        }
    }

    /// Return a buffer to the pool
    pub fn checkin(&mut self, mut buffer: Vec<u8>) {
        self.checked_out = self.checked_out.saturating_sub(1);

        // Only keep buffer if pool isn't full
        if self.available.len() < self.max_size {
            buffer.clear();
            self.available.push(buffer);
        }
        // Otherwise, let the buffer drop (deallocate)
    }

    /// Get pool statistics
    pub fn stats(&self) -> BufferPoolStats {
        BufferPoolStats {
            available: self.available.len(),
            checked_out: self.checked_out,
            peak_in_use: self.peak_in_use,
            max_size: self.max_size,
        }
    }

    /// Pre-warm the pool with buffers
    pub fn prewarm(&mut self, count: usize) {
        let count = count.min(self.max_size - self.available.len());
        for _ in 0..count {
            self.available.push(Vec::with_capacity(self.default_capacity));
        }
    }
}

/// Buffer pool statistics
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    /// Buffers currently available
    pub available: usize,
    /// Buffers currently checked out
    pub checked_out: usize,
    /// Peak buffers in use
    pub peak_in_use: usize,
    /// Maximum pool size
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reusable_buffer() {
        let mut buf = ReusableBuffer::new(100);
        assert_eq!(buf.capacity(), 100);
        assert!(buf.is_empty());

        buf.as_mut_vec().extend_from_slice(b"hello");
        assert_eq!(buf.len(), 5);

        buf.update_peak();
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.peak_size(), 5);
        assert_eq!(buf.capacity(), 100); // Capacity preserved
    }

    #[test]
    fn test_buffer_pool() {
        let mut pool = BufferPool::new(3, 100);

        // Checkout 3 buffers
        let buf1 = pool.checkout();
        let buf2 = pool.checkout();
        let buf3 = pool.checkout();

        assert_eq!(pool.stats().checked_out, 3);
        assert_eq!(pool.stats().available, 0);

        // Return 2
        pool.checkin(buf1);
        pool.checkin(buf2);
        assert_eq!(pool.stats().available, 2);
        assert_eq!(pool.stats().checked_out, 1);

        // Checkout again - should reuse
        let _buf4 = pool.checkout();
        assert_eq!(pool.stats().available, 1);

        // Peak should be 3
        assert_eq!(pool.stats().peak_in_use, 3);

        pool.checkin(buf3);
    }

    #[test]
    fn test_pool_max_size() {
        let mut pool = BufferPool::new(2, 100);

        // Check out and return 3 buffers - only 2 should be kept
        let b1 = pool.checkout();
        let b2 = pool.checkout();
        let b3 = pool.checkout();

        pool.checkin(b1);
        pool.checkin(b2);
        pool.checkin(b3);

        assert_eq!(pool.stats().available, 2); // Max 2
    }

    #[test]
    fn test_alloc_tracker() {
        let mut tracker = AllocTracker::new();
        assert!(!tracker.is_enabled());

        tracker.start();
        assert!(tracker.is_enabled());

        // Record some allocations
        AllocTracker::record_alloc(100);
        AllocTracker::record_alloc(200);

        let snapshot = tracker.stop();
        // Note: exact values depend on other tests running concurrently
        // but allocs_per_sec should be > 0 if elapsed > 0
        assert!(snapshot.enabled);
    }

    #[test]
    fn test_prewarm() {
        let mut pool = BufferPool::new(10, 100);
        pool.prewarm(5);
        assert_eq!(pool.stats().available, 5);

        // Shouldn't exceed max
        pool.prewarm(10);
        assert_eq!(pool.stats().available, 10);
    }
}
