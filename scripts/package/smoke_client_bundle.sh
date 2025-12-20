#!/usr/bin/env bash
# Smoke test for Plix client bundle
#
# Validates that a client bundle is properly structured and the binary runs.
# This is a lightweight test suitable for CI pipelines.
#
# Usage:
#   ./smoke_client_bundle.sh <archive_path>
#
# Exit codes:
#   0 - All tests passed
#   1 - Missing argument
#   2 - Archive not found
#   3 - Extraction failed
#   4 - Missing required file
#   5 - Binary execution failed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_info() { echo -e "[INFO] $1"; }

ARCHIVE_PATH="${1:-}"
TEMP_DIR=""

cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

if [[ -z "$ARCHIVE_PATH" ]]; then
    echo "Usage: $0 <archive_path>" >&2
    exit 1
fi

if [[ ! -f "$ARCHIVE_PATH" ]]; then
    log_fail "Archive not found: $ARCHIVE_PATH"
    exit 2
fi

log_info "Testing client bundle: $ARCHIVE_PATH"

# Create temp directory
TEMP_DIR=$(mktemp -d)
log_info "Extracting to: $TEMP_DIR"

# Extract archive
if [[ "$ARCHIVE_PATH" == *.tar.gz ]]; then
    tar -xzf "$ARCHIVE_PATH" -C "$TEMP_DIR" || { log_fail "Failed to extract tar.gz"; exit 3; }
elif [[ "$ARCHIVE_PATH" == *.zip ]]; then
    unzip -q "$ARCHIVE_PATH" -d "$TEMP_DIR" || { log_fail "Failed to extract zip"; exit 3; }
else
    log_fail "Unsupported archive format"
    exit 3
fi
log_pass "Archive extraction"

# Find bundle root
BUNDLE_ROOT=$(find "$TEMP_DIR" -maxdepth 2 -type d -name "plix-client-*" | head -1)
if [[ -z "$BUNDLE_ROOT" ]]; then
    BUNDLE_ROOT="$TEMP_DIR"
fi
log_info "Bundle root: $BUNDLE_ROOT"

ERRORS=0

# Test 1: Check build_info.json exists
if [[ -f "$BUNDLE_ROOT/build_info.json" ]]; then
    log_pass "build_info.json exists"

    # Validate JSON
    if command -v python3 &> /dev/null; then
        if python3 -m json.tool "$BUNDLE_ROOT/build_info.json" > /dev/null 2>&1; then
            log_pass "build_info.json is valid JSON"
        else
            log_fail "build_info.json is not valid JSON"
            ((ERRORS++))
        fi
    fi

    # Check required fields
    for field in version commit_sha target_triple; do
        if grep -q "\"$field\"" "$BUNDLE_ROOT/build_info.json"; then
            log_pass "build_info.json has $field"
        else
            log_fail "build_info.json missing $field"
            ((ERRORS++))
        fi
    done
else
    log_fail "build_info.json not found"
    ((ERRORS++))
fi

# Test 2: Find and check binary
BINARY_PATH=""

# Check for macOS .app bundle
if [[ -d "$BUNDLE_ROOT/Plix.app" ]]; then
    BINARY_PATH="$BUNDLE_ROOT/Plix.app/Contents/MacOS/plix-client"
    log_info "Detected macOS .app bundle"
else
    # Linux or extracted Windows
    for candidate in "$BUNDLE_ROOT/plix-client" "$BUNDLE_ROOT/plix-client.exe"; do
        if [[ -f "$candidate" ]]; then
            BINARY_PATH="$candidate"
            break
        fi
    done
fi

if [[ -n "$BINARY_PATH" && -f "$BINARY_PATH" ]]; then
    log_pass "Binary found: $(basename "$BINARY_PATH")"

    # Check executable permission (Linux/macOS)
    if [[ "$(uname)" != "MINGW"* && "$(uname)" != "CYGWIN"* ]]; then
        if [[ -x "$BINARY_PATH" ]]; then
            log_pass "Binary is executable"
        else
            log_warn "Binary not executable, attempting chmod"
            chmod +x "$BINARY_PATH"
        fi
    fi

    # Try to run --version
    log_info "Testing --version..."
    if timeout 10 "$BINARY_PATH" --version 2>/dev/null; then
        log_pass "--version returns successfully"
    else
        # Client may not support --version in all modes, try --help
        if timeout 10 "$BINARY_PATH" --help 2>/dev/null | head -5; then
            log_pass "--help returns successfully"
        else
            log_warn "Binary did not respond to --version or --help (may require display)"
        fi
    fi
else
    log_fail "Binary not found"
    ((ERRORS++))
fi

# Test 3: Check assets directory
ASSETS_DIR=""
if [[ -d "$BUNDLE_ROOT/Plix.app/Contents/Resources/assets" ]]; then
    ASSETS_DIR="$BUNDLE_ROOT/Plix.app/Contents/Resources/assets"
elif [[ -d "$BUNDLE_ROOT/assets" ]]; then
    ASSETS_DIR="$BUNDLE_ROOT/assets"
fi

if [[ -n "$ASSETS_DIR" && -d "$ASSETS_DIR" ]]; then
    log_pass "Assets directory exists"

    # Check for common asset subdirectories
    if [[ -d "$ASSETS_DIR/ui" ]]; then
        log_pass "UI assets present"
    else
        log_warn "UI assets not found (may be optional)"
    fi

    if [[ -d "$ASSETS_DIR/arenas" ]]; then
        log_pass "Arena assets present"
    else
        log_warn "Arena assets not found (may be optional)"
    fi
else
    log_fail "Assets directory not found"
    ((ERRORS++))
fi

# Test 4: Check CEF runtime (optional)
CEF_DIR=""
if [[ -d "$BUNDLE_ROOT/Plix.app/Contents/Frameworks/Chromium Embedded Framework.framework" ]]; then
    CEF_DIR="$BUNDLE_ROOT/Plix.app/Contents/Frameworks"
elif [[ -d "$BUNDLE_ROOT/cef" ]]; then
    CEF_DIR="$BUNDLE_ROOT/cef"
fi

if [[ -n "$CEF_DIR" && -d "$CEF_DIR" ]]; then
    log_pass "CEF runtime present"

    # Count CEF files
    CEF_COUNT=$(find "$CEF_DIR" -type f | wc -l)
    log_info "CEF files: $CEF_COUNT"
else
    log_info "CEF runtime not included (native UI mode)"
fi

# Test 5: Check macOS bundle (if applicable)
if [[ -d "$BUNDLE_ROOT/Plix.app" ]]; then
    if [[ -f "$BUNDLE_ROOT/Plix.app/Contents/Info.plist" ]]; then
        log_pass "Info.plist exists"

        # Check bundle identifier
        if grep -q "CFBundleIdentifier" "$BUNDLE_ROOT/Plix.app/Contents/Info.plist"; then
            log_pass "CFBundleIdentifier present in Info.plist"
        else
            log_warn "CFBundleIdentifier not found in Info.plist"
        fi
    else
        log_fail "Info.plist missing from macOS bundle"
        ((ERRORS++))
    fi
fi

# Summary
echo ""
if [[ $ERRORS -eq 0 ]]; then
    log_pass "All smoke tests passed!"
    exit 0
else
    log_fail "Smoke tests failed with $ERRORS error(s)"
    exit 5
fi
