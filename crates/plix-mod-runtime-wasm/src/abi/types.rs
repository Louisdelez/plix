//! ABI type definitions for host-mod communication
//!
//! These types are serialized with bincode for efficient binary transfer.

use serde::{Deserialize, Serialize};

/// Request structure for API calls from mod to host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiRequest {
    /// ABI version (currently 1)
    pub abi_version: u8,
    /// Operation code (see HostCallOp)
    pub op: u8,
    /// bincode-serialized operation arguments
    pub payload: Vec<u8>,
}

impl AbiRequest {
    /// Create a new ABI request
    pub fn new(op: u8, payload: Vec<u8>) -> Self {
        Self {
            abi_version: super::ABI_VERSION as u8,
            op,
            payload,
        }
    }

    /// Encode this request to bincode bytes
    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Decode an ABI request from bincode bytes
    pub fn decode(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

/// Response structure from host calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiResponse {
    /// true if operation succeeded
    pub success: bool,
    /// Error code if failed (EMOD001-007)
    pub error_code: Option<u16>,
    /// Human-readable error message
    pub error_message: Option<String>,
    /// bincode-serialized result data
    pub payload: Vec<u8>,
}

impl AbiResponse {
    /// Create a successful response with payload
    pub fn success(payload: Vec<u8>) -> Self {
        Self {
            success: true,
            error_code: None,
            error_message: None,
            payload,
        }
    }

    /// Create a successful response with no payload
    pub fn success_empty() -> Self {
        Self::success(Vec::new())
    }

    /// Create an error response
    pub fn error(code: u16, message: impl Into<String>) -> Self {
        Self {
            success: false,
            error_code: Some(code),
            error_message: Some(message.into()),
            payload: Vec::new(),
        }
    }

    /// Create an error response from ModApiError
    pub fn from_mod_error(err: &plix_mod_core::errors::ModApiError) -> Self {
        let code = match err.code {
            plix_mod_core::errors::ErrorCode::InvalidArgument => 1,
            plix_mod_core::errors::ErrorCode::PermissionDenied => 2,
            plix_mod_core::errors::ErrorCode::NotFound => 3,
            plix_mod_core::errors::ErrorCode::OutOfBounds => 4,
            plix_mod_core::errors::ErrorCode::RateLimited => 5,
            plix_mod_core::errors::ErrorCode::WorldNotReady => 6,
            plix_mod_core::errors::ErrorCode::Unsupported => 7,
        };
        Self::error(code, &err.message)
    }

    /// Encode this response to bincode bytes
    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Decode an ABI response from bincode bytes
    pub fn decode(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

/// Encode a value to bincode bytes
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(value)
}

/// Decode a value from bincode bytes
pub fn decode<'de, T: Deserialize<'de>>(data: &'de [u8]) -> Result<T, bincode::Error> {
    bincode::deserialize(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_request_roundtrip() {
        let request = AbiRequest::new(0x30, vec![1, 2, 3, 4]);
        let encoded = request.encode().unwrap();
        let decoded = AbiRequest::decode(&encoded).unwrap();

        assert_eq!(decoded.abi_version, 1);
        assert_eq!(decoded.op, 0x30);
        assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_abi_response_success() {
        let response = AbiResponse::success(vec![42]);
        assert!(response.success);
        assert!(response.error_code.is_none());
        assert_eq!(response.payload, vec![42]);
    }

    #[test]
    fn test_abi_response_error() {
        let response = AbiResponse::error(2, "Permission denied");
        assert!(!response.success);
        assert_eq!(response.error_code, Some(2));
        assert_eq!(response.error_message.as_deref(), Some("Permission denied"));
    }

    #[test]
    fn test_abi_response_roundtrip() {
        let response = AbiResponse::error(5, "Rate limited");
        let encoded = response.encode().unwrap();
        let decoded = AbiResponse::decode(&encoded).unwrap();

        assert!(!decoded.success);
        assert_eq!(decoded.error_code, Some(5));
        assert_eq!(decoded.error_message.as_deref(), Some("Rate limited"));
    }

    #[test]
    fn test_encode_decode_struct() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestPayload {
            x: i32,
            y: i32,
            z: i32,
        }

        let payload = TestPayload { x: 1, y: 2, z: 3 };
        let encoded = encode(&payload).unwrap();
        let decoded: TestPayload = decode(&encoded).unwrap();

        assert_eq!(decoded, payload);
    }
}
