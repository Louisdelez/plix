//! Fuzz target for ServerMessage decoding
//!
//! Tests that decoding arbitrary bytes never panics or causes OOM.
//! The decoder must gracefully return errors for invalid input.

#![no_main]

use libfuzzer_sys::fuzz_target;
use plix_common::protocol::{decode, ServerMessage};

fuzz_target!(|data: &[u8]| {
    // Must not panic - invalid input should return error
    let _ = decode::<ServerMessage>(data);
});
