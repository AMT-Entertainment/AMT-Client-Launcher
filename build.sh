#!/bin/bash
# AMT Client Build Script
# Builds the launcher for all platforms

set -e

echo "=== AMT Client Build ==="
echo ""

# Check dependencies
command -v bun >/dev/null 2>&1 || { echo "Error: bun not found. Install from https://bun.sh"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found. Install from https://rustup.rs"; exit 1; }
command -v rustc >/dev/null 2>&1 || { echo "Error: rustc not found."; exit 1; }

echo "Dependencies: OK"
echo "  bun: $(bun --version)"
echo "  rustc: $(rustc --version)"
echo "  cargo: $(cargo --version)"
echo ""

# Install JS dependencies
echo "Installing frontend dependencies..."
bun install
echo ""

# Build frontend
echo "Building frontend..."
bun run build
echo ""

# Build for current platform
echo "Building native binary..."
BUILD_MODE="${1:-release}"

if [ "$BUILD_MODE" = "release" ]; then
    cd src-tauri && cargo build --release
    echo ""
    echo "Build complete! Binary at: src-tauri/target/release/amt-client"
else
    cd src-tauri && cargo build
    echo ""
    echo "Build complete! Binary at: src-tauri/target/debug/amt-client"
fi

echo ""
echo "=== Build finished ==="
echo ""
echo "To create platform installers, run:"
echo "  bun run tauri build"
echo ""
echo "This will create:"
echo "  macOS: .dmg (src-tauri/target/release/bundle/dmg/)"
echo "  Windows: .msi/.exe (src-tauri/target/release/bundle/msi/)"
echo "  Linux: .deb/.AppImage (src-tauri/target/release/bundle/deb/)"
