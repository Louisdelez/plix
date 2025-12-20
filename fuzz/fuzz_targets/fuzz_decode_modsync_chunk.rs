//! Fuzz target for mod sync message decoding
//!
//! Tests that decoding PayloadChunk, PayloadBegin, PayloadAck, and PayloadEnd
//! from arbitrary bytes never panics or causes OOM.

#![no_main]

use libfuzzer_sys::fuzz_target;
use plix_common::protocol::messages::{PayloadAck, PayloadBegin, PayloadChunk, PayloadEnd};
use plix_common::protocol::decode;

fuzz_target!(|data: &[u8]| {
    // Test all mod sync message types
    // Must not panic - invalid input should return error

    // PayloadChunk
    let _ = decode::<PayloadChunk>(data);

    // PayloadBegin
    let _ = decode::<PayloadBegin>(data);

    // PayloadAck
    let _ = decode::<PayloadAck>(data);

    // PayloadEnd
    let _ = decode::<PayloadEnd>(data);
});
