#!/usr/bin/env bash
# Generate build_info.json from a Plix binary
#
# This script extracts build information from a compiled binary
# and outputs it as JSON for bundle inclusion.
#
# Usage:
#   ./generate_build_info.sh --binary-path <path> --output <path>
#
# Exit codes:
#   0 - Success
#   1 - Missing required argument
#   2 - Binary not found or not executable
#   3 - Failed to execute binary

set -euo pipefail

# Default values
BINARY_PATH=""
OUTPUT_PATH=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --binary-path)
            BINARY_PATH="$2"
            shift 2
            ;;
        --output)
            OUTPUT_PATH="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 --binary-path <path> --output <path>"
            echo ""
            echo "Generate build_info.json from a Plix binary"
            echo ""
            echo "Options:"
            echo "  --binary-path  Path to the compiled binary (required)"
            echo "  --output       Output path for build_info.json (required)"
            echo "  -h, --help     Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Validate arguments
if [[ -z "$BINARY_PATH" ]]; then
    echo "Error: --binary-path is required" >&2
    exit 1
fi

if [[ -z "$OUTPUT_PATH" ]]; then
    echo "Error: --output is required" >&2
    exit 1
fi

# Check binary exists and is executable
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Error: Binary not found: $BINARY_PATH" >&2
    exit 2
fi

if [[ ! -x "$BINARY_PATH" ]]; then
    echo "Error: Binary is not executable: $BINARY_PATH" >&2
    exit 2
fi

echo "Extracting build info from: $BINARY_PATH" >&2

# Try to get version info from binary
VERSION_OUTPUT=$("$BINARY_PATH" --version 2>&1) || {
    echo "Error: Failed to execute binary with --version" >&2
    exit 3
}

# Parse version output
# Expected format: "plix-server 0.1.0 (abc1234) built 2025-12-19"
# or simpler: "plix-server 0.1.0"
VERSION=$(echo "$VERSION_OUTPUT" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "unknown")
COMMIT_SHORT=$(echo "$VERSION_OUTPUT" | grep -oE '\([a-f0-9]+\)' | tr -d '()' || echo "unknown")
BUILD_DATE=$(echo "$VERSION_OUTPUT" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || echo "unknown")

# Get additional info from git if available
if command -v git &> /dev/null && git rev-parse --git-dir > /dev/null 2>&1; then
    COMMIT_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    COMMIT_SHORT=${COMMIT_SHORT:-$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")}
    BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
    IS_DIRTY=$(git diff --quiet && echo "false" || echo "true")
else
    COMMIT_SHA="${COMMIT_SHORT}0000000000000000000000000000000000"
    BRANCH="unknown"
    IS_DIRTY="false"
fi

# Detect target triple
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)   TARGET_TRIPLE="x86_64-unknown-linux-gnu" ;;
    Linux-aarch64)  TARGET_TRIPLE="aarch64-unknown-linux-gnu" ;;
    Darwin-x86_64)  TARGET_TRIPLE="x86_64-apple-darwin" ;;
    Darwin-arm64)   TARGET_TRIPLE="aarch64-apple-darwin" ;;
    MINGW*|CYGWIN*|MSYS*) TARGET_TRIPLE="x86_64-pc-windows-msvc" ;;
    *)              TARGET_TRIPLE="unknown" ;;
esac

# Get Rust version if available
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
else
    RUST_VERSION="unknown"
fi

# Build timestamp
BUILD_TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Create JSON output
cat > "$OUTPUT_PATH" << EOF
{
  "version": "$VERSION",
  "commit_sha": "$COMMIT_SHA",
  "commit_sha_short": "$COMMIT_SHORT",
  "build_timestamp": "$BUILD_TIMESTAMP",
  "build_date": "$BUILD_DATE",
  "target_triple": "$TARGET_TRIPLE",
  "rust_version": "$RUST_VERSION",
  "branch": "$BRANCH",
  "is_dirty": $IS_DIRTY
}
EOF

echo "Generated: $OUTPUT_PATH" >&2
echo "$OUTPUT_PATH"
