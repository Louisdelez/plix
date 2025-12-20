#!/usr/bin/env bash
# Plix Server Launcher
#
# This script provides a convenient way to start the Plix headless server
# with proper environment setup and logging.
#
# Usage:
#   ./run_server.sh [config_file]
#   ./run_server.sh --help
#
# If no config file is specified, looks for server.toml in common locations.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Show help
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    echo "Plix Server Launcher"
    echo ""
    echo "Usage: $0 [config_file]"
    echo ""
    echo "Arguments:"
    echo "  config_file    Path to server.toml config file (optional)"
    echo ""
    echo "Config file search order (if not specified):"
    echo "  1. ./server.toml"
    echo "  2. ~/.config/plix/server.toml"
    echo "  3. /etc/plix/server.toml"
    echo "  4. Built-in defaults"
    echo ""
    echo "Environment Variables:"
    echo "  PLIX_CONFIG     Override config file path"
    echo "  PLIX_LOG_LEVEL  Override log level (trace, debug, info, warn, error)"
    echo "  PLIX_PORT       Override server port"
    exit 0
fi

# Find binary
BINARY=""
for candidate in \
    "$SCRIPT_DIR/plix-server-headless" \
    "$SCRIPT_DIR/../plix-server-headless" \
    "/usr/local/bin/plix-server-headless" \
    "./plix-server-headless"; do
    if [[ -x "$candidate" ]]; then
        BINARY="$candidate"
        break
    fi
done

if [[ -z "$BINARY" ]]; then
    echo "Error: plix-server-headless binary not found" >&2
    echo "Please ensure the binary is in the same directory as this script" >&2
    exit 1
fi

# Find config file
CONFIG="${PLIX_CONFIG:-${1:-}}"
if [[ -z "$CONFIG" ]]; then
    for candidate in \
        "./server.toml" \
        "$HOME/.config/plix/server.toml" \
        "/etc/plix/server.toml"; do
        if [[ -f "$candidate" ]]; then
            CONFIG="$candidate"
            echo "Using config: $CONFIG"
            break
        fi
    done
fi

# Build command line arguments
ARGS=()

if [[ -n "$CONFIG" && -f "$CONFIG" ]]; then
    ARGS+=("--config" "$CONFIG")
elif [[ -n "$CONFIG" ]]; then
    echo "Warning: Config file not found: $CONFIG" >&2
    echo "Starting with default settings..." >&2
fi

# Apply environment variable overrides
if [[ -n "${PLIX_LOG_LEVEL:-}" ]]; then
    ARGS+=("--log-level" "$PLIX_LOG_LEVEL")
fi

if [[ -n "${PLIX_PORT:-}" ]]; then
    ARGS+=("--port" "$PLIX_PORT")
fi

# Print startup info
echo "Starting Plix Server..."
echo "  Binary: $BINARY"
echo "  Config: ${CONFIG:-default}"
echo "  Args: ${ARGS[*]:-none}"
echo ""

# Execute the server
exec "$BINARY" "${ARGS[@]}"
