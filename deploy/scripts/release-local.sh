#!/usr/bin/env bash
# =============================================================================
# Plix Server Local Release Script
# Feature 028: Dedicated Server Packaging
# =============================================================================
# Creates a self-contained release archive for non-Docker deployment
# =============================================================================
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory (for relative paths)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}/../.."

# Default values
OUTPUT_DIR="${PROJECT_ROOT}/release"
VERSION=""
TARGET="release"
SKIP_BUILD=false

# Help message
show_help() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Creates a release archive for plix-server (non-Docker deployment).

Options:
  --version VERSION    Version string (default: git describe or 'dev')
  --output DIR         Output directory (default: ./release)
  --debug              Build debug instead of release
  --skip-build         Skip cargo build (use existing binary)
  -h, --help           Show this help message

Output:
  plix-server-VERSION-PLATFORM.tar.gz
  plix-server-VERSION-PLATFORM.tar.gz.sha256

Contents:
  - plix-server binary
  - assets/arenas/ (arena definitions)
  - config/server.toml.example
  - README.txt (quick start guide)

Examples:
  $(basename "$0")                           # Build release archive
  $(basename "$0") --version 1.0.0           # Build with specific version
  $(basename "$0") --skip-build              # Package existing binary
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --debug)
            TARGET="debug"
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Detect platform
detect_platform() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$arch" in
        x86_64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) arch="unknown" ;;
    esac

    echo "${os}-${arch}"
}

PLATFORM=$(detect_platform)

# Determine version
if [[ -z "$VERSION" ]]; then
    cd "$PROJECT_ROOT"
    if git describe --tags --always 2>/dev/null; then
        VERSION=$(git describe --tags --always 2>/dev/null)
    else
        VERSION="dev"
    fi
fi

# Archive name
ARCHIVE_NAME="plix-server-${VERSION}-${PLATFORM}"
ARCHIVE_FILE="${OUTPUT_DIR}/${ARCHIVE_NAME}.tar.gz"
CHECKSUM_FILE="${OUTPUT_DIR}/${ARCHIVE_NAME}.tar.gz.sha256"

echo -e "${BLUE}=== Plix Server Release Build ===${NC}"
echo -e "  Version:  ${VERSION}"
echo -e "  Platform: ${PLATFORM}"
echo -e "  Target:   ${TARGET}"
echo -e "  Output:   ${OUTPUT_DIR}"
echo ""

# Build binary
if [[ "$SKIP_BUILD" == "false" ]]; then
    echo -e "${GREEN}Building plix-server (${TARGET})...${NC}"
    cd "$PROJECT_ROOT"

    if [[ "$TARGET" == "release" ]]; then
        cargo build --release --bin plix-server
    else
        cargo build --bin plix-server
    fi

    echo -e "${GREEN}✓ Build complete${NC}"
else
    echo -e "${YELLOW}Skipping build (using existing binary)${NC}"
fi

# Verify binary exists
BINARY_PATH="${PROJECT_ROOT}/target/${TARGET}/plix-server"
if [[ ! -f "$BINARY_PATH" ]]; then
    echo -e "${RED}Error: Binary not found at ${BINARY_PATH}${NC}"
    echo -e "Run without --skip-build to compile first."
    exit 2
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Create staging directory
STAGING_DIR=$(mktemp -d)
trap "rm -rf $STAGING_DIR" EXIT

echo -e "${GREEN}Packaging release...${NC}"

# Create release structure
mkdir -p "${STAGING_DIR}/plix-server"
mkdir -p "${STAGING_DIR}/plix-server/assets/arenas"
mkdir -p "${STAGING_DIR}/plix-server/config"

# Copy binary
cp "$BINARY_PATH" "${STAGING_DIR}/plix-server/"

# Copy assets
if [[ -d "${PROJECT_ROOT}/assets/arenas" ]]; then
    cp -r "${PROJECT_ROOT}/assets/arenas"/* "${STAGING_DIR}/plix-server/assets/arenas/"
    echo -e "  ✓ Copied arenas"
fi

# Copy config example
if [[ -f "${PROJECT_ROOT}/deploy/config/server.toml.example" ]]; then
    cp "${PROJECT_ROOT}/deploy/config/server.toml.example" "${STAGING_DIR}/plix-server/config/"
    echo -e "  ✓ Copied config example"
fi

# Create README
cat > "${STAGING_DIR}/plix-server/README.txt" << 'READMEEOF'
================================================================================
                          Plix Server - Quick Start
================================================================================

REQUIREMENTS
------------
- Linux x86_64 or arm64 (glibc 2.31+)
- UDP port 7777 open (configurable)

QUICK START
-----------
1. Extract the archive:
   tar -xzf plix-server-*.tar.gz
   cd plix-server

2. Run the server:
   ./plix-server

   The server will start with default settings:
   - Game Mode: FFA (Free-For-All)
   - Port: 7777/udp
   - Max Players: 16

CONFIGURATION
-------------
Configure via environment variables or command-line flags:

  Environment Variables:
    PLIX_PORT=7777              # UDP port
    PLIX_SERVER_NAME="My Server" # Server name
    PLIX_GAME_MODE=ffa          # Game mode: ffa, tdm, ctf, br_lite
    PLIX_ARENA=test_arena       # Arena name
    PLIX_MAX_PLAYERS=16         # Max players
    PLIX_TICKRATE=60            # Server tick rate
    RUST_LOG=info               # Log level

  Command-line flags:
    ./plix-server --port 7777 --server-name "My Server" --mode ffa

  Priority: CLI flags > Environment variables > Defaults

EXAMPLES
--------
  # FFA server with custom name
  PLIX_SERVER_NAME="FFA Arena" ./plix-server

  # TDM server on custom port
  ./plix-server --port 8888 --mode tdm

  # CTF server with more players
  ./plix-server --mode ctf --max-players 32

LOGGING
-------
Logs are written to stdout. Use RUST_LOG to control verbosity:
  RUST_LOG=debug ./plix-server    # Verbose
  RUST_LOG=warn ./plix-server     # Quiet

ARENAS
------
Arena definitions are in assets/arenas/*.toml
Create custom arenas by adding new .toml files.

DOCUMENTATION
-------------
Full documentation: https://github.com/your-org/plix

================================================================================
READMEEOF

echo -e "  ✓ Created README"

# Create archive
echo -e "${GREEN}Creating archive...${NC}"
cd "${STAGING_DIR}"
tar -czf "$ARCHIVE_FILE" plix-server/

echo -e "  ✓ Created ${ARCHIVE_FILE}"

# Generate checksum
echo -e "${GREEN}Generating checksum...${NC}"
cd "$OUTPUT_DIR"
sha256sum "$(basename "$ARCHIVE_FILE")" > "$CHECKSUM_FILE"

echo -e "  ✓ Created ${CHECKSUM_FILE}"

# Summary
echo ""
echo -e "${GREEN}=== Release Complete ===${NC}"
echo -e "  Archive:  ${ARCHIVE_FILE}"
echo -e "  Checksum: ${CHECKSUM_FILE}"
echo ""
echo -e "  Size: $(du -h "$ARCHIVE_FILE" | cut -f1)"
echo -e "  SHA256: $(cat "$CHECKSUM_FILE" | cut -d' ' -f1)"
echo ""
echo -e "${BLUE}To deploy:${NC}"
echo -e "  scp ${ARCHIVE_FILE} user@server:/path/"
echo -e "  ssh user@server 'cd /path && tar -xzf $(basename "$ARCHIVE_FILE") && cd plix-server && ./plix-server'"
