#!/usr/bin/env bash
# Package Plix headless server for macOS
#
# Creates a distributable zip archive containing:
# - plix-server-headless binary
# - build_info.json
# - configs/examples/ directory
# - docs/README.md
# - run_server.sh launcher script
#
# Usage:
#   ./server_macos.sh --binary-path <path> --version <semver> --output-dir <path>
#
# Exit codes:
#   0 - Success
#   1 - Missing required argument
#   2 - Binary not found
#   3 - Config examples not found
#   4 - Archive creation failed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Default values
BINARY_PATH=""
VERSION=""
OUTPUT_DIR=""

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
        -h|--help)
            echo "Usage: $0 --binary-path <path> --version <semver> --output-dir <path>"
            echo ""
            echo "Package Plix headless server for macOS"
            echo ""
            echo "Required arguments:"
            echo "  --binary-path  Path to compiled plix-server-headless binary"
            echo "  --version      Version string (semver, e.g., 0.1.0)"
            echo "  --output-dir   Output directory for archive"
            echo ""
            echo "Optional arguments:"
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

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Bundle naming
PLATFORM="macos"
BUNDLE_NAME="plix-server-headless-${PLATFORM}-${VERSION}"
BUNDLE_DIR="$OUTPUT_DIR/$BUNDLE_NAME"
ARCHIVE_PATH="$OUTPUT_DIR/${BUNDLE_NAME}.zip"

echo "Packaging plix-server-headless for macOS..." >&2
echo "  Version: $VERSION" >&2
echo "  Binary: $BINARY_PATH" >&2
echo "  Output: $ARCHIVE_PATH" >&2

# Clean up any existing bundle directory
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Copy binary
echo "Copying binary..." >&2
cp "$BINARY_PATH" "$BUNDLE_DIR/plix-server-headless"
chmod +x "$BUNDLE_DIR/plix-server-headless"

# Copy example configs
echo "Copying example configs..." >&2
mkdir -p "$BUNDLE_DIR/configs/examples"
if [[ -d "$PROJECT_ROOT/deploy/configs/examples" ]]; then
    cp "$PROJECT_ROOT/deploy/configs/examples/"*.toml "$BUNDLE_DIR/configs/examples/" 2>/dev/null || true
else
    cat > "$BUNDLE_DIR/configs/examples/server.toml" << 'EOF'
# Plix Server Configuration

[network]
bind_address = "0.0.0.0"
port = 7777
max_players = 32

[game]
tick_rate = 30
arena = "ffa_small"

[server]
name = "My Plix Server"
motd = "Welcome!"

[logging]
level = "info"

[persistence]
autosave_interval_secs = 300
shutdown_timeout_secs = 5
EOF
fi

# Generate build_info.json
echo "Generating build_info.json..." >&2
"$SCRIPT_DIR/generate_build_info.sh" \
    --binary-path "$BUNDLE_DIR/plix-server-headless" \
    --output "$BUNDLE_DIR/build_info.json" 2>/dev/null || {
    cat > "$BUNDLE_DIR/build_info.json" << EOF
{
  "version": "$VERSION",
  "commit_sha": "unknown",
  "commit_sha_short": "unknown",
  "build_timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "build_date": "$(date -u +"%Y-%m-%d")",
  "target_triple": "x86_64-apple-darwin",
  "rust_version": "unknown",
  "branch": "unknown",
  "is_dirty": false
}
EOF
}

# Create launcher script
echo "Creating launcher script..." >&2
cat > "$BUNDLE_DIR/run_server.sh" << 'EOF'
#!/usr/bin/env bash
# Plix Server Launcher
#
# Usage: ./run_server.sh [config_file]
#
# If no config file is specified, uses configs/examples/server.toml

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/plix-server-headless"
CONFIG="${1:-$SCRIPT_DIR/configs/examples/server.toml}"

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found: $BINARY" >&2
    exit 1
fi

if [[ ! -f "$CONFIG" ]]; then
    echo "Error: Config not found: $CONFIG" >&2
    echo "Using default settings..." >&2
    exec "$BINARY"
else
    exec "$BINARY" --config "$CONFIG"
fi
EOF
chmod +x "$BUNDLE_DIR/run_server.sh"

# Create docs directory with README
echo "Creating documentation..." >&2
mkdir -p "$BUNDLE_DIR/docs"
cat > "$BUNDLE_DIR/docs/README.md" << EOF
# Plix Headless Server

Version: $VERSION
Platform: macOS

## Quick Start

1. Edit the configuration file:
   \`\`\`bash
   cp configs/examples/server.toml ./server.toml
   vim server.toml
   \`\`\`

2. Start the server:
   \`\`\`bash
   ./run_server.sh server.toml
   # or directly:
   ./plix-server-headless --config server.toml
   \`\`\`

3. Stop the server:
   - Press Ctrl+C for graceful shutdown
   - Send SIGTERM for graceful shutdown
   - Server will force-exit after shutdown timeout (default: 5s)

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 64 | Port bind failed |
| 65 | Asset load failed |
| 66 | Persistence error |
| 67 | Network error |
| 68 | Shutdown timeout |

## Command Line Options

\`\`\`
./plix-server-headless --help
\`\`\`

## Configuration

See \`configs/examples/server.toml\` for all configuration options.

## launchd Service

Example launchd plist (save to ~/Library/LaunchAgents/com.plix.server.plist):

\`\`\`xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.plix.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/plix/plix-server-headless</string>
        <string>--config</string>
        <string>/usr/local/plix/server.toml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>WorkingDirectory</key>
    <string>/usr/local/plix</string>
</dict>
</plist>
\`\`\`
EOF

# Validate bundle before archiving
echo "Validating bundle..." >&2
if [[ ! -x "$BUNDLE_DIR/plix-server-headless" ]]; then
    echo "Error: Binary not executable in bundle" >&2
    exit 4
fi
if [[ ! -f "$BUNDLE_DIR/build_info.json" ]]; then
    echo "Error: build_info.json missing from bundle" >&2
    exit 4
fi

# Create archive
echo "Creating archive..." >&2
cd "$OUTPUT_DIR"
zip -r -q "$(basename "$ARCHIVE_PATH")" "$BUNDLE_NAME" || {
    echo "Error: Failed to create archive" >&2
    exit 4
}

# Generate checksum
echo "Generating checksum..." >&2
if command -v shasum &> /dev/null; then
    shasum -a 256 "$(basename "$ARCHIVE_PATH")" > "$(basename "$ARCHIVE_PATH").sha256"
elif command -v sha256sum &> /dev/null; then
    sha256sum "$(basename "$ARCHIVE_PATH")" > "$(basename "$ARCHIVE_PATH").sha256"
fi

# Clean up bundle directory
rm -rf "$BUNDLE_DIR"

echo "Package created successfully!" >&2
echo "  Archive: $ARCHIVE_PATH" >&2
echo "  Checksum: $ARCHIVE_PATH.sha256" >&2

# Output archive path to stdout
echo "$ARCHIVE_PATH"
