//! Chat filter test mod for plix WASM runtime testing
//!
//! This mod demonstrates event handling:
//! - Subscribes to PlayerChat events (event ID 5)
//! - Checks for capability to cancel
//! - Cancels events containing "badword"

// Event type IDs (from abi/mod.rs)
const EVENT_PLAYER_CHAT: i32 = 5;

// Host function imports from "plix" module
#[link(wasm_import_module = "plix")]
extern "C" {
    fn log(level: i32, ptr: i32, len: i32) -> i32;
    fn subscribe_event(event_type: i32) -> i32;
    fn cancel_event() -> i32;
    #[allow(dead_code)]
    fn has_capability(cap_id: i32) -> i32;
}

/// Initialize the mod - subscribe to chat events
#[no_mangle]
pub extern "C" fn mod_init() -> i32 {
    unsafe {
        // Subscribe to PlayerChat events
        subscribe_event(EVENT_PLAYER_CHAT);

        // Log initialization
        let msg = "Chat filter mod initialized";
        log(2, msg.as_ptr() as i32, msg.len() as i32); // level 2 = INFO
    }
    0 // Success
}

/// Handle incoming events
///
/// For PlayerChat events, check if message contains "badword" and cancel if so
#[no_mangle]
pub extern "C" fn mod_on_event(event_id: i32, payload_ptr: i32, payload_len: i32) -> i32 {
    if event_id != EVENT_PLAYER_CHAT {
        return 0; // Not our event
    }

    // In a real mod, we would deserialize the payload to get the message
    // For testing, we just look at the raw bytes for "badword"
    let payload = unsafe {
        core::slice::from_raw_parts(payload_ptr as *const u8, payload_len as usize)
    };

    // Check if payload contains "badword" (simple byte search)
    let badword = b"badword";
    let contains_badword = payload
        .windows(badword.len())
        .any(|window| window == badword);

    if contains_badword {
        unsafe {
            // Log that we're filtering
            let msg = "Filtering chat message with badword";
            log(1, msg.as_ptr() as i32, msg.len() as i32); // level 1 = WARN

            // Try to cancel the event
            let result = cancel_event();
            if result != 0 {
                // Failed to cancel (probably no permission)
                let msg = "Failed to cancel event - no permission";
                log(1, msg.as_ptr() as i32, msg.len() as i32);
                return 0; // Return 0 = event not cancelled
            }
            // Cancel succeeded! Return non-zero to indicate event was cancelled
            return 1;
        }
    }

    0 // Success, event not cancelled
}

/// Shutdown the mod
#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 {
    unsafe {
        let msg = "Chat filter mod shutting down";
        log(2, msg.as_ptr() as i32, msg.len() as i32);
    }
    0 // Success
}
