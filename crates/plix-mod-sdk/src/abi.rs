//! ABI (Application Binary Interface) for host function calls
//!
//! This module provides the low-level FFI declarations for host functions
//! and memory management helpers. Mod authors should use the higher-level
//! wrappers in other modules instead of calling these directly.

use alloc::string::String;
use alloc::vec::Vec;
use core::slice;

/// Response buffer location in WASM linear memory (64KB at offset 0x10000)
pub const RESPONSE_BUFFER_OFFSET: usize = 0x10000;
pub const RESPONSE_BUFFER_SIZE: usize = 65536;

// === Host Function Declarations ===
// These are provided by the Plix mod runtime (wasmtime)

#[link(wasm_import_module = "plix")]
extern "C" {
    // === Logging (0x01) ===
    /// Log a message at the given level
    /// level: 0=error, 1=warn, 2=info, 3=debug, 4=trace
    fn plix_log(level: i32, msg_ptr: i32, msg_len: i32) -> i32;

    // === Capabilities (0x10-0x12) ===
    /// Check if mod has a capability
    fn plix_has_capability(cap_id: i32) -> i32;

    /// Get the runtime API version
    fn plix_get_api_version() -> i32;

    /// Get the runtime ABI version
    fn plix_get_abi_version() -> i32;

    // === Events (0x20-0x21) ===
    /// Subscribe to an event type
    fn plix_subscribe_event(event_type: i32) -> i32;

    /// Cancel the current event (must be in event handler context)
    fn plix_cancel_event() -> i32;

    // === Generic Host Call ===
    /// Make a generic host call with request/response
    /// op: operation code (see HostCallOp)
    /// req_ptr, req_len: bincode-encoded request
    /// Returns: 0 on success, error code on failure
    /// Response is written to the response buffer (plix_response_ptr/len)
    fn plix_host_call(op: i32, req_ptr: i32, req_len: i32) -> i32;

    /// Get pointer to response data after a host call
    fn plix_response_ptr() -> i32;

    /// Get length of response data after a host call
    fn plix_response_len() -> i32;
}

// === Operation Codes ===

/// Operation codes for host function calls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HostCallOp {
    // Logging (0x01-0x0F)
    Log = 0x01,

    // Capabilities (0x10-0x1F)
    HasCapability = 0x10,
    GetApiVersion = 0x11,
    GetAbiVersion = 0x12,

    // Events (0x20-0x2F)
    SubscribeEvent = 0x20,
    CancelEvent = 0x21,

    // World API (0x30-0x3F)
    WorldGetBlock = 0x30,
    WorldSetBlock = 0x31,
    WorldRaycast = 0x32,
    WorldQueryAabb = 0x33,

    // Entity API (0x40-0x4F)
    EntityGetTransform = 0x40,
    EntityGetHealth = 0x41,
    EntityApplyDamage = 0x42,
    EntityApplyImpulse = 0x43,

    // Net API (0x50-0x5F)
    NetSend = 0x50,
    NetBroadcast = 0x51,

    // Timer API (0x60-0x6F)
    TimerSetTimeout = 0x60,
    TimerSetInterval = 0x61,
    TimerClear = 0x62,
}

// === Safe Wrappers for Direct Host Functions ===

/// Log a message at the given level
pub fn log(level: i32, message: &str) {
    let bytes = message.as_bytes();
    unsafe {
        plix_log(level, bytes.as_ptr() as i32, bytes.len() as i32);
    }
}

/// Check if the mod has a capability
pub fn has_capability(cap_id: u32) -> bool {
    unsafe { plix_has_capability(cap_id as i32) != 0 }
}

/// Get the runtime API version
pub fn get_api_version() -> u32 {
    unsafe { plix_get_api_version() as u32 }
}

/// Get the runtime ABI version
pub fn get_abi_version() -> u32 {
    unsafe { plix_get_abi_version() as u32 }
}

/// Subscribe to an event type
pub fn subscribe_event(event_type: u8) -> i32 {
    unsafe { plix_subscribe_event(event_type as i32) }
}

/// Cancel the current event
pub fn cancel_event() -> i32 {
    unsafe { plix_cancel_event() }
}

// === Generic Host Call with Response ===

/// Make a host call with a bincode-encoded request
///
/// Returns Ok(response_bytes) on success, Err(error_code) on failure.
pub fn host_call(op: HostCallOp, request: &[u8]) -> Result<Vec<u8>, i32> {
    let result = unsafe { plix_host_call(op as i32, request.as_ptr() as i32, request.len() as i32) };

    if result != 0 {
        return Err(result);
    }

    // Read response from buffer
    let response = read_response();
    Ok(response)
}

/// Read the response buffer after a host call
pub fn read_response() -> Vec<u8> {
    unsafe {
        let ptr = plix_response_ptr() as usize;
        let len = plix_response_len() as usize;

        if len == 0 {
            return Vec::new();
        }

        let slice = slice::from_raw_parts(ptr as *const u8, len);
        slice.to_vec()
    }
}

/// Get response pointer (for manual response handling)
pub fn response_ptr() -> usize {
    unsafe { plix_response_ptr() as usize }
}

/// Get response length (for manual response handling)
pub fn response_len() -> usize {
    unsafe { plix_response_len() as usize }
}

// === Memory Helpers ===

/// Convert a Rust string slice to (ptr, len) for FFI
pub fn str_to_ptr_len(s: &str) -> (i32, i32) {
    let bytes = s.as_bytes();
    (bytes.as_ptr() as i32, bytes.len() as i32)
}

/// Convert a byte slice to (ptr, len) for FFI
pub fn bytes_to_ptr_len(bytes: &[u8]) -> (i32, i32) {
    (bytes.as_ptr() as i32, bytes.len() as i32)
}

/// Read a string from the response buffer (assumes UTF-8)
pub fn read_response_string() -> Option<String> {
    let bytes = read_response();
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_call_op_values() {
        assert_eq!(HostCallOp::Log as u8, 0x01);
        assert_eq!(HostCallOp::HasCapability as u8, 0x10);
        assert_eq!(HostCallOp::WorldGetBlock as u8, 0x30);
        assert_eq!(HostCallOp::EntityGetTransform as u8, 0x40);
        assert_eq!(HostCallOp::NetSend as u8, 0x50);
        assert_eq!(HostCallOp::TimerSetTimeout as u8, 0x60);
    }

    #[test]
    fn test_str_to_ptr_len() {
        let s = "hello";
        let (ptr, len) = str_to_ptr_len(s);
        assert_eq!(len, 5);
        assert!(ptr > 0);
    }
}
