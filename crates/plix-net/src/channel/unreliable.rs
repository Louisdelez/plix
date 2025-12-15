//! Unreliable channel - fire and forget, no guarantees

use crate::packet::{Channel, Packet, PacketHeader};

/// Unreliable channel for fire-and-forget messages
///
/// Used for time-sensitive data where old packets are useless:
/// - Player inputs
/// - World snapshots
#[derive(Debug, Default)]
pub struct UnreliableChannel {
    /// Next sequence number to use
    next_sequence: u16,
    /// Last received remote sequence
    remote_sequence: u16,
    /// Bitmap of received packets
    received_bits: u32,
    /// Whether we've received any packet yet
    received_first: bool,
}

impl UnreliableChannel {
    /// Create a new unreliable channel
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a packet with the given payload
    pub fn create_packet(&mut self, payload: Vec<u8>) -> Packet {
        let header = PacketHeader::new(
            Channel::Unreliable,
            self.next_sequence,
            self.remote_sequence,
            self.received_bits,
        );
        self.next_sequence = self.next_sequence.wrapping_add(1);

        Packet { header, payload }
    }

    /// Process a received packet, updating ack state
    /// Returns true if this is a new packet (not duplicate)
    pub fn receive(&mut self, header: &PacketHeader) -> bool {
        let sequence = header.sequence;

        // Handle first packet specially
        if !self.received_first {
            self.received_first = true;
            self.remote_sequence = sequence;
            return true;
        }

        // Check if this is newer than our current remote sequence
        let diff = sequence.wrapping_sub(self.remote_sequence) as i16;

        if diff > 0 {
            // New packet, update state
            if diff < 32 {
                self.received_bits = self.received_bits.checked_shl(diff as u32).unwrap_or(0);
                // Mark previous remote_sequence as received at bit (diff - 1)
                self.received_bits |= 1 << (diff as u32 - 1);
            } else {
                self.received_bits = 0;
            }
            self.remote_sequence = sequence;
            true
        } else if diff < 0 && diff > -32 {
            // Old but recent packet, mark in bitmap
            let bit_index = (-diff - 1) as u32;
            let bit = 1u32 << bit_index;
            if self.received_bits & bit == 0 {
                self.received_bits |= bit;
                true // New, but out of order
            } else {
                false // Duplicate
            }
        } else {
            // Very old or duplicate (diff == 0 means same sequence, which is duplicate)
            false
        }
    }

    /// Get the current remote sequence and ack bits for header creation
    pub fn ack_state(&self) -> (u16, u32) {
        (self.remote_sequence, self.received_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_increment() {
        let mut channel = UnreliableChannel::new();

        let p1 = channel.create_packet(vec![]);
        let p2 = channel.create_packet(vec![]);

        assert_eq!(p1.header.sequence, 0);
        assert_eq!(p2.header.sequence, 1);
    }

    #[test]
    fn test_receive_in_order() {
        let mut channel = UnreliableChannel::new();

        let h1 = PacketHeader::new(Channel::Unreliable, 0, 0, 0);
        let h2 = PacketHeader::new(Channel::Unreliable, 1, 0, 0);
        let h3 = PacketHeader::new(Channel::Unreliable, 2, 0, 0);

        assert!(channel.receive(&h1));
        assert!(channel.receive(&h2));
        assert!(channel.receive(&h3));

        assert_eq!(channel.remote_sequence, 2);
    }

    #[test]
    fn test_receive_out_of_order() {
        let mut channel = UnreliableChannel::new();

        let h1 = PacketHeader::new(Channel::Unreliable, 0, 0, 0);
        let h3 = PacketHeader::new(Channel::Unreliable, 2, 0, 0);
        let h2 = PacketHeader::new(Channel::Unreliable, 1, 0, 0);

        assert!(channel.receive(&h1));
        assert!(channel.receive(&h3)); // Skip 1
        assert!(channel.receive(&h2)); // Late arrival

        assert_eq!(channel.remote_sequence, 2);
    }

    #[test]
    fn test_duplicate_detection() {
        let mut channel = UnreliableChannel::new();

        let h1 = PacketHeader::new(Channel::Unreliable, 5, 0, 0);

        assert!(channel.receive(&h1));
        assert!(!channel.receive(&h1)); // Duplicate
    }
}
