//! Block physics event queue
//!
//! Bounded FIFO queue with deduplication for physics events.

use std::collections::{HashSet, VecDeque};

use crate::types::BlockPos;

use super::event::BlockPhysicsEvent;

/// Bounded FIFO queue for pending physics events with deduplication.
///
/// Uses a VecDeque for FIFO ordering and a HashSet for O(1) duplicate detection.
/// Two events are considered duplicates if they have the same (pos, kind_discriminant).
#[derive(Debug, Clone)]
pub struct BlockPhysicsQueue {
    /// Ordered event queue (FIFO)
    events: VecDeque<BlockPhysicsEvent>,
    /// Deduplication set: (BlockPos, kind discriminant)
    pending: HashSet<(BlockPos, u8)>,
}

impl Default for BlockPhysicsQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockPhysicsQueue {
    /// Create an empty queue
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            pending: HashSet::new(),
        }
    }

    /// Add event to queue if not duplicate.
    /// Returns true if added, false if duplicate.
    pub fn push(&mut self, event: BlockPhysicsEvent) -> bool {
        let key = event.dedup_key();
        if self.pending.contains(&key) {
            return false;
        }
        self.pending.insert(key);
        self.events.push_back(event);
        true
    }

    /// Remove and return front event, or None if empty
    pub fn pop(&mut self) -> Option<BlockPhysicsEvent> {
        if let Some(event) = self.events.pop_front() {
            self.pending.remove(&event.dedup_key());
            Some(event)
        } else {
            None
        }
    }

    /// Current number of pending events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Remove all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_queue_empty() {
        let queue = BlockPhysicsQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_push_pop_fifo() {
        let mut queue = BlockPhysicsQueue::new();

        let event1 = BlockPhysicsEvent::fall(BlockPos::new(0, 0, 0));
        let event2 = BlockPhysicsEvent::fall(BlockPos::new(1, 0, 0));
        let event3 = BlockPhysicsEvent::fall(BlockPos::new(2, 0, 0));

        assert!(queue.push(event1));
        assert!(queue.push(event2));
        assert!(queue.push(event3));

        assert_eq!(queue.len(), 3);

        // FIFO: first in, first out
        assert_eq!(queue.pop(), Some(event1));
        assert_eq!(queue.pop(), Some(event2));
        assert_eq!(queue.pop(), Some(event3));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_duplicate_rejection() {
        let mut queue = BlockPhysicsQueue::new();

        let pos = BlockPos::new(5, 10, 15);
        let event = BlockPhysicsEvent::fall(pos);

        assert!(queue.push(event));
        assert!(!queue.push(event)); // Duplicate - rejected
        assert_eq!(queue.len(), 1);

        // After popping, we can add again
        queue.pop();
        assert!(queue.push(event));
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_different_kinds_not_duplicate() {
        let mut queue = BlockPhysicsQueue::new();

        let pos = BlockPos::new(5, 10, 15);
        let fall = BlockPhysicsEvent::fall(pos);
        let liquid = BlockPhysicsEvent::liquid_spread(pos, 0);

        // Same position but different kind = not duplicate
        assert!(queue.push(fall));
        assert!(queue.push(liquid));
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut queue = BlockPhysicsQueue::new();

        queue.push(BlockPhysicsEvent::fall(BlockPos::new(0, 0, 0)));
        queue.push(BlockPhysicsEvent::fall(BlockPos::new(1, 0, 0)));
        assert_eq!(queue.len(), 2);

        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        // Can add same events again after clear
        assert!(queue.push(BlockPhysicsEvent::fall(BlockPos::new(0, 0, 0))));
    }

    #[test]
    fn test_invariant_sync() {
        let mut queue = BlockPhysicsQueue::new();

        // Add some events
        for i in 0..10 {
            queue.push(BlockPhysicsEvent::fall(BlockPos::new(i, 0, 0)));
        }

        // Invariant: events.len() == pending.len()
        assert_eq!(queue.events.len(), queue.pending.len());

        // Pop some
        for _ in 0..5 {
            queue.pop();
        }

        // Still in sync
        assert_eq!(queue.events.len(), queue.pending.len());
        assert_eq!(queue.len(), 5);
    }
}
