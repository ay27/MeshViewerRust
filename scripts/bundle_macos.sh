#!/bin/bash
#
# bundle_macos.sh - Build and package MeshViewerRust as a macOS .app bundle
#
# Usage:
#   ./scripts/bundle_macos.sh           # Release build (default)
#   ./scripts/bundle_macos.sh debug     # Debug build
#

set -euo pipefail

# --- Configuration ---
APP_NAME="MeshViewerRust"
BUNDLE_NAME="MeshViewerRust.app"
BINARY_NAME="mesh_viewer_rust"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

BUILD_MODE="${1:-release}"

if [ "$BUILD_MODE" = "debug" ]; then
    BUILD_DIR="$PROJECT_DIR/target/debug"
    CARGO_FLAGS=""
else
    BUILD_MODE="release"
    BUILD_DIR="$PROJECT_DIR/target/release"
    CARGO_FLAGS="--release"
fi

OUTPUT_DIR="$PROJECT_DIR/target/$BUILD_MODE/bundle"
APP_DIR="$OUTPUT_DIR/$BUNDLE_NAME"

echo "=== Building $APP_NAME ($BUILD_MODE) ==="

# --- Step 1: Build the binary ---
echo "[1/5] Compiling with cargo build $CARGO_FLAGS ..."
cd "$PROJECT_DIR"
cargo build $CARGO_FLAGS

if [ ! -f "$BUILD_DIR/$BINARY_NAME" ]; then
    echo "ERROR: Binary not found at $BUILD_DIR/$BINARY_NAME"
    exit 1
fi

echo "[2/5] Creating .app bundle structure ..."

# --- Step 2: Create .app directory structure ---
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# --- Step 3: Copy files ---
echo "[3/5] Copying files ..."

# Copy binary
cp "$BUILD_DIR/$BINARY_NAME" "$APP_DIR/Contents/MacOS/$BINARY_NAME"

# Copy Info.plist
if [ -f "$PROJECT_DIR/macos/Info.plist" ]; then
    cp "$PROJECT_DIR/macos/Info.plist" "$APP_DIR/Contents/Info.plist"
else
    echo "WARNING: macos/Info.plist not found, generating a minimal one ..."
    cat > "$APP_DIR/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>MeshViewerRust</string>
    <key>CFBundleIdentifier</key>
    <string>com.meshviewerrust.app</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>mesh_viewer_rust</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST
fi

# Copy icon if exists
if [ -f "$PROJECT_DIR/macos/AppIcon.icns" ]; then
    cp "$PROJECT_DIR/macos/AppIcon.icns" "$APP_DIR/Contents/Resources/AppIcon.icns"
else
    echo "NOTE: No icon file found at macos/AppIcon.icns (app will use default icon)"
fi

# Copy assets if the directory has content
if [ -d "$PROJECT_DIR/assets" ] && [ "$(ls -A "$PROJECT_DIR/assets" 2>/dev/null)" ]; then
    cp -r "$PROJECT_DIR/assets" "$APP_DIR/Contents/Resources/assets"
fi

# Copy config
if [ -d "$PROJECT_DIR/config" ]; then
    cp -r "$PROJECT_DIR/config" "$APP_DIR/Contents/Resources/config"
fi

# --- Step 4: Ad-hoc code sign ---
echo "[4/5] Signing .app bundle ..."
codesign --remove-signature "$APP_DIR" 2>/dev/null || true
codesign -s - "$APP_DIR" -f
echo "  Signature: $(codesign -v "$APP_DIR" 2>&1 && echo 'valid' || echo 'FAILED')"

# --- Step 5: Done ---
echo "[5/5] Bundle complete!"
echo ""
echo "  Output: $APP_DIR"
echo "  Size:   $(du -sh "$APP_DIR" | cut -f1)"
echo ""
echo "You can now:"
echo "  open \"$APP_DIR\"                    # Launch the app"
echo "  cp -r \"$APP_DIR\" /Applications/     # Install to Applications"
echo ""

# Create a DMG (release only).
if [ "$BUILD_MODE" = "release" ]; then
    DMG_PATH="$OUTPUT_DIR/MeshViewerRust-0.1.0.dmg"
    rm -f "$DMG_PATH"

    if command -v create-dmg &>/dev/null; then
        echo "Detected create-dmg, building .dmg installer ..."
        create-dmg \
            --volname "MeshViewerRust" \
            --window-pos 200 120 \
            --window-size 600 400 \
            --icon-size 100 \
            --icon "$BUNDLE_NAME" 150 190 \
            --app-drop-link 450 190 \
            "$DMG_PATH" \
            "$APP_DIR" || true
    elif command -v hdiutil &>/dev/null; then
        echo "create-dmg not found, falling back to hdiutil ..."
        STAGING_DIR="$OUTPUT_DIR/dmg-staging"
        rm -rf "$STAGING_DIR"
        mkdir -p "$STAGING_DIR"
        cp -R "$APP_DIR" "$STAGING_DIR/"
        ln -s /Applications "$STAGING_DIR/Applications" || true
        hdiutil create \
            -volname "MeshViewerRust" \
            -srcfolder "$STAGING_DIR" \
            -ov \
            -format UDZO \
            "$DMG_PATH"
        rm -rf "$STAGING_DIR"
    else
        echo "WARNING: Neither create-dmg nor hdiutil is available; skipping DMG creation."
    fi

    if [ -f "$DMG_PATH" ]; then
        echo "DMG created: $DMG_PATH"
    else
        echo "DMG was not created."
    fi
fi
