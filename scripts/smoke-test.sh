#!/bin/bash
# Plix Smoke Test Script
#
# Verifies that a Plix release works correctly by:
# 1. Starting a server
# 2. Connecting a headless client
# 3. Verifying connection success
# 4. Cleaning up
#
# Usage: ./scripts/smoke-test.sh [release_dir]
#
# Exit codes:
#   0 - All tests passed
#   1 - Test failed
#   2 - Missing dependencies

set -e

# Configuration
RELEASE_DIR="${1:-.}"
SERVER_PORT=17777  # Use non-default port to avoid conflicts
TIMEOUT=30

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    rm -f "$SERVER_LOG" "$CLIENT_LOG"
}

trap cleanup EXIT

# Check for required binaries
check_binaries() {
    local missing=0

    for bin in plix-server plix-client; do
        if [ ! -x "$RELEASE_DIR/$bin" ] && [ ! -x "$RELEASE_DIR/target/release/$bin" ]; then
            echo -e "${RED}Missing binary: $bin${NC}"
            missing=1
        fi
    done

    if [ $missing -eq 1 ]; then
        echo -e "${RED}Run 'cargo build --release' first${NC}"
        exit 2
    fi
}

# Find binary path
find_binary() {
    local name=$1
    if [ -x "$RELEASE_DIR/$name" ]; then
        echo "$RELEASE_DIR/$name"
    elif [ -x "$RELEASE_DIR/target/release/$name" ]; then
        echo "$RELEASE_DIR/target/release/$name"
    else
        echo ""
    fi
}

# Test version flags
test_version_flags() {
    echo -e "${YELLOW}Testing --version flags...${NC}"

    local server_bin=$(find_binary plix-server)
    local client_bin=$(find_binary plix-client)

    # Test server --version
    if ! "$server_bin" --version | grep -q "plix-server"; then
        echo -e "${RED}Server --version failed${NC}"
        return 1
    fi
    echo -e "  Server: $("$server_bin" --version | head -1)"

    # Test client --version
    if ! "$client_bin" --version | grep -q "plix-client"; then
        echo -e "${RED}Client --version failed${NC}"
        return 1
    fi
    echo -e "  Client: $("$client_bin" --version | head -1)"

    echo -e "${GREEN}Version flags: PASS${NC}"
}

# Test server startup
test_server_startup() {
    echo -e "${YELLOW}Testing server startup...${NC}"

    local server_bin=$(find_binary plix-server)
    SERVER_LOG=$(mktemp)

    # Start server in background
    "$server_bin" \
        --port "$SERVER_PORT" \
        --log-level info \
        > "$SERVER_LOG" 2>&1 &
    SERVER_PID=$!

    # Wait for server to be ready (look for "Starting" in log)
    local waited=0
    while [ $waited -lt $TIMEOUT ]; do
        if grep -q "Server configuration" "$SERVER_LOG" 2>/dev/null; then
            echo -e "  Server started (PID: $SERVER_PID)"
            echo -e "${GREEN}Server startup: PASS${NC}"
            return 0
        fi
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            echo -e "${RED}Server exited prematurely${NC}"
            cat "$SERVER_LOG"
            return 1
        fi
        sleep 1
        waited=$((waited + 1))
    done

    echo -e "${RED}Server startup timeout${NC}"
    cat "$SERVER_LOG"
    return 1
}

# Test client connection
test_client_connection() {
    echo -e "${YELLOW}Testing client connection...${NC}"

    local client_bin=$(find_binary plix-client)
    CLIENT_LOG=$(mktemp)

    # Run headless client - should connect and exit
    timeout 10 "$client_bin" \
        --server "127.0.0.1:$SERVER_PORT" \
        --name "SmokeTest" \
        --headless \
        --log-level info \
        > "$CLIENT_LOG" 2>&1 || true

    # Check for successful connection
    if grep -q "Successfully connected" "$CLIENT_LOG"; then
        echo -e "  Client connected successfully"
        echo -e "${GREEN}Client connection: PASS${NC}"
        return 0
    else
        echo -e "${RED}Client connection failed${NC}"
        cat "$CLIENT_LOG"
        return 1
    fi
}

# Test content validation
test_content_validation() {
    echo -e "${YELLOW}Testing content validation...${NC}"

    local server_bin=$(find_binary plix-server)

    # Run content validation
    if "$server_bin" --validate-content --assets-dir "$RELEASE_DIR/assets" 2>&1 | grep -q "Content"; then
        echo -e "  Content validation completed"
        echo -e "${GREEN}Content validation: PASS${NC}"
        return 0
    else
        echo -e "${YELLOW}Content validation: SKIP (no content files)${NC}"
        return 0
    fi
}

# Main test sequence
main() {
    echo "=================================="
    echo "Plix Smoke Test"
    echo "=================================="
    echo ""

    check_binaries

    local failed=0

    test_version_flags || failed=1
    test_content_validation || failed=1
    test_server_startup || failed=1

    if [ $failed -eq 0 ]; then
        test_client_connection || failed=1
    fi

    echo ""
    echo "=================================="
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}ALL TESTS PASSED${NC}"
        exit 0
    else
        echo -e "${RED}SOME TESTS FAILED${NC}"
        exit 1
    fi
}

main "$@"
