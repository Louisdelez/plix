//! Codec for encoding/decoding ABI payloads
//!
//! Uses bincode for binary serialization, matching the runtime.
//! All types must be compatible with plix-mod-runtime-wasm ABI v1.

use alloc::string::String;
use alloc::vec::Vec;
use crate::error::{err_invalid, ModError, Result};
use serde::{Deserialize, Serialize};

/// Encode a value to bincode bytes
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).map_err(|e| err_invalid(alloc::format!("Encode error: {}", e)))
}

/// Decode a value from bincode bytes
pub fn decode<'de, T: Deserialize<'de>>(data: &'de [u8]) -> Result<T> {
    bincode::deserialize(data).map_err(|e| err_invalid(alloc::format!("Decode error: {}", e)))
}

/// ABI Response structure (matches plix-mod-runtime-wasm)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiResponse {
    /// true if operation succeeded
    pub success: bool,
    /// Error code if failed (1-7 for EMOD001-007)
    pub error_code: Option<u16>,
    /// Human-readable error message
    pub error_message: Option<String>,
    /// bincode-serialized result data
    pub payload: Vec<u8>,
}

impl AbiResponse {
    /// Decode an ABI response from bytes
    pub fn decode(data: &[u8]) -> Result<Self> {
        decode(data)
    }

    /// Check if the response is successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Convert to Result, extracting payload on success
    pub fn into_result(self) -> Result<Vec<u8>> {
        if self.success {
            Ok(self.payload)
        } else {
            Err(ModError::from_abi(
                self.error_code.unwrap_or(7),
                self.error_message,
            ))
        }
    }

    /// Decode the payload as a specific type
    pub fn decode_payload<'de, T: Deserialize<'de>>(&'de self) -> Result<T> {
        decode(&self.payload)
    }
}

// === Request Payload Types ===
// These match plix-mod-runtime-wasm ABI v1

/// World GetBlock request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlockRequest {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// World SetBlock request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBlockRequest {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_id: u16,
}

/// World Raycast request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaycastRequest {
    pub origin_x: f32,
    pub origin_y: f32,
    pub origin_z: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub dir_z: f32,
    pub max_distance: f32,
}

/// World QueryAabb request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAabbRequest {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
    pub limit: u32,
}

/// Entity GetTransform request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransformRequest {
    pub entity_index: u32,
    pub entity_generation: u32,
}

/// Entity ApplyDamage request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyDamageRequest {
    pub entity_index: u32,
    pub entity_generation: u32,
    pub amount: f32,
    pub source_type: Option<String>,
}

/// Entity ApplyImpulse request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyImpulseRequest {
    pub entity_index: u32,
    pub entity_generation: u32,
    pub impulse_x: f32,
    pub impulse_y: f32,
    pub impulse_z: f32,
}

/// Net Send request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetSendRequest {
    pub channel: String,
    pub target_type: u8, // 0=Server, 1=Client(id), 2=AllClients, 3=Team(id)
    pub target_id: u64,  // player_id or team_id depending on target_type
    pub payload: Vec<u8>,
}

/// Net Broadcast request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBroadcastRequest {
    pub channel: String,
    pub payload: Vec<u8>,
}

/// Timer SetTimeout request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTimeoutRequest {
    pub delay_ms: u32,
    pub callback_id: u32,
}

/// Timer SetInterval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetIntervalRequest {
    pub interval_ms: u32,
    pub callback_id: u32,
}

/// Timer Clear request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearTimerRequest {
    pub timer_id: u32,
}

// === Response Payload Types ===

/// GetBlock response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlockResponse {
    pub block_id: u16,
}

/// Raycast hit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaycastHitResponse {
    pub hit: bool,
    pub pos_x: i32,
    pub pos_y: i32,
    pub pos_z: i32,
    pub block_id: u16,
    pub face: u8, // 0=Top, 1=Bottom, 2=North, 3=South, 4=East, 5=West
    pub distance: f32,
}

/// AABB query result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockQueryItem {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_id: u16,
}

/// AABB query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAabbResponse {
    pub blocks: Vec<BlockQueryItem>,
}

/// Transform response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResponse {
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub rot_w: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
}

/// Timer handle response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerHandleResponse {
    pub timer_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{PlayerChatPayload, BlockPlacedPayload};

    #[test]
    fn test_encode_decode_roundtrip() {
        let request = GetBlockRequest { x: 1, y: 2, z: 3 };
        let encoded = encode(&request).unwrap();
        let decoded: GetBlockRequest = decode(&encoded).unwrap();

        assert_eq!(decoded.x, 1);
        assert_eq!(decoded.y, 2);
        assert_eq!(decoded.z, 3);
    }

    #[test]
    fn test_abi_response_success() {
        let response = AbiResponse {
            success: true,
            error_code: None,
            error_message: None,
            payload: vec![42],
        };

        assert!(response.is_success());
        let result = response.into_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![42]);
    }

    #[test]
    fn test_abi_response_error() {
        let response = AbiResponse {
            success: false,
            error_code: Some(2),
            error_message: Some("Permission denied".into()),
            payload: vec![],
        };

        assert!(!response.is_success());
        let result = response.into_result();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::PermissionDenied);
    }

    #[test]
    fn test_raycast_request_encode() {
        let request = RaycastRequest {
            origin_x: 0.0,
            origin_y: 64.0,
            origin_z: 0.0,
            dir_x: 1.0,
            dir_y: 0.0,
            dir_z: 0.0,
            max_distance: 100.0,
        };
        let encoded = encode(&request).unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_net_send_request_encode() {
        let request = NetSendRequest {
            channel: "mod:test:ui".into(),
            target_type: 1, // Client
            target_id: 123,
            payload: vec![1, 2, 3],
        };
        let encoded = encode(&request).unwrap();
        let decoded: NetSendRequest = decode(&encoded).unwrap();

        assert_eq!(decoded.channel, "mod:test:ui");
        assert_eq!(decoded.target_type, 1);
        assert_eq!(decoded.target_id, 123);
    }

    // Event payload roundtrip tests
    #[test]
    fn test_player_chat_payload_roundtrip() {
        let payload = PlayerChatPayload {
            player_id: 42,
            text: "Hello world!".into(),
        };
        let encoded = encode(&payload).unwrap();
        let decoded: PlayerChatPayload = decode(&encoded).unwrap();

        assert_eq!(decoded.player_id, 42);
        assert_eq!(decoded.text, "Hello world!");
    }

    #[test]
    fn test_block_placed_payload_roundtrip() {
        let payload = BlockPlacedPayload {
            player_id: Some(123),
            pos: glam::IVec3::new(10, 64, -5),
            block_id: 42,
        };
        let encoded = encode(&payload).unwrap();
        let decoded: BlockPlacedPayload = decode(&encoded).unwrap();

        assert_eq!(decoded.player_id, Some(123));
        assert_eq!(decoded.pos.x, 10);
        assert_eq!(decoded.pos.y, 64);
        assert_eq!(decoded.pos.z, -5);
        assert_eq!(decoded.block_id, 42);
    }

    // World call payload tests
    #[test]
    fn test_set_block_request_roundtrip() {
        let request = SetBlockRequest {
            x: -100,
            y: 255,
            z: 50,
            block_id: 1024,
        };
        let encoded = encode(&request).unwrap();
        let decoded: SetBlockRequest = decode(&encoded).unwrap();

        assert_eq!(decoded.x, -100);
        assert_eq!(decoded.y, 255);
        assert_eq!(decoded.z, 50);
        assert_eq!(decoded.block_id, 1024);
    }

    #[test]
    fn test_query_aabb_request_roundtrip() {
        let request = QueryAabbRequest {
            min_x: 0,
            min_y: 60,
            min_z: 0,
            max_x: 16,
            max_y: 80,
            max_z: 16,
            limit: 1000,
        };
        let encoded = encode(&request).unwrap();
        let decoded: QueryAabbRequest = decode(&encoded).unwrap();

        assert_eq!(decoded.min_x, 0);
        assert_eq!(decoded.max_y, 80);
        assert_eq!(decoded.limit, 1000);
    }

    #[test]
    fn test_raycast_hit_response_roundtrip() {
        let response = RaycastHitResponse {
            hit: true,
            pos_x: 10,
            pos_y: 64,
            pos_z: 20,
            block_id: 3,
            face: 0, // Top
            distance: 5.5,
        };
        let encoded = encode(&response).unwrap();
        let decoded: RaycastHitResponse = decode(&encoded).unwrap();

        assert!(decoded.hit);
        assert_eq!(decoded.block_id, 3);
        assert!((decoded.distance - 5.5).abs() < f32::EPSILON);
    }
}
