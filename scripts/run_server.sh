#!/bin/bash
# Run Plix server with optional arguments
#
# Usage: ./scripts/run_server.sh [OPTIONS]
#
# Options are passed directly to plix-server.
# Run with --help to see available options.

set -e

cd "$(dirname "$0")/.."

# Build in release mode if not already built
if [ ! -f "target/release/plix-server" ]; then
    echo "Building plix-server..."
    cargo build --release -p plix-server
fi

# Run the server
exec ./target/release/plix-server "$@"
