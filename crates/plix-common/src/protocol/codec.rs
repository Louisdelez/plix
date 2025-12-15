//! Binary codec for protocol messages

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

use super::MAX_PAYLOAD_SIZE;

/// Protocol encoding/decoding errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Message too large: {0} bytes (max {MAX_PAYLOAD_SIZE})")]
    MessageTooLarge(usize),

    #[error("Failed to encode message: {0}")]
    EncodeError(#[from] bincode::Error),

    #[error("Failed to decode message: {0}")]
    DecodeError(String),

    #[error("Invalid protocol version: expected {expected}, got {got}")]
    VersionMismatch { expected: u8, got: u8 },
}

/// Encode a message to bytes
pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>, ProtocolError> {
    let bytes = bincode::serialize(message)?;
    if bytes.len() > MAX_PAYLOAD_SIZE {
        return Err(ProtocolError::MessageTooLarge(bytes.len()));
    }
    Ok(bytes)
}

/// Decode a message from bytes
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ProtocolError> {
    bincode::deserialize(bytes).map_err(|e| ProtocolError::DecodeError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ClientMessage, ServerMessage, PROTOCOL_VERSION};
    use crate::time::Tick;
    use crate::types::PlayerId;

    #[test]
    fn test_client_message_roundtrip() {
        let msg = ClientMessage::Connect {
            protocol_version: PROTOCOL_VERSION,
            name: "TestPlayer".to_string(),
        };

        let bytes = encode(&msg).unwrap();
        let decoded: ClientMessage = decode(&bytes).unwrap();

        match decoded {
            ClientMessage::Connect {
                protocol_version,
                name,
            } => {
                assert_eq!(protocol_version, PROTOCOL_VERSION);
                assert_eq!(name, "TestPlayer");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_roundtrip() {
        let msg = ServerMessage::Connected {
            player_id: PlayerId(42),
            tick: Tick(1000),
            tick_rate: 60,
            arena_data: vec![1, 2, 3, 4],
        };

        let bytes = encode(&msg).unwrap();
        let decoded: ServerMessage = decode(&bytes).unwrap();

        match decoded {
            ServerMessage::Connected {
                player_id,
                tick,
                tick_rate,
                arena_data,
            } => {
                assert_eq!(player_id, PlayerId(42));
                assert_eq!(tick, Tick(1000));
                assert_eq!(tick_rate, 60);
                assert_eq!(arena_data, vec![1, 2, 3, 4]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_message_size_limit() {
        let large_data = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        let msg = ServerMessage::Connected {
            player_id: PlayerId(1),
            tick: Tick(0),
            tick_rate: 60,
            arena_data: large_data,
        };

        let result = encode(&msg);
        assert!(matches!(result, Err(ProtocolError::MessageTooLarge(_))));
    }
}
