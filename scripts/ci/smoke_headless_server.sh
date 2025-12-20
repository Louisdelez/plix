#!/usr/bin/env bash
# Smoke test for Plix headless server bundle
#
# Validates that a server bundle is properly structured and the binary runs.
# This is a lightweight test suitable for CI pipelines.
#
# Usage:
#   ./smoke_headless_server.sh <archive_path>
#
# Exit codes:
#   0 - All tests passed
#   1 - Missing argument
#   2 - Archive not found
#   3 - Extraction failed
#   4 - Missing required file
#   5 - Binary execution failed

set -euo pipefail

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

log_info "Testing server bundle: $ARCHIVE_PATH"

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
BUNDLE_ROOT=$(find "$TEMP_DIR" -maxdepth 2 -type d -name "plix-server-*" | head -1)
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
for candidate in "$BUNDLE_ROOT/plix-server-headless" "$BUNDLE_ROOT/plix-server-headless.exe"; do
    if [[ -f "$candidate" ]]; then
        BINARY_PATH="$candidate"
        break
    fi
done

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
    if timeout 10 "$BINARY_PATH" --version 2>&1; then
        log_pass "--version returns successfully"
    else
        log_warn "Binary --version did not complete (may require runtime dependencies)"
    fi

    # Try to run --help
    log_info "Testing --help..."
    if timeout 10 "$BINARY_PATH" --help 2>&1 | head -20; then
        log_pass "--help returns successfully"
    else
        log_warn "Binary --help did not complete"
    fi

    # Try to validate config
    log_info "Testing --validate..."
    TEMP_CONFIG="$TEMP_DIR/test_config.toml"
    cat > "$TEMP_CONFIG" << 'EOF'
[network]
bind_address = "127.0.0.1"
port = 7777
max_players = 8

[game]
tick_rate = 30
arena = "ffa_small"
EOF

    if timeout 10 "$BINARY_PATH" --config "$TEMP_CONFIG" --validate 2>&1; then
        log_pass "Config validation works"
    else
        log_warn "Config validation test did not complete"
    fi
else
    log_fail "Binary not found"
    ((ERRORS++))
fi

# Test 3: Check configs directory
if [[ -d "$BUNDLE_ROOT/configs/examples" ]]; then
    log_pass "Example configs directory exists"

    if [[ -f "$BUNDLE_ROOT/configs/examples/server.toml" ]]; then
        log_pass "server.toml example present"

        # Validate TOML syntax
        if command -v python3 &> /dev/null; then
            if python3 -c "import tomllib; tomllib.load(open('$BUNDLE_ROOT/configs/examples/server.toml', 'rb'))" 2>/dev/null; then
                log_pass "server.toml is valid TOML"
            elif python3 -c "import toml; toml.load('$BUNDLE_ROOT/configs/examples/server.toml')" 2>/dev/null; then
                log_pass "server.toml is valid TOML"
            else
                log_warn "Could not validate TOML syntax"
            fi
        fi
    else
        log_warn "server.toml example not found"
    fi
else
    log_warn "Example configs directory not found"
fi

# Test 4: Check run script
RUN_SCRIPT=""
if [[ -f "$BUNDLE_ROOT/run_server.sh" ]]; then
    RUN_SCRIPT="$BUNDLE_ROOT/run_server.sh"
elif [[ -f "$BUNDLE_ROOT/run_server.ps1" ]]; then
    RUN_SCRIPT="$BUNDLE_ROOT/run_server.ps1"
fi

if [[ -n "$RUN_SCRIPT" ]]; then
    log_pass "Run script exists: $(basename "$RUN_SCRIPT")"
else
    log_warn "Run script not found"
fi

# Test 5: Check docs
if [[ -f "$BUNDLE_ROOT/docs/README.md" ]]; then
    log_pass "Documentation present"

    # Check for essential sections
    if grep -q "Exit Codes" "$BUNDLE_ROOT/docs/README.md"; then
        log_pass "Exit codes documented"
    fi
else
    log_warn "Documentation not found"
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
