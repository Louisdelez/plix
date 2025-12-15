//! Command buffer for client-side prediction

use std::collections::VecDeque;

use plix_common::protocol::PlayerInput;
use plix_common::types::InputSeq;

/// Maximum buffered commands
const MAX_COMMANDS: usize = 128;

/// Command buffer for prediction and reconciliation
#[derive(Debug, Default)]
pub struct CommandBuffer {
    /// Pending commands awaiting server confirmation
    pending: VecDeque<PlayerInput>,
}

impl CommandBuffer {
    /// Create a new command buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new command
    pub fn push(&mut self, input: PlayerInput) {
        if self.pending.len() >= MAX_COMMANDS {
            self.pending.pop_front();
        }
        self.pending.push_back(input);
    }

    /// Remove commands that have been acknowledged
    pub fn acknowledge(&mut self, acked_seq: InputSeq) {
        while let Some(front) = self.pending.front() {
            if !front.seq.is_newer_than(acked_seq) {
                self.pending.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get pending commands for replay (after reconciliation)
    pub fn pending_commands(&self) -> impl Iterator<Item = &PlayerInput> {
        self.pending.iter()
    }

    /// Get pending count
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Clear all pending commands
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::time::Tick;

    fn make_input(seq: u16) -> PlayerInput {
        PlayerInput::empty(InputSeq(seq), Tick(0))
    }

    #[test]
    fn test_acknowledge() {
        let mut buffer = CommandBuffer::new();

        buffer.push(make_input(1));
        buffer.push(make_input(2));
        buffer.push(make_input(3));
        buffer.push(make_input(4));

        buffer.acknowledge(InputSeq(2));

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.pending.front().unwrap().seq, InputSeq(3));
    }
}
