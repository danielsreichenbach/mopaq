#!/bin/bash
# Test crypto performance

set -e

echo "Testing MPQ Crypto Performance"
echo "=============================="
echo

# Build in release mode for performance testing
echo "Building in release mode..."
cargo build --release --bin storm-cli 2>/dev/null || {
    echo "Build failed! Showing full output:"
    cargo build --release --bin storm-cli
    exit 1
}

# Test crypto functions
echo
echo "Testing crypto functions:"
echo "------------------------"
cargo run --release --bin storm-cli -- debug crypto

# Run benchmarks if requested
if [ "$1" = "--bench" ]; then
    echo
    echo "Running crypto benchmarks:"
    echo "-------------------------"
    cargo bench --bench crypto
fi

echo
echo "✓ Crypto tests completed!"
echo
echo "To run full benchmarks, use: $0 --bench"
