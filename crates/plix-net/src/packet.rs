//! Packet header and framing for Plix networking
//!
//! Packet structure:
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │ Header (1 byte)                                         │
//! ├─────────────────────────────────────────────────────────┤
//! │ Sequence Number (2 bytes, big-endian)                   │
//! ├─────────────────────────────────────────────────────────┤
//! │ Ack Number (2 bytes, big-endian)                        │
//! ├─────────────────────────────────────────────────────────┤
//! │ Ack Bits (4 bytes, big-endian)                          │
//! ├─────────────────────────────────────────────────────────┤
//! │ Payload (variable, max 1389 bytes)                      │
//! └─────────────────────────────────────────────────────────┘
//! ```

use thiserror::Error;

/// Protocol version (2 bits, 0-3)
pub const PROTOCOL_VERSION: u8 = 0;

/// Header size in bytes
pub const HEADER_SIZE: usize = 9;

/// Maximum payload size
pub const MAX_PAYLOAD_SIZE: usize = 1389;

/// Channel type for packet delivery guarantees
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Channel {
    /// No guarantees, fire and forget
    Unreliable = 0,
    /// Guaranteed delivery, no ordering
    Reliable = 1,
    /// Guaranteed delivery, in-order
    ReliableOrdered = 2,
}

impl Channel {
    /// Parse channel from header byte
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            0 => Some(Channel::Unreliable),
            1 => Some(Channel::Reliable),
            2 => Some(Channel::ReliableOrdered),
            _ => None,
        }
    }

    /// Convert channel to bits for header
    pub fn to_bits(self) -> u8 {
        self as u8
    }
}

/// Packet header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    /// Protocol version
    pub version: u8,
    /// Delivery channel
    pub channel: Channel,
    /// Sequence number for this packet
    pub sequence: u16,
    /// Last received sequence number
    pub ack: u16,
    /// Bitmap of previous 32 packets received
    pub ack_bits: u32,
}

impl PacketHeader {
    /// Create a new packet header
    pub fn new(channel: Channel, sequence: u16, ack: u16, ack_bits: u32) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            channel,
            sequence,
            ack,
            ack_bits,
        }
    }

    /// Encode header to bytes
    pub fn encode(&self, buf: &mut [u8]) -> Result<(), PacketError> {
        if buf.len() < HEADER_SIZE {
            return Err(PacketError::BufferTooSmall);
        }

        // Header byte: [version:2][channel:2][reserved:4]
        buf[0] = (self.version & 0x03) << 6 | (self.channel.to_bits() & 0x03) << 4;

        // Sequence number (big-endian)
        buf[1..3].copy_from_slice(&self.sequence.to_be_bytes());

        // Ack number (big-endian)
        buf[3..5].copy_from_slice(&self.ack.to_be_bytes());

        // Ack bits (big-endian)
        buf[5..9].copy_from_slice(&self.ack_bits.to_be_bytes());

        Ok(())
    }

    /// Decode header from bytes
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < HEADER_SIZE {
            return Err(PacketError::BufferTooSmall);
        }

        let header_byte = buf[0];
        let version = (header_byte >> 6) & 0x03;
        let channel_bits = (header_byte >> 4) & 0x03;

        if version != PROTOCOL_VERSION {
            return Err(PacketError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                got: version,
            });
        }

        let channel =
            Channel::from_bits(channel_bits).ok_or(PacketError::InvalidChannel(channel_bits))?;

        let sequence = u16::from_be_bytes([buf[1], buf[2]]);
        let ack = u16::from_be_bytes([buf[3], buf[4]]);
        let ack_bits = u32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);

        Ok(Self {
            version,
            channel,
            sequence,
            ack,
            ack_bits,
        })
    }
}

/// Complete packet (header + payload)
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl Packet {
    /// Create a new packet
    pub fn new(header: PacketHeader, payload: Vec<u8>) -> Result<Self, PacketError> {
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(PacketError::PayloadTooLarge(payload.len()));
        }
        Ok(Self { header, payload })
    }

    /// Encode packet to bytes
    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        let mut buf = vec![0u8; HEADER_SIZE + self.payload.len()];
        self.header.encode(&mut buf)?;
        buf[HEADER_SIZE..].copy_from_slice(&self.payload);
        Ok(buf)
    }

    /// Decode packet from bytes
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        let header = PacketHeader::decode(buf)?;
        let payload = buf[HEADER_SIZE..].to_vec();
        Self::new(header, payload)
    }
}

/// Packet encoding/decoding errors
#[derive(Debug, Error)]
pub enum PacketError {
    #[error("Buffer too small for packet header")]
    BufferTooSmall,

    #[error("Protocol version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u8, got: u8 },

    #[error("Invalid channel: {0}")]
    InvalidChannel(u8),

    #[error("Payload too large: {0} bytes (max {MAX_PAYLOAD_SIZE})")]
    PayloadTooLarge(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_roundtrip() {
        let header = PacketHeader::new(Channel::Reliable, 1234, 5678, 0xDEADBEEF);

        let mut buf = [0u8; HEADER_SIZE];
        header.encode(&mut buf).unwrap();

        let decoded = PacketHeader::decode(&buf).unwrap();
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_packet_roundtrip() {
        let header = PacketHeader::new(Channel::Unreliable, 100, 99, 0);
        let payload = vec![1, 2, 3, 4, 5];
        let packet = Packet::new(header, payload.clone()).unwrap();

        let encoded = packet.encode().unwrap();
        let decoded = Packet::decode(&encoded).unwrap();

        assert_eq!(decoded.header, header);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_payload_too_large() {
        let header = PacketHeader::new(Channel::Unreliable, 0, 0, 0);
        let payload = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        assert!(Packet::new(header, payload).is_err());
    }
}
