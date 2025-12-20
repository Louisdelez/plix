//! Test mod that is missing required exports
//!
//! This mod is intentionally missing mod_init to test export validation.

/// Handle an event - but mod_init is missing!
#[no_mangle]
pub extern "C" fn mod_on_event(_event_id: i32, _payload_ptr: i32, _payload_len: i32) -> i32 {
    0
}

/// Shutdown the mod
#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 {
    0
}
