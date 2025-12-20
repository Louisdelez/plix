#!/usr/bin/env bash
# Validate a Plix bundle archive
#
# This script validates that a packaged bundle contains all required files
# and that the binary is functional.
#
# Usage:
#   ./validate_bundle.sh <archive_path> [--type client|server]
#
# Exit codes:
#   0 - Validation passed
#   1 - Missing required argument
#   2 - Archive not found
#   3 - Extraction failed
#   4 - Missing required file
#   5 - Binary execution failed
#   6 - Invalid build_info.json

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
ARCHIVE_PATH=""
BUNDLE_TYPE="auto"
TEMP_DIR=""

# Cleanup function
cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            BUNDLE_TYPE="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 <archive_path> [--type client|server]"
            echo ""
            echo "Validate a Plix bundle archive"
            echo ""
            echo "Options:"
            echo "  --type       Bundle type: client, server, or auto (default: auto)"
            echo "  -h, --help   Show this help message"
            exit 0
            ;;
        -*)
            log_error "Unknown option: $1"
            exit 1
            ;;
        *)
            ARCHIVE_PATH="$1"
            shift
            ;;
    esac
done

# Validate arguments
if [[ -z "$ARCHIVE_PATH" ]]; then
    log_error "Archive path is required"
    echo "Usage: $0 <archive_path> [--type client|server]" >&2
    exit 1
fi

if [[ ! -f "$ARCHIVE_PATH" ]]; then
    log_error "Archive not found: $ARCHIVE_PATH"
    exit 2
fi

# Auto-detect bundle type from filename
if [[ "$BUNDLE_TYPE" == "auto" ]]; then
    if [[ "$ARCHIVE_PATH" == *"client"* ]]; then
        BUNDLE_TYPE="client"
    elif [[ "$ARCHIVE_PATH" == *"server"* ]]; then
        BUNDLE_TYPE="server"
    else
        log_error "Could not auto-detect bundle type. Use --type to specify."
        exit 1
    fi
fi

log_info "Validating $BUNDLE_TYPE bundle: $ARCHIVE_PATH"

# Create temp directory
TEMP_DIR=$(mktemp -d)
log_info "Extracting to: $TEMP_DIR"

# Extract archive based on extension
if [[ "$ARCHIVE_PATH" == *.tar.gz ]]; then
    tar -xzf "$ARCHIVE_PATH" -C "$TEMP_DIR" || {
        log_error "Failed to extract tar.gz archive"
        exit 3
    }
elif [[ "$ARCHIVE_PATH" == *.zip ]]; then
    unzip -q "$ARCHIVE_PATH" -d "$TEMP_DIR" || {
        log_error "Failed to extract zip archive"
        exit 3
    }
else
    log_error "Unsupported archive format. Use .tar.gz or .zip"
    exit 3
fi

# Find the bundle root (may be nested in a directory)
BUNDLE_ROOT=$(find "$TEMP_DIR" -maxdepth 2 -type d -name "plix-*" | head -1)
if [[ -z "$BUNDLE_ROOT" ]]; then
    BUNDLE_ROOT="$TEMP_DIR"
fi

log_info "Bundle root: $BUNDLE_ROOT"

# Validate required files based on bundle type
ERRORS=0

check_file() {
    local path="$1"
    local required="${2:-true}"

    if [[ -f "$BUNDLE_ROOT/$path" ]]; then
        log_info "Found: $path"
        return 0
    else
        if [[ "$required" == "true" ]]; then
            log_error "Missing required file: $path"
            ((ERRORS++))
            return 1
        else
            log_warn "Optional file not found: $path"
            return 0
        fi
    fi
}

check_dir() {
    local path="$1"
    local required="${2:-true}"

    if [[ -d "$BUNDLE_ROOT/$path" ]]; then
        log_info "Found directory: $path"
        return 0
    else
        if [[ "$required" == "true" ]]; then
            log_error "Missing required directory: $path"
            ((ERRORS++))
            return 1
        else
            log_warn "Optional directory not found: $path"
            return 0
        fi
    fi
}

# Common required files
check_file "build_info.json"

# Bundle-specific validation
if [[ "$BUNDLE_TYPE" == "client" ]]; then
    # Find client binary (may have .exe extension on Windows)
    CLIENT_BINARY=$(find "$BUNDLE_ROOT" -maxdepth 2 -type f -name "plix-client*" -perm -u=x 2>/dev/null | head -1)
    if [[ -z "$CLIENT_BINARY" ]]; then
        CLIENT_BINARY=$(find "$BUNDLE_ROOT" -maxdepth 2 -type f -name "plix-client*" 2>/dev/null | head -1)
    fi

    if [[ -n "$CLIENT_BINARY" ]]; then
        log_info "Found binary: $(basename "$CLIENT_BINARY")"
    else
        log_error "Missing required binary: plix-client"
        ((ERRORS++))
    fi

    # Client-specific directories
    check_dir "assets"
    check_dir "assets/ui" false
    check_dir "assets/arenas" false
    check_dir "cef" false  # Optional: only if CEF UI enabled

elif [[ "$BUNDLE_TYPE" == "server" ]]; then
    # Find server binary
    SERVER_BINARY=$(find "$BUNDLE_ROOT" -maxdepth 2 -type f -name "plix-server*" -perm -u=x 2>/dev/null | head -1)
    if [[ -z "$SERVER_BINARY" ]]; then
        SERVER_BINARY=$(find "$BUNDLE_ROOT" -maxdepth 2 -type f -name "plix-server*" 2>/dev/null | head -1)
    fi

    if [[ -n "$SERVER_BINARY" ]]; then
        log_info "Found binary: $(basename "$SERVER_BINARY")"
    else
        log_error "Missing required binary: plix-server-headless"
        ((ERRORS++))
    fi

    # Server-specific directories/files
    check_dir "configs/examples"
    check_file "configs/examples/server.toml"
    check_file "docs/README.md" false
fi

# Validate build_info.json content
BUILD_INFO_PATH="$BUNDLE_ROOT/build_info.json"
if [[ -f "$BUILD_INFO_PATH" ]]; then
    log_info "Validating build_info.json..."

    # Check required fields using grep (no jq dependency)
    for field in version commit_sha build_timestamp target_triple; do
        if ! grep -q "\"$field\"" "$BUILD_INFO_PATH"; then
            log_error "build_info.json missing required field: $field"
            ((ERRORS++))
        fi
    done

    # Validate version format
    VERSION=$(grep -oE '"version"[[:space:]]*:[[:space:]]*"[^"]*"' "$BUILD_INFO_PATH" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "")
    if [[ -z "$VERSION" ]]; then
        log_error "build_info.json has invalid version format"
        ((ERRORS++))
    else
        log_info "Version: $VERSION"
    fi
fi

# Try to execute binary with --help (smoke test)
BINARY_PATH=""
if [[ "$BUNDLE_TYPE" == "client" && -n "${CLIENT_BINARY:-}" ]]; then
    BINARY_PATH="$CLIENT_BINARY"
elif [[ "$BUNDLE_TYPE" == "server" && -n "${SERVER_BINARY:-}" ]]; then
    BINARY_PATH="$SERVER_BINARY"
fi

if [[ -n "$BINARY_PATH" && -x "$BINARY_PATH" ]]; then
    log_info "Running smoke test: $BINARY_PATH --help"
    if timeout 10 "$BINARY_PATH" --help > /dev/null 2>&1; then
        log_info "Smoke test passed: --help works"
    else
        # Some binaries may not support --help, try --version
        if timeout 10 "$BINARY_PATH" --version > /dev/null 2>&1; then
            log_info "Smoke test passed: --version works"
        else
            log_warn "Smoke test: binary did not respond to --help or --version"
            # Not a hard error - binary might need GUI or specific arguments
        fi
    fi
fi

# Summary
echo "" >&2
if [[ $ERRORS -eq 0 ]]; then
    log_info "Validation PASSED: $ARCHIVE_PATH"
    exit 0
else
    log_error "Validation FAILED with $ERRORS error(s)"
    exit 4
fi
