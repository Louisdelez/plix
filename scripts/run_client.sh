#!/bin/bash
# Run Plix client with optional arguments
#
# Usage: ./scripts/run_client.sh [OPTIONS]
#
# Options are passed directly to plix-client.
# Run with --help to see available options.

set -e

cd "$(dirname "$0")/.."

# Build in release mode if not already built
if [ ! -f "target/release/plix-client" ]; then
    echo "Building plix-client..."
    cargo build --release -p plix-client
fi

# Run the client
exec ./target/release/plix-client "$@"
