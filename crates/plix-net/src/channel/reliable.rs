//! Reliable channel - guaranteed delivery, no ordering

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::packet::{Channel, Packet, PacketHeader};

/// Pending packet awaiting acknowledgment
#[derive(Debug)]
struct PendingPacket {
    sequence: u16,
    payload: Vec<u8>,
    sent_at: Instant,
    retries: u8,
}

/// Reliable channel for guaranteed delivery
///
/// Packets are resent until acknowledged. No ordering guarantee.
/// Used for:
/// - Game events (hits, deaths)
/// - Connection management
#[derive(Debug)]
pub struct ReliableChannel {
    /// Next sequence number to use
    next_sequence: u16,
    /// Last received remote sequence
    remote_sequence: u16,
    /// Bitmap of received packets
    received_bits: u32,
    /// Packets awaiting acknowledgment
    pending: VecDeque<PendingPacket>,
    /// Base RTT for resend timing
    rtt: Duration,
}

impl ReliableChannel {
    /// Maximum retry attempts before giving up
    pub const MAX_RETRIES: u8 = 10;

    /// Create a new reliable channel
    pub fn new() -> Self {
        Self {
            next_sequence: 0,
            remote_sequence: 0,
            received_bits: 0,
            pending: VecDeque::new(),
            rtt: Duration::from_millis(100), // Initial estimate
        }
    }

    /// Create a packet with the given payload and queue for tracking
    pub fn send(&mut self, payload: Vec<u8>) -> Packet {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);

        let header = PacketHeader::new(
            Channel::Reliable,
            sequence,
            self.remote_sequence,
            self.received_bits,
        );

        // Track for resend
        self.pending.push_back(PendingPacket {
            sequence,
            payload: payload.clone(),
            sent_at: Instant::now(),
            retries: 0,
        });

        Packet { header, payload }
    }

    /// Process received packet, updating ack state
    /// Returns true if this is a new packet
    pub fn receive(&mut self, header: &PacketHeader) -> bool {
        // Process acks from the remote side
        self.process_acks(header.ack, header.ack_bits);

        // Update our receive tracking
        let sequence = header.sequence;
        let diff = sequence.wrapping_sub(self.remote_sequence) as i16;

        if diff > 0 {
            if diff < 32 {
                self.received_bits = self.received_bits.checked_shl(diff as u32).unwrap_or(0);
                self.received_bits |= 1;
            } else {
                self.received_bits = 0;
            }
            self.remote_sequence = sequence;
            true
        } else if diff < 0 && diff > -32 {
            let bit_index = (-diff - 1) as u32;
            let bit = 1u32 << bit_index;
            if self.received_bits & bit == 0 {
                self.received_bits |= bit;
                true
            } else {
                false // Duplicate
            }
        } else {
            false
        }
    }

    /// Process acknowledgments from remote
    fn process_acks(&mut self, ack: u16, ack_bits: u32) {
        self.pending.retain(|pending| {
            let diff = ack.wrapping_sub(pending.sequence) as i16;

            if diff == 0 {
                // This packet is acked
                return false;
            } else if diff > 0 && diff <= 32 {
                // Check in ack_bits
                let bit_index = (diff - 1) as u32;
                if ack_bits & (1 << bit_index) != 0 {
                    return false; // Acked
                }
            }
            true // Keep, not acked
        });
    }

    /// Get packets that need to be resent
    pub fn get_resends(&mut self) -> Vec<Packet> {
        let resend_timeout = self.rtt * 3 / 2; // RTT * 1.5
        let resend_timeout = resend_timeout.max(Duration::from_millis(100));
        let now = Instant::now();

        let mut resends = Vec::new();

        for pending in &mut self.pending {
            if now.duration_since(pending.sent_at) >= resend_timeout {
                if pending.retries < Self::MAX_RETRIES {
                    pending.retries += 1;
                    pending.sent_at = now;

                    let header = PacketHeader::new(
                        Channel::Reliable,
                        pending.sequence,
                        self.remote_sequence,
                        self.received_bits,
                    );

                    resends.push(Packet {
                        header,
                        payload: pending.payload.clone(),
                    });
                }
            }
        }

        resends
    }

    /// Update RTT estimate
    pub fn update_rtt(&mut self, rtt: Duration) {
        // Simple exponential moving average
        self.rtt = (self.rtt * 7 + rtt) / 8;
    }

    /// Check if any packets have exceeded max retries
    pub fn has_failed_packets(&self) -> bool {
        self.pending.iter().any(|p| p.retries >= Self::MAX_RETRIES)
    }

    /// Get count of pending (unacked) packets
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get ack state for header creation
    pub fn ack_state(&self) -> (u16, u32) {
        (self.remote_sequence, self.received_bits)
    }
}

impl Default for ReliableChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_tracks_pending() {
        let mut channel = ReliableChannel::new();

        channel.send(vec![1, 2, 3]);
        channel.send(vec![4, 5, 6]);

        assert_eq!(channel.pending_count(), 2);
    }

    #[test]
    fn test_ack_removes_pending() {
        let mut channel = ReliableChannel::new();

        channel.send(vec![1, 2, 3]);
        assert_eq!(channel.pending_count(), 1);

        // Simulate receiving an ack for sequence 0
        let ack_header = PacketHeader::new(Channel::Reliable, 0, 0, 0);
        channel.receive(&ack_header);

        assert_eq!(channel.pending_count(), 0);
    }
}
