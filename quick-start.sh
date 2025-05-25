#!/bin/bash
# Quick start script for StormLib-rs

echo "StormLib-rs Quick Start"
echo "======================"
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo is not installed. Please install Rust first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi

echo "1. Building all components..."
cargo build --all

if [ $? -eq 0 ]; then
    echo "✓ Build successful!"
    echo ""
    echo "2. Installing storm-cli..."
    cargo install --path storm-cli

    echo ""
    echo "✓ Installation complete!"
    echo ""
    echo "You can now use storm-cli:"
    echo "  storm-cli --help           # Show help"
    echo "  storm-cli list <mpq>       # List archive contents"
    echo "  storm-cli extract <mpq>    # Extract files"
    echo "  storm-cli create <mpq> <dir> # Create archive"
    echo ""
    echo "Library usage:"
    echo "  Add to Cargo.toml: storm = { path = \"storm\" }"
    echo ""
    echo "FFI usage:"
    echo "  Link against: target/*/libstorm.{so,dylib,dll}"
    echo "  Include: storm-ffi/include/StormLib.h"
else
    echo "✗ Build failed. Please check the errors above."
    exit 1
fi
