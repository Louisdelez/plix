//! Subtitle queue for displaying audio event captions
//!
//! Implements a max 3-entry queue with drop-oldest behavior.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::AudioEvent;

/// Maximum number of simultaneous subtitles
pub const MAX_SUBTITLES: usize = 3;

/// A single subtitle entry
#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    /// Unique ID for this subtitle instance
    pub id: u64,

    /// The audio event that triggered this subtitle
    pub event: AudioEvent,

    /// Display text
    pub text: String,

    /// When this subtitle was created
    pub created_at: Instant,

    /// How long to display (from config)
    pub duration: Duration,
}

impl SubtitleEntry {
    /// Create a new subtitle entry
    pub fn new(id: u64, event: AudioEvent, duration_ms: u32) -> Self {
        Self {
            id,
            event,
            text: event.subtitle_text().to_string(),
            created_at: Instant::now(),
            duration: Duration::from_millis(duration_ms as u64),
        }
    }

    /// Create a subtitle with custom text
    pub fn with_text(id: u64, event: AudioEvent, text: String, duration_ms: u32) -> Self {
        Self {
            id,
            event,
            text,
            created_at: Instant::now(),
            duration: Duration::from_millis(duration_ms as u64),
        }
    }

    /// Check if this subtitle has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    /// Get remaining time before expiry
    pub fn remaining_time(&self) -> Duration {
        let elapsed = self.created_at.elapsed();
        if elapsed >= self.duration {
            Duration::ZERO
        } else {
            self.duration - elapsed
        }
    }
}

/// Queue of active subtitles (max 3, drop oldest when full)
#[derive(Debug)]
pub struct SubtitleQueue {
    /// The subtitle entries (front = oldest, back = newest)
    entries: VecDeque<SubtitleEntry>,

    /// Counter for generating unique IDs
    next_id: u64,
}

impl Default for SubtitleQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl SubtitleQueue {
    /// Create a new empty queue
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_SUBTITLES),
            next_id: 1,
        }
    }

    /// Push a new subtitle, dropping oldest if at capacity
    pub fn push(&mut self, event: AudioEvent, duration_ms: u32) -> &SubtitleEntry {
        // Drop oldest if at capacity
        while self.entries.len() >= MAX_SUBTITLES {
            self.entries.pop_front();
        }

        let id = self.next_id;
        self.next_id += 1;

        let entry = SubtitleEntry::new(id, event, duration_ms);
        self.entries.push_back(entry);

        self.entries.back().unwrap()
    }

    /// Push a subtitle with custom text
    pub fn push_with_text(
        &mut self,
        event: AudioEvent,
        text: String,
        duration_ms: u32,
    ) -> &SubtitleEntry {
        // Drop oldest if at capacity
        while self.entries.len() >= MAX_SUBTITLES {
            self.entries.pop_front();
        }

        let id = self.next_id;
        self.next_id += 1;

        let entry = SubtitleEntry::with_text(id, event, text, duration_ms);
        self.entries.push_back(entry);

        self.entries.back().unwrap()
    }

    /// Process expiry - remove expired subtitles and return their IDs
    pub fn tick(&mut self) -> Vec<u64> {
        let mut expired_ids = Vec::new();

        // Collect expired entries
        self.entries.retain(|entry| {
            if entry.is_expired() {
                expired_ids.push(entry.id);
                false
            } else {
                true
            }
        });

        expired_ids
    }

    /// Get all active entries
    pub fn entries(&self) -> impl Iterator<Item = &SubtitleEntry> {
        self.entries.iter()
    }

    /// Get the number of active subtitles
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all subtitles
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Remove a specific subtitle by ID
    pub fn remove(&mut self, id: u64) -> bool {
        let initial_len = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < initial_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_subtitle_entry_new() {
        let entry = SubtitleEntry::new(1, AudioEvent::ChatMessage, 3000);
        assert_eq!(entry.id, 1);
        assert_eq!(entry.event, AudioEvent::ChatMessage);
        assert_eq!(entry.text, "[Chat]");
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_subtitle_entry_with_text() {
        let entry = SubtitleEntry::with_text(1, AudioEvent::ChatMessage, "Custom text".to_string(), 3000);
        assert_eq!(entry.text, "Custom text");
    }

    #[test]
    fn test_subtitle_queue_new() {
        let queue = SubtitleQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_subtitle_queue_push() {
        let mut queue = SubtitleQueue::new();
        let entry = queue.push(AudioEvent::ChatMessage, 3000);
        assert_eq!(entry.id, 1);
        assert_eq!(queue.len(), 1);

        let entry = queue.push(AudioEvent::PlayerJoin, 3000);
        assert_eq!(entry.id, 2);
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_subtitle_queue_max_capacity() {
        let mut queue = SubtitleQueue::new();

        // Add 3 entries (max)
        queue.push(AudioEvent::ChatMessage, 3000);
        queue.push(AudioEvent::PlayerJoin, 3000);
        queue.push(AudioEvent::PlayerLeave, 3000);
        assert_eq!(queue.len(), MAX_SUBTITLES);

        // Collect IDs before adding 4th
        let ids_before: Vec<_> = queue.entries().map(|e| e.id).collect();
        assert_eq!(ids_before, vec![1, 2, 3]);

        // Add 4th - should drop oldest (id=1)
        queue.push(AudioEvent::PlayerHit, 3000);
        assert_eq!(queue.len(), MAX_SUBTITLES);

        let ids_after: Vec<_> = queue.entries().map(|e| e.id).collect();
        assert_eq!(ids_after, vec![2, 3, 4]); // 1 was dropped

        // Add 5th - should drop next oldest (id=2)
        queue.push(AudioEvent::BlockPlace, 3000);
        let ids_final: Vec<_> = queue.entries().map(|e| e.id).collect();
        assert_eq!(ids_final, vec![3, 4, 5]); // 2 was dropped
    }

    #[test]
    fn test_subtitle_queue_clear() {
        let mut queue = SubtitleQueue::new();
        queue.push(AudioEvent::ChatMessage, 3000);
        queue.push(AudioEvent::PlayerJoin, 3000);
        assert_eq!(queue.len(), 2);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_subtitle_queue_remove() {
        let mut queue = SubtitleQueue::new();
        queue.push(AudioEvent::ChatMessage, 3000); // id=1
        queue.push(AudioEvent::PlayerJoin, 3000); // id=2
        queue.push(AudioEvent::PlayerLeave, 3000); // id=3

        // Remove middle entry
        assert!(queue.remove(2));
        assert_eq!(queue.len(), 2);

        let ids: Vec<_> = queue.entries().map(|e| e.id).collect();
        assert_eq!(ids, vec![1, 3]);

        // Remove non-existent
        assert!(!queue.remove(999));
    }

    #[test]
    fn test_subtitle_queue_tick_expiry() {
        let mut queue = SubtitleQueue::new();

        // Add entry with very short duration (1ms)
        let entry = SubtitleEntry {
            id: 1,
            event: AudioEvent::ChatMessage,
            text: "[Chat]".to_string(),
            created_at: Instant::now() - Duration::from_millis(100), // Already 100ms old
            duration: Duration::from_millis(50), // Only 50ms duration
        };
        queue.entries.push_back(entry);

        // Should be expired
        let expired = queue.tick();
        assert_eq!(expired, vec![1]);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_subtitle_entry_remaining_time() {
        let entry = SubtitleEntry::new(1, AudioEvent::ChatMessage, 3000);
        let remaining = entry.remaining_time();
        assert!(remaining <= Duration::from_millis(3000));
        assert!(remaining > Duration::from_millis(2900)); // Should be close to 3s
    }
}
