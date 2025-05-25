#!/bin/bash
# Test the new hash debug commands

set -e

echo "Testing storm-cli debug hash commands"
echo "====================================="
echo

# Build the CLI tool
echo "Building storm-cli..."
cargo build --bin storm-cli 2>/dev/null || {
    echo "Build failed! Showing full output:"
    cargo build --bin storm-cli
    exit 1
}

# Function to run storm-cli
run_storm() {
    cargo run -q --bin storm-cli -- "$@"
}

echo "1. Generate all hash values for a filename:"
echo "----------------------------------------"
run_storm debug hash "(listfile)" --all
echo

echo "2. Generate specific hash type:"
echo "------------------------------"
run_storm debug hash "units\\human\\footman.mdx" --hash-type file-key
echo

echo "3. Generate Jenkins hash:"
echo "------------------------"
run_storm debug hash "war3map.j" --jenkins
echo

echo "4. Compare hash values between files:"
echo "------------------------------------"
run_storm debug hash-compare "file.txt" "FILE.TXT"
echo

echo "5. Check path normalization:"
echo "----------------------------"
run_storm debug hash-compare "path/to/file.txt" "path\\to\\file.txt"
echo

echo "6. Generate table encryption keys:"
echo "---------------------------------"
echo "Hash table key:"
run_storm debug hash "(hash table)" --hash-type file-key
echo
echo "Block table key:"
run_storm debug hash "(block table)" --hash-type file-key
echo

echo "7. Test collision detection:"
echo "---------------------------"
run_storm debug hash-compare "collision1.txt" "collision2.txt"
echo

echo "8. Help for hash command:"
echo "------------------------"
run_storm debug hash --help || true
echo

echo "✓ Hash debug commands working correctly!"
echo
echo "These commands are useful for:"
echo "- Debugging MPQ file lookups"
echo "- Understanding hash collisions"
echo "- Verifying hash calculations"
echo "- Generating encryption keys for tables"
