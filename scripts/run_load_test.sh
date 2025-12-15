#!/bin/bash
# T078 [US2] - Load test runner script

# Ensure cargo is in PATH
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
#
# Usage:
#   ./scripts/run_load_test.sh [BOTS] [DURATION] [SERVER]
#
# Arguments:
#   BOTS      Number of bots to spawn (default: 8)
#   DURATION  Test duration in seconds (default: 60)
#   SERVER    Server address (default: 127.0.0.1:7777)
#
# Example:
#   ./scripts/run_load_test.sh 16 120 192.168.1.100:7777

set -e

# Parse arguments
BOTS=${1:-8}
DURATION=${2:-60}
SERVER=${3:-127.0.0.1:7777}

echo "==================================="
echo "Plix Load Test"
echo "==================================="
echo "Server:   $SERVER"
echo "Bots:     $BOTS"
echo "Duration: ${DURATION}s"
echo "==================================="
echo ""

# Parse port from SERVER address
PORT=$(echo $SERVER | cut -d':' -f2)

# Check if server is running
if ! nc -z -w1 $(echo $SERVER | tr ':' ' ') 2>/dev/null; then
    echo "Warning: Server at $SERVER may not be running"
    echo "Starting server..."
    cargo run --release --bin plix-server -- --port $PORT &
    SERVER_PID=$!
    sleep 2
    echo "Server started (PID: $SERVER_PID)"
    STARTED_SERVER=true
else
    echo "Server is running at $SERVER"
    STARTED_SERVER=false
fi

echo ""
echo "Starting load test..."
echo ""

# Run the load test
cargo run --release --bin plix-tools -- \
    --log-level info \
    bot \
    --server "$SERVER" \
    --count "$BOTS" \
    --duration "$DURATION"

EXIT_CODE=$?

# Clean up if we started the server
if [ "$STARTED_SERVER" = true ] && [ -n "$SERVER_PID" ]; then
    echo ""
    echo "Stopping server..."
    kill $SERVER_PID 2>/dev/null || true
fi

echo ""
echo "==================================="
echo "Load test complete"
echo "==================================="

exit $EXIT_CODE
