//! Game event buffering for replication

use std::collections::VecDeque;

use plix_common::protocol::GameEvent;
use plix_common::time::Tick;

/// Maximum events to buffer
const MAX_EVENTS: usize = 256;

/// Event with tick info
#[derive(Debug, Clone)]
pub struct TimedEvent {
    pub tick: Tick,
    pub event: GameEvent,
}

/// Buffer for game events pending replication
#[derive(Debug, Default)]
pub struct EventBuffer {
    events: VecDeque<TimedEvent>,
}

impl EventBuffer {
    /// Create a new event buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an event
    pub fn push(&mut self, tick: Tick, event: GameEvent) {
        if self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(TimedEvent { tick, event });
    }

    /// Get events since a specific tick
    pub fn events_since(&self, since_tick: Tick) -> Vec<&GameEvent> {
        self.events
            .iter()
            .filter(|e| e.tick.is_newer_than(since_tick))
            .map(|e| &e.event)
            .collect()
    }

    /// Get all pending events
    pub fn all_events(&self) -> Vec<&GameEvent> {
        self.events.iter().map(|e| &e.event).collect()
    }

    /// Clear events older than a tick
    pub fn prune(&mut self, older_than: Tick) {
        while let Some(front) = self.events.front() {
            if older_than.is_newer_than(front.tick) {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get event count
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::PlayerId;

    #[test]
    fn test_push_and_get() {
        let mut buffer = EventBuffer::new();

        buffer.push(Tick(1), GameEvent::PlayerLeft { id: PlayerId(1) });
        buffer.push(Tick(2), GameEvent::PlayerLeft { id: PlayerId(2) });

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.all_events().len(), 2);
    }

    #[test]
    fn test_events_since() {
        let mut buffer = EventBuffer::new();

        buffer.push(Tick(1), GameEvent::PlayerLeft { id: PlayerId(1) });
        buffer.push(Tick(5), GameEvent::PlayerLeft { id: PlayerId(2) });
        buffer.push(Tick(10), GameEvent::PlayerLeft { id: PlayerId(3) });

        let events = buffer.events_since(Tick(3));
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_prune() {
        let mut buffer = EventBuffer::new();

        buffer.push(Tick(1), GameEvent::PlayerLeft { id: PlayerId(1) });
        buffer.push(Tick(5), GameEvent::PlayerLeft { id: PlayerId(2) });
        buffer.push(Tick(10), GameEvent::PlayerLeft { id: PlayerId(3) });

        buffer.prune(Tick(6));
        assert_eq!(buffer.len(), 1);
    }
}
