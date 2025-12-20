//! Memory bomb test mod for memory limit enforcement testing
//!
//! This mod intentionally attempts to allocate excessive memory via memory.grow
//! to test that the ResourceLimiter-based memory limits work correctly.

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

/// Handle incoming events - this one tries to allocate huge amounts of memory!
#[no_mangle]
pub extern "C" fn mod_on_event(_event_id: i32, _payload_ptr: i32, _payload_len: i32) -> i32 {
    // Try to grow memory by a huge amount (512 pages = 32 MiB each time)
    // Each WASM page is 64 KiB, so 512 pages = 32 MiB
    // We'll try to allocate way more than the 32 MiB limit

    let mut total_pages: i32 = 0;

    // Try to allocate 1024 pages at a time (64 MiB chunks)
    // This should quickly exceed the 32 MiB default limit
    loop {
        // memory.grow returns -1 if it fails
        let result = core::arch::wasm32::memory_grow(0, 1024);
        if result == usize::MAX {
            // Growth failed - that's expected when we hit the limit
            break;
        }
        total_pages += 1024;

        // Safety: don't loop forever, cap at 10 iterations (640 MiB attempt)
        if total_pages >= 10240 {
            break;
        }
    }

    // Return the number of pages we managed to allocate / 16 (so it fits in i32 nicely)
    // This lets the test verify how much was allocated before being stopped
    total_pages / 16
}

/// Shutdown the mod
#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 {
    0 // Success
}
