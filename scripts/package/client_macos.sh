#!/usr/bin/env bash
# Package Plix client for macOS
#
# Creates a distributable zip archive containing a proper .app bundle:
# - Plix.app/Contents/MacOS/plix-client
# - Plix.app/Contents/Resources/assets/
# - Plix.app/Contents/Frameworks/ (CEF if enabled)
# - Plix.app/Contents/Info.plist
# - build_info.json (in archive root)
#
# Usage:
#   ./client_macos.sh --binary-path <path> --version <semver> --output-dir <path> [--assets-dir <path>] [--cef-dir <path>]
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
BUNDLE_IDENTIFIER="com.plix.game-client"

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
        --bundle-identifier)
            BUNDLE_IDENTIFIER="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 --binary-path <path> --version <semver> --output-dir <path>"
            echo ""
            echo "Package Plix client for macOS as .app bundle"
            echo ""
            echo "Required arguments:"
            echo "  --binary-path  Path to compiled plix-client binary"
            echo "  --version      Version string (semver, e.g., 0.1.0)"
            echo "  --output-dir   Output directory for archive"
            echo ""
            echo "Optional arguments:"
            echo "  --assets-dir          Path to assets directory (default: ./assets)"
            echo "  --cef-dir             Path to CEF framework directory (optional)"
            echo "  --bundle-identifier   macOS bundle identifier (default: com.plix.game-client)"
            echo "  -h, --help            Show this help message"
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
PLATFORM="macos"
BUNDLE_NAME="plix-client-${PLATFORM}-${VERSION}"
BUNDLE_DIR="$OUTPUT_DIR/$BUNDLE_NAME"
APP_BUNDLE="$BUNDLE_DIR/Plix.app"
ARCHIVE_PATH="$OUTPUT_DIR/${BUNDLE_NAME}.zip"

echo "Packaging plix-client for macOS..." >&2
echo "  Version: $VERSION" >&2
echo "  Binary: $BINARY_PATH" >&2
echo "  Assets: $ASSETS_DIR" >&2
echo "  Output: $ARCHIVE_PATH" >&2

# Clean up any existing bundle directory
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Create .app bundle structure
echo "Creating .app bundle structure..." >&2
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"
mkdir -p "$APP_BUNDLE/Contents/Frameworks"

# Copy binary
echo "Copying binary..." >&2
cp "$BINARY_PATH" "$APP_BUNDLE/Contents/MacOS/plix-client"
chmod +x "$APP_BUNDLE/Contents/MacOS/plix-client"

# Copy assets to Resources
echo "Copying assets..." >&2
cp -r "$ASSETS_DIR" "$APP_BUNDLE/Contents/Resources/assets"

# Create Info.plist
echo "Creating Info.plist..." >&2
cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>plix-client</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_IDENTIFIER</string>
    <key>CFBundleName</key>
    <string>Plix</string>
    <key>CFBundleDisplayName</key>
    <string>Plix</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleSupportedPlatforms</key>
    <array>
        <string>MacOSX</string>
    </array>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.games</string>
</dict>
</plist>
EOF

# Create PkgInfo
echo "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

# Generate build_info.json in bundle root (outside .app)
echo "Generating build_info.json..." >&2
"$SCRIPT_DIR/generate_build_info.sh" \
    --binary-path "$APP_BUNDLE/Contents/MacOS/plix-client" \
    --output "$BUNDLE_DIR/build_info.json" 2>/dev/null || {
    # If script fails, create minimal build_info.json
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

# Copy CEF framework if specified
if [[ -n "$CEF_DIR" && -d "$CEF_DIR" ]]; then
    echo "Copying CEF framework..." >&2

    # Look for CEF framework in common locations
    CEF_FRAMEWORK=""
    for candidate in \
        "$CEF_DIR/Chromium Embedded Framework.framework" \
        "$CEF_DIR/Release/Chromium Embedded Framework.framework" \
        "$CEF_DIR/cef_binary_*/Release/Chromium Embedded Framework.framework"; do
        if [[ -d "$candidate" ]]; then
            CEF_FRAMEWORK="$candidate"
            break
        fi
    done

    if [[ -n "$CEF_FRAMEWORK" ]]; then
        cp -R "$CEF_FRAMEWORK" "$APP_BUNDLE/Contents/Frameworks/"
    else
        echo "Warning: CEF framework not found in $CEF_DIR" >&2
    fi

    # Copy helper apps if present
    for helper in "Plix Helper.app" "Plix Helper (GPU).app" "Plix Helper (Plugin).app" "Plix Helper (Renderer).app"; do
        if [[ -d "$CEF_DIR/$helper" ]]; then
            cp -R "$CEF_DIR/$helper" "$APP_BUNDLE/Contents/Frameworks/"
        fi
    done
fi

# Validate bundle before archiving
echo "Validating bundle..." >&2
if [[ ! -x "$APP_BUNDLE/Contents/MacOS/plix-client" ]]; then
    echo "Error: Binary not executable in bundle" >&2
    exit 4
fi
if [[ ! -f "$BUNDLE_DIR/build_info.json" ]]; then
    echo "Error: build_info.json missing from bundle" >&2
    exit 4
fi
if [[ ! -d "$APP_BUNDLE/Contents/Resources/assets" ]]; then
    echo "Error: assets directory missing from bundle" >&2
    exit 4
fi
if [[ ! -f "$APP_BUNDLE/Contents/Info.plist" ]]; then
    echo "Error: Info.plist missing from bundle" >&2
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
else
    echo "Warning: No SHA256 tool found, skipping checksum" >&2
fi

# Clean up bundle directory
rm -rf "$BUNDLE_DIR"

echo "Package created successfully!" >&2
echo "  Archive: $ARCHIVE_PATH" >&2
echo "  Checksum: $ARCHIVE_PATH.sha256" >&2

# Output archive path to stdout (for CI consumption)
echo "$ARCHIVE_PATH"
