#!/bin/bash
# Build script for {{mod_name}}
# Compiles the mod to WebAssembly

set -e

# Check for wasm target
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "Error: wasm32-unknown-unknown target not installed"
    echo "Run: rustup target add wasm32-unknown-unknown"
    exit 1
fi

# Build in release mode by default
MODE="${1:-release}"

if [ "$MODE" = "release" ]; then
    cargo build --target wasm32-unknown-unknown --release
    echo "Built: target/wasm32-unknown-unknown/release/{{mod_id}}.wasm"
else
    cargo build --target wasm32-unknown-unknown
    echo "Built: target/wasm32-unknown-unknown/debug/{{mod_id}}.wasm"
fi
