#!/bin/bash
# Create necessary directories for the project

echo "Creating project directories..."

# Create directories
mkdir -p storm/{src,tests,benches}
mkdir -p storm-ffi/{src,examples,include}
mkdir -p storm-cli/{src/commands,tests}
mkdir -p test-data/{v1,v2,v3,v4}
mkdir -p docs
mkdir -p scripts

# Make scripts executable
chmod +x scripts/*.py 2>/dev/null || true
chmod +x quick-start.sh 2>/dev/null || true

echo "✓ Project directories created successfully!"
echo ""
echo "Project structure:"
echo "- storm/       - Core library"
echo "- storm-ffi/   - FFI bindings"
echo "- storm-cli/   - CLI tool (binary: storm-cli)"
echo "- test-data/   - Test archives"
echo "- docs/        - Documentation"
echo "- scripts/     - Utility scripts"
