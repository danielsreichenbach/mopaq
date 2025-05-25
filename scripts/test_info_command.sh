#!/bin/bash
# Test the debug info command implementation

set -e

echo "Testing storm-cli debug info command"
echo "===================================="
echo

# Create test MPQ files if they don't exist
if [ ! -f "test-data/v1/simple.mpq" ]; then
    echo "Creating test MPQ files..."
    python3 scripts/create_test_mpq.py
    echo
fi

# Build the project
echo "Building storm-cli..."
cargo build --bin storm-cli 2>/dev/null || {
    echo "Build failed! Showing full output:"
    cargo build --bin storm-cli
    exit 1
}
echo

# Test each format version
for mpq in test-data/v1/simple.mpq test-data/v2/simple.mpq test-data/v4/simple.mpq test-data/v1/userdata.mpq; do
    if [ -f "$mpq" ]; then
        echo "Testing: $mpq"
        echo "---"
        cargo run -q --bin storm-cli -- debug info "$mpq" || {
            echo "Error processing $mpq"
        }
        echo
        echo
    fi
done

echo "✓ Debug info command tests completed!"
