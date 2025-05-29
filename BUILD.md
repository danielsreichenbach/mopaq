# Building mopaq

## Prerequisites

- Rust 1.86.0 or later
- A C compiler (for FFI examples)
- Make (optional, for convenience commands)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/danielsreichenbach/mopaq
cd mopaq

# Build all crates
cargo build --all

# Run tests
cargo test --all

# Build release version
cargo build --all --release
```

## Testing the Implementation

```bash
# Create test MPQ files
python3 scripts/create_test_mpq.py

# Test the debug info command
cargo run --bin storm-cli -- debug info test-data/v1/simple.mpq
cargo run --bin storm-cli -- debug info test-data/v2/simple.mpq
cargo run --bin storm-cli -- debug info test-data/v4/simple.mpq
```

## Using Make

If you have Make installed, you can use the convenience commands:

```bash
# Build everything
make build

# Run all checks (format, lint, test)
make check

# Generate documentation
make doc

# Install the CLI tool
make install-cli
```

## Individual Crate Building

```bash
# Build only the core library
cargo build -p mopaq

# Build only the FFI library
cargo build -p storm-ffi

# Build only the CLI tool
cargo build -p storm-cli
```

## Common Issues

### Missing Dependencies

The FFI crate requires `cbindgen` for header generation. It will be automatically installed as a build dependency.

### Platform-Specific Notes

- **Windows**: You may need Visual Studio or MinGW for C compilation
- **macOS**: Xcode Command Line Tools required
- **Linux**: gcc or clang required
