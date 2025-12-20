#!/usr/bin/env bash
# Package Plix client for Linux
#
# Creates a distributable tar.gz archive containing:
# - plix-client binary
# - build_info.json
# - assets/ directory
# - cef/ directory (if CEF UI enabled)
#
# Usage:
#   ./client_linux.sh --binary-path <path> --version <semver> --output-dir <path> [--assets-dir <path>] [--cef-dir <path>]
#
# Exit codes:
#   0 - Success
#   1 - Missing required argument
#   2 - Binary not found
#   3 - Assets not found
#   4 - Archive creation failed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default values
BINARY_PATH=""
VERSION=""
OUTPUT_DIR=""
ASSETS_DIR="./assets"
CEF_DIR=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --binary-path)
            BINARY_PATH="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --assets-dir)
            ASSETS_DIR="$2"
            shift 2
            ;;
        --cef-dir)
            CEF_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 --binary-path <path> --version <semver> --output-dir <path>"
            echo ""
            echo "Package Plix client for Linux"
            echo ""
            echo "Required arguments:"
            echo "  --binary-path  Path to compiled plix-client binary"
            echo "  --version      Version string (semver, e.g., 0.1.0)"
            echo "  --output-dir   Output directory for archive"
            echo ""
            echo "Optional arguments:"
            echo "  --assets-dir   Path to assets directory (default: ./assets)"
            echo "  --cef-dir      Path to CEF runtime directory (optional)"
            echo "  -h, --help     Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$BINARY_PATH" ]]; then
    echo "Error: --binary-path is required" >&2
    exit 1
fi

if [[ -z "$VERSION" ]]; then
    echo "Error: --version is required" >&2
    exit 1
fi

if [[ -z "$OUTPUT_DIR" ]]; then
    echo "Error: --output-dir is required" >&2
    exit 1
fi

# Validate binary exists
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Error: Binary not found: $BINARY_PATH" >&2
    exit 2
fi

# Validate assets exist
if [[ ! -d "$ASSETS_DIR" ]]; then
    echo "Error: Assets directory not found: $ASSETS_DIR" >&2
    exit 3
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Bundle naming
PLATFORM="linux-x86_64"
BUNDLE_NAME="plix-client-${PLATFORM}-${VERSION}"
BUNDLE_DIR="$OUTPUT_DIR/$BUNDLE_NAME"
ARCHIVE_PATH="$OUTPUT_DIR/${BUNDLE_NAME}.tar.gz"

echo "Packaging plix-client for Linux..." >&2
echo "  Version: $VERSION" >&2
echo "  Binary: $BINARY_PATH" >&2
echo "  Assets: $ASSETS_DIR" >&2
echo "  Output: $ARCHIVE_PATH" >&2

# Clean up any existing bundle directory
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Copy binary
echo "Copying binary..." >&2
cp "$BINARY_PATH" "$BUNDLE_DIR/plix-client"
chmod +x "$BUNDLE_DIR/plix-client"

# Copy assets
echo "Copying assets..." >&2
cp -r "$ASSETS_DIR" "$BUNDLE_DIR/assets"

# Generate build_info.json
echo "Generating build_info.json..." >&2
"$SCRIPT_DIR/generate_build_info.sh" \
    --binary-path "$BUNDLE_DIR/plix-client" \
    --output "$BUNDLE_DIR/build_info.json" || {
    # If script fails, create minimal build_info.json
    cat > "$BUNDLE_DIR/build_info.json" << EOF
{
  "version": "$VERSION",
  "commit_sha": "unknown",
  "commit_sha_short": "unknown",
  "build_timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "build_date": "$(date -u +"%Y-%m-%d")",
  "target_triple": "x86_64-unknown-linux-gnu",
  "rust_version": "unknown",
  "branch": "unknown",
  "is_dirty": false
}
EOF
}

# Copy CEF runtime if specified
if [[ -n "$CEF_DIR" && -d "$CEF_DIR" ]]; then
    echo "Copying CEF runtime..." >&2
    mkdir -p "$BUNDLE_DIR/cef"

    # Copy essential CEF files for Linux
    # These are the core files needed for CEF to function
    for file in libcef.so icudtl.dat resources.pak chrome_100_percent.pak chrome_200_percent.pak; do
        if [[ -f "$CEF_DIR/$file" ]]; then
            cp "$CEF_DIR/$file" "$BUNDLE_DIR/cef/"
        elif [[ -f "$CEF_DIR/Release/$file" ]]; then
            cp "$CEF_DIR/Release/$file" "$BUNDLE_DIR/cef/"
        fi
    done

    # Copy locales
    if [[ -d "$CEF_DIR/locales" ]]; then
        cp -r "$CEF_DIR/locales" "$BUNDLE_DIR/cef/"
    elif [[ -d "$CEF_DIR/Resources/locales" ]]; then
        cp -r "$CEF_DIR/Resources/locales" "$BUNDLE_DIR/cef/"
    fi

    # Copy subprocess helper if present
    if [[ -f "$CEF_DIR/chrome-sandbox" ]]; then
        cp "$CEF_DIR/chrome-sandbox" "$BUNDLE_DIR/cef/"
    fi
fi

# Create optional config directory
mkdir -p "$BUNDLE_DIR/config/defaults"

# Validate bundle before archiving
echo "Validating bundle..." >&2
if [[ ! -x "$BUNDLE_DIR/plix-client" ]]; then
    echo "Error: Binary not executable in bundle" >&2
    exit 4
fi
if [[ ! -f "$BUNDLE_DIR/build_info.json" ]]; then
    echo "Error: build_info.json missing from bundle" >&2
    exit 4
fi
if [[ ! -d "$BUNDLE_DIR/assets" ]]; then
    echo "Error: assets directory missing from bundle" >&2
    exit 4
fi

# Create archive
echo "Creating archive..." >&2
cd "$OUTPUT_DIR"
tar -czf "$(basename "$ARCHIVE_PATH")" "$BUNDLE_NAME" || {
    echo "Error: Failed to create archive" >&2
    exit 4
}

# Generate checksum
echo "Generating checksum..." >&2
sha256sum "$(basename "$ARCHIVE_PATH")" > "$(basename "$ARCHIVE_PATH").sha256"

# Clean up bundle directory
rm -rf "$BUNDLE_DIR"

echo "Package created successfully!" >&2
echo "  Archive: $ARCHIVE_PATH" >&2
echo "  Checksum: $ARCHIVE_PATH.sha256" >&2

# Output archive path to stdout (for CI consumption)
echo "$ARCHIVE_PATH"
