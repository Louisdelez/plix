#!/bin/bash
# Pack script for {{mod_name}}
# Creates a distributable .plixmod bundle

set -e

# Build first if wasm doesn't exist
WASM_PATH="target/wasm32-unknown-unknown/release/{{mod_id}}.wasm"
if [ ! -f "$WASM_PATH" ]; then
    echo "Building mod first..."
    ./build.sh release
fi

# Use plix-mod CLI if available, otherwise manual pack
if command -v plix-mod &> /dev/null; then
    plix-mod pack
else
    echo "plix-mod CLI not found, using manual pack..."

    # Create bundle manually
    BUNDLE="{{mod_id}}-$(grep '^version' mod.toml | cut -d'"' -f2).plixmod"

    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cp mod.toml "$TEMP_DIR/"
    cp "$WASM_PATH" "$TEMP_DIR/mod.wasm"

    # Copy assets if they exist
    if [ -d "assets" ]; then
        cp -r assets "$TEMP_DIR/"
    fi

    # Create ZIP
    (cd "$TEMP_DIR" && zip -r "$OLDPWD/$BUNDLE" . -x ".*")
    rm -rf "$TEMP_DIR"

    echo "Packed: $BUNDLE"
    sha256sum "$BUNDLE"
fi
