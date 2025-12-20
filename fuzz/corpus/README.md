# Fuzz Corpus

This directory contains seed inputs for fuzz testing.

## Corpus Structure

- `client_messages/` - Valid ClientMessage binary samples
- `server_messages/` - Valid ServerMessage binary samples
- `modsync/` - Valid PayloadChunk, PayloadBegin, PayloadAck, PayloadEnd samples

## Generating Corpus

Corpus files are binary-encoded protocol messages. Generate them by:

1. Running the test suite which exercises encode/decode roundtrips
2. Using the `plix-tools` crate to generate specific message types
3. Capturing real protocol traffic during testing

## Fuzz Testing

Run fuzz tests with:

```bash
cd fuzz
cargo fuzz run fuzz_decode_client_message
cargo fuzz run fuzz_decode_server_message
cargo fuzz run fuzz_decode_modsync_chunk
```

With timeout:
```bash
cargo fuzz run fuzz_decode_client_message -- -max_total_time=300
```
