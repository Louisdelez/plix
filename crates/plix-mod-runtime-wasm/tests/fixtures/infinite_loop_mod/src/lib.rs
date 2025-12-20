//! Infinite loop test mod for CPU budget enforcement testing
//!
//! This mod intentionally runs an infinite loop in its event handler
//! to test that the fuel-based CPU budget enforcement works correctly.

// Host function imports from "plix" module
#[link(wasm_import_module = "plix")]
extern "C" {
    fn subscribe_event(event_type: i32) -> i32;
}

/// Initialize the mod - subscribe to events
#[no_mangle]
pub extern "C" fn mod_init() -> i32 {
    unsafe {
        // Subscribe to event type 5 (PlayerChat)
        subscribe_event(5);
    }
    0 // Success
}

/// Handle incoming events - this one runs an infinite loop!
#[no_mangle]
pub extern "C" fn mod_on_event(_event_id: i32, _payload_ptr: i32, _payload_len: i32) -> i32 {
    // Intentionally run an infinite loop to test CPU budget enforcement
    let mut counter: u64 = 0;
    loop {
        counter = counter.wrapping_add(1);
        // Add some computation to consume fuel
        if counter % 1000000 == 0 {
            // This branch will never actually help us exit
            // but prevents the compiler from optimizing the loop away
            core::hint::black_box(counter);
        }
    }
}

/// Shutdown the mod
#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 {
    0 // Success
}
