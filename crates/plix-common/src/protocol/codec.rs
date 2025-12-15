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
    use crate::protocol::{
        ClientMessage, GameEvent, MatchEndReason, MatchPhase, PlayerScore, ServerMessage,
        PROTOCOL_VERSION,
    };
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
    fn test_ready_toggle_roundtrip() {
        let msg = ClientMessage::ReadyToggle;

        let bytes = encode(&msg).unwrap();
        let decoded: ClientMessage = decode(&bytes).unwrap();

        assert!(matches!(decoded, ClientMessage::ReadyToggle));
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
    fn test_match_phase_changed_roundtrip() {
        let event = GameEvent::MatchPhaseChanged {
            from: MatchPhase::Lobby,
            to: MatchPhase::Countdown,
        };
        let msg = ServerMessage::Event(event);

        let bytes = encode(&msg).unwrap();
        let decoded: ServerMessage = decode(&bytes).unwrap();

        match decoded {
            ServerMessage::Event(GameEvent::MatchPhaseChanged { from, to }) => {
                assert_eq!(from, MatchPhase::Lobby);
                assert_eq!(to, MatchPhase::Countdown);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_countdown_tick_roundtrip() {
        let event = GameEvent::CountdownTick { remaining: 3 };
        let msg = ServerMessage::Event(event);

        let bytes = encode(&msg).unwrap();
        let decoded: ServerMessage = decode(&bytes).unwrap();

        match decoded {
            ServerMessage::Event(GameEvent::CountdownTick { remaining }) => {
                assert_eq!(remaining, 3);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_score_update_roundtrip() {
        let event = GameEvent::ScoreUpdate {
            player_id: PlayerId(5),
            kills: 10,
            deaths: 3,
        };
        let msg = ServerMessage::Event(event);

        let bytes = encode(&msg).unwrap();
        let decoded: ServerMessage = decode(&bytes).unwrap();

        match decoded {
            ServerMessage::Event(GameEvent::ScoreUpdate {
                player_id,
                kills,
                deaths,
            }) => {
                assert_eq!(player_id, PlayerId(5));
                assert_eq!(kills, 10);
                assert_eq!(deaths, 3);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_match_end_roundtrip() {
        let event = GameEvent::MatchEnd {
            winner: Some(PlayerId(7)),
            scores: vec![
                PlayerScore {
                    player_id: PlayerId(7),
                    name: "Winner".to_string(),
                    kills: 5,
                    deaths: 2,
                },
                PlayerScore {
                    player_id: PlayerId(8),
                    name: "Loser".to_string(),
                    kills: 3,
                    deaths: 4,
                },
            ],
            reason: MatchEndReason::ScoreLimit,
        };
        let msg = ServerMessage::Event(event);

        let bytes = encode(&msg).unwrap();
        let decoded: ServerMessage = decode(&bytes).unwrap();

        match decoded {
            ServerMessage::Event(GameEvent::MatchEnd {
                winner,
                scores,
                reason,
            }) => {
                assert_eq!(winner, Some(PlayerId(7)));
                assert_eq!(scores.len(), 2);
                assert_eq!(scores[0].kills, 5);
                assert_eq!(reason, MatchEndReason::ScoreLimit);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_arena_changed_roundtrip() {
        let event = GameEvent::ArenaChanged {
            name: "arena_02".to_string(),
        };
        let msg = ServerMessage::Event(event);

        let bytes = encode(&msg).unwrap();
        let decoded: ServerMessage = decode(&bytes).unwrap();

        match decoded {
            ServerMessage::Event(GameEvent::ArenaChanged { name }) => {
                assert_eq!(name, "arena_02");
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
