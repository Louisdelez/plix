# Fuzzing Guide

This document describes how to run fuzz tests for the Plix protocol implementation.

## Prerequisites

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

Note: Fuzzing requires a nightly Rust toolchain. cargo-fuzz will automatically use nightly.

## Available Fuzz Targets

| Target | Description |
|--------|-------------|
| `fuzz_decode_client_message` | Fuzz ClientMessage decode from arbitrary bytes |
| `fuzz_decode_server_message` | Fuzz ServerMessage decode from arbitrary bytes |
| `fuzz_decode_modsync_chunk` | Fuzz PayloadChunk/Begin/Ack/End decode from arbitrary bytes |

## Running Fuzz Tests

### Basic Usage

```bash
# Enter the fuzz directory
cd fuzz

# List available targets
cargo fuzz list

# Run a specific target (runs until stopped with Ctrl+C)
cargo fuzz run fuzz_decode_client_message
```

### With Timeout

For CI or time-limited runs:

```bash
# Run for 5 minutes
cargo fuzz run fuzz_decode_client_message -- -max_total_time=300

# Run for 1 hour
cargo fuzz run fuzz_decode_client_message -- -max_total_time=3600
```

### With Custom Corpus

```bash
cargo fuzz run fuzz_decode_client_message corpus/client_messages/
```

### Running All Targets

```bash
for target in $(cargo fuzz list); do
    echo "Fuzzing $target for 60 seconds..."
    cargo fuzz run "$target" -- -max_total_time=60
done
```

## Interpreting Results

### Success Output

```
#12345    DONE   cov: 342 ft: 1234 corp: 56
```

- `#12345` - Number of iterations
- `cov: 342` - Code coverage (basic blocks)
- `ft: 1234` - Coverage features
- `corp: 56` - Corpus size (unique inputs discovered)

No crashes after many iterations = PASS.

### Failure Output

```
==12345==ERROR: libFuzzer: deadly signal
SUMMARY: libFuzzer: deadly-signal
```

A crash was found. The failing input is saved to `fuzz/artifacts/`.

## Reproducing and Minimizing Crashes

### Reproduce a Crash

```bash
cargo fuzz run fuzz_decode_client_message fuzz/artifacts/fuzz_decode_client_message/crash-abc123
```

### Minimize Crash Input

Produce the smallest input that still crashes:

```bash
cargo fuzz tmin fuzz_decode_client_message fuzz/artifacts/fuzz_decode_client_message/crash-abc123
```

## Coverage Report

Generate a coverage report:

```bash
cargo fuzz coverage fuzz_decode_client_message

# View the report
llvm-cov show fuzz/target/x86_64-unknown-linux-gnu/coverage/fuzz_decode_client_message \
    --instr-profile=fuzz/coverage/fuzz_decode_client_message/coverage.profdata
```

## Expected Behavior

All fuzz targets should:

1. **Never panic** - Invalid input must return `Err`, not panic
2. **Never hang** - Decode operations should be O(n) or better
3. **Never OOM** - Size limits prevent allocation attacks

If any of these conditions are violated, it indicates a security vulnerability that must be fixed.

## Adding New Fuzz Targets

1. Create a new file in `fuzz/fuzz_targets/`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = your_parse_function(data);
});
```

2. Add to `fuzz/Cargo.toml`:

```toml
[[bin]]
name = "fuzz_your_target"
path = "fuzz_targets/fuzz_your_target.rs"
test = false
doc = false
bench = false
```

3. (Optional) Add seed corpus to `fuzz/corpus/your_target/`

## Troubleshooting

### "cargo-fuzz not found"

```bash
cargo install cargo-fuzz
```

### "requires nightly"

cargo-fuzz automatically uses nightly. If you have issues:

```bash
rustup install nightly
cargo +nightly fuzz run fuzz_decode_client_message
```

### OOM during fuzzing

Increase memory limit:

```bash
cargo fuzz run fuzz_decode_client_message -- -rss_limit_mb=4096
```

### Fuzzer timeout (not a bug)

If libFuzzer itself times out (not the decode function), increase timeout:

```bash
cargo fuzz run fuzz_decode_client_message -- -max_total_time=7200
```
