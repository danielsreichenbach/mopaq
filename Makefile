# Makefile for mopaq development

.PHONY: all build test check fmt clippy doc clean bench release help

# Default target
all: check test

# Build all crates
build:
	@echo "Building all crates..."
	@cargo build --all

# Run all tests
test:
	@echo "Running tests..."
	@cargo test --all --all-features

# Run a comprehensive check (format, lint, test)
check: fmt clippy test
	@echo "All checks passed!"

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt --all

# Run clippy linter
clippy:
	@echo "Running clippy..."
	@cargo clippy --all-features --all-targets -- -D warnings

# Generate documentation
doc:
	@echo "Generating documentation..."
	@cargo doc --all-features --no-deps --open

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf storm-ffi/include/StormLib.h

# Run benchmarks
bench:
	@echo "Running benchmarks..."
	@cargo bench

# Build release binaries
release:
	@echo "Building release binaries..."
	@cargo build --all --release

# Run security audit
audit:
	@echo "Running security audit..."
	@cargo audit

# Install the CLI tool locally
install-cli:
	@echo "Installing storm-cli..."
	@cargo install --path storm-cli

# Run FFI tests with C example
test-ffi: build
	@echo "Testing FFI bindings..."
	@cargo test --package storm-ffi

# Test debug commands
test-debug: build
	@echo "Testing debug commands..."
	@cargo run --bin storm-cli -- debug crypto
	@cargo run --bin storm-cli -- debug hash "(listfile)" --all

# Create test data directory
test-data:
	@mkdir -p test-data/{v1,v2,v3,v4}
	@chmod +x scripts/*.py scripts/*.sh 2>/dev/null || true

# Help target
help:
	@echo "mopaq Development Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  all         - Run format, lint, and test (default)"
	@echo "  build       - Build all crates"
	@echo "  test        - Run all tests"
	@echo "  check       - Run format, clippy, and tests"
	@echo "  fmt         - Format code with rustfmt"
	@echo "  clippy      - Run clippy linter"
	@echo "  doc         - Generate and open documentation"
	@echo "  clean       - Clean build artifacts"
	@echo "  bench       - Run benchmarks"
	@echo "  release     - Build release binaries"
	@echo "  audit       - Run security audit"
	@echo "  install-cli - Install the storm-cli tool"
	@echo "  test-ffi    - Test FFI bindings"
	@echo "  test-debug  - Test debug commands"
	@echo "  test-data   - Create test data directories"
	@echo "  help        - Show this help message"
