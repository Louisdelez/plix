//! Minimal test mod for plix WASM runtime testing
//!
//! This mod exports all required functions with minimal implementations
//! to verify basic loading and initialization works.

/// Static flag to track if mod_init was called (for testing)
static mut INIT_CALLED: bool = false;

/// Initialize the mod
///
/// Returns 0 on success (following WASM convention)
#[no_mangle]
pub extern "C" fn mod_init() -> i32 {
    unsafe {
        INIT_CALLED = true;
    }
    0 // Success
}

/// Handle an event
///
/// # Arguments
/// * `event_id` - The type of event
/// * `payload_ptr` - Pointer to event payload in linear memory
/// * `payload_len` - Length of the payload
///
/// Returns 0 for success, non-zero for handled/cancelled
#[no_mangle]
pub extern "C" fn mod_on_event(_event_id: i32, _payload_ptr: i32, _payload_len: i32) -> i32 {
    0 // Success, event not cancelled
}

/// Shutdown the mod
///
/// Called when the mod is being unloaded. Returns 0 on success.
#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 {
    0 // Success
}
