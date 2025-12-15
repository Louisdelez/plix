//! Ordered reliable channel - guaranteed delivery, in-order

use std::collections::{BTreeMap, VecDeque};
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

/// Buffered received packet awaiting in-order delivery
#[derive(Debug)]
struct BufferedPacket {
    payload: Vec<u8>,
}

/// Ordered reliable channel - guaranteed delivery with in-order processing
///
/// Used for critical ordered operations:
/// - Connection handshake
/// - Round start/end
/// - Kick/disconnect
#[derive(Debug)]
pub struct OrderedChannel {
    /// Next sequence number to send
    next_send_sequence: u16,
    /// Next sequence number we expect to receive in-order
    next_receive_sequence: u16,
    /// Last received remote sequence (for ack)
    remote_sequence: u16,
    /// Bitmap of received packets
    received_bits: u32,
    /// Packets awaiting acknowledgment
    pending_send: VecDeque<PendingPacket>,
    /// Out-of-order received packets buffered for later delivery
    receive_buffer: BTreeMap<u16, BufferedPacket>,
    /// Base RTT for resend timing
    rtt: Duration,
}

impl OrderedChannel {
    /// Maximum retry attempts
    pub const MAX_RETRIES: u8 = 10;
    /// Maximum buffer size for out-of-order packets
    pub const MAX_BUFFER_SIZE: usize = 64;

    /// Create a new ordered channel
    pub fn new() -> Self {
        Self {
            next_send_sequence: 0,
            next_receive_sequence: 0,
            remote_sequence: 0,
            received_bits: 0,
            pending_send: VecDeque::new(),
            receive_buffer: BTreeMap::new(),
            rtt: Duration::from_millis(100),
        }
    }

    /// Send a packet (queued for tracking)
    pub fn send(&mut self, payload: Vec<u8>) -> Packet {
        let sequence = self.next_send_sequence;
        self.next_send_sequence = self.next_send_sequence.wrapping_add(1);

        let header = PacketHeader::new(
            Channel::ReliableOrdered,
            sequence,
            self.remote_sequence,
            self.received_bits,
        );

        self.pending_send.push_back(PendingPacket {
            sequence,
            payload: payload.clone(),
            sent_at: Instant::now(),
            retries: 0,
        });

        Packet { header, payload }
    }

    /// Process a received packet
    /// Returns payloads ready for in-order delivery
    pub fn receive(&mut self, header: &PacketHeader, payload: Vec<u8>) -> Vec<Vec<u8>> {
        // Process acks
        self.process_acks(header.ack, header.ack_bits);

        // Update receive tracking
        self.update_receive_tracking(header.sequence);

        let sequence = header.sequence;
        let expected = self.next_receive_sequence;

        // Check if this is the packet we're waiting for
        if sequence == expected {
            // Deliver this one and any buffered ones that follow
            let mut deliveries = vec![payload];
            self.next_receive_sequence = self.next_receive_sequence.wrapping_add(1);

            // Check buffer for consecutive packets
            while let Some(buffered) = self.receive_buffer.remove(&self.next_receive_sequence) {
                deliveries.push(buffered.payload);
                self.next_receive_sequence = self.next_receive_sequence.wrapping_add(1);
            }

            deliveries
        } else {
            // Check if this is a future packet we should buffer
            let diff = sequence.wrapping_sub(expected) as i16;
            if diff > 0 && diff < Self::MAX_BUFFER_SIZE as i16 {
                // Buffer for later
                if !self.receive_buffer.contains_key(&sequence) {
                    self.receive_buffer
                        .insert(sequence, BufferedPacket { payload });
                }
            }
            // Else: old duplicate, ignore

            vec![]
        }
    }

    /// Update receive tracking state
    fn update_receive_tracking(&mut self, sequence: u16) {
        let diff = sequence.wrapping_sub(self.remote_sequence) as i16;

        if diff > 0 {
            if diff < 32 {
                self.received_bits = self.received_bits.checked_shl(diff as u32).unwrap_or(0);
                self.received_bits |= 1;
            } else {
                self.received_bits = 0;
            }
            self.remote_sequence = sequence;
        } else if diff < 0 && diff > -32 {
            let bit_index = (-diff - 1) as u32;
            self.received_bits |= 1 << bit_index;
        }
    }

    /// Process acknowledgments
    fn process_acks(&mut self, ack: u16, ack_bits: u32) {
        self.pending_send.retain(|pending| {
            let diff = ack.wrapping_sub(pending.sequence) as i16;

            if diff == 0 {
                return false;
            } else if diff > 0 && diff <= 32 {
                let bit_index = (diff - 1) as u32;
                if ack_bits & (1 << bit_index) != 0 {
                    return false;
                }
            }
            true
        });
    }

    /// Get packets that need resending
    pub fn get_resends(&mut self) -> Vec<Packet> {
        let resend_timeout = self.rtt * 3 / 2;
        let resend_timeout = resend_timeout.max(Duration::from_millis(100));
        let now = Instant::now();

        let mut resends = Vec::new();

        for pending in &mut self.pending_send {
            if now.duration_since(pending.sent_at) >= resend_timeout {
                if pending.retries < Self::MAX_RETRIES {
                    pending.retries += 1;
                    pending.sent_at = now;

                    let header = PacketHeader::new(
                        Channel::ReliableOrdered,
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
        self.rtt = (self.rtt * 7 + rtt) / 8;
    }

    /// Check for failed packets
    pub fn has_failed_packets(&self) -> bool {
        self.pending_send
            .iter()
            .any(|p| p.retries >= Self::MAX_RETRIES)
    }

    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.pending_send.len()
    }

    /// Get buffered receive count
    pub fn buffered_count(&self) -> usize {
        self.receive_buffer.len()
    }
}

impl Default for OrderedChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_order_delivery() {
        let mut channel = OrderedChannel::new();

        let h0 = PacketHeader::new(Channel::ReliableOrdered, 0, 0, 0);
        let h1 = PacketHeader::new(Channel::ReliableOrdered, 1, 0, 0);
        let h2 = PacketHeader::new(Channel::ReliableOrdered, 2, 0, 0);

        let d0 = channel.receive(&h0, vec![0]);
        let d1 = channel.receive(&h1, vec![1]);
        let d2 = channel.receive(&h2, vec![2]);

        assert_eq!(d0, vec![vec![0]]);
        assert_eq!(d1, vec![vec![1]]);
        assert_eq!(d2, vec![vec![2]]);
    }

    #[test]
    fn test_out_of_order_buffering() {
        let mut channel = OrderedChannel::new();

        let h0 = PacketHeader::new(Channel::ReliableOrdered, 0, 0, 0);
        let h2 = PacketHeader::new(Channel::ReliableOrdered, 2, 0, 0);
        let h1 = PacketHeader::new(Channel::ReliableOrdered, 1, 0, 0);

        // Receive 0, then 2 (skip 1)
        let d0 = channel.receive(&h0, vec![0]);
        assert_eq!(d0, vec![vec![0]]);

        let d2 = channel.receive(&h2, vec![2]);
        assert!(d2.is_empty()); // Buffered, waiting for 1
        assert_eq!(channel.buffered_count(), 1);

        // Now receive 1, should deliver both 1 and 2
        let d1 = channel.receive(&h1, vec![1]);
        assert_eq!(d1, vec![vec![1], vec![2]]);
        assert_eq!(channel.buffered_count(), 0);
    }
}
