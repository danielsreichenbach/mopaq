# Building StormLib-rs

## Prerequisites

- Rust 1.86.0 or later
- A C compiler (for FFI examples)
- Make (optional, for convenience commands)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/danielsreichenbach/stormlib-rs
cd stormlib-rs

# Build all crates
cargo build --all

# Run tests
cargo test --all

# Build release version
cargo build --all --release
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
cargo build -p storm

# Build only the FFI library
cargo build -p storm-ffi

# Build only the CLI tool
cargo build -p storm-cli
```

## Common Issues

### Edition 2021

We use Rust edition 2021. If you see errors about `unsafe(no_mangle)`, ensure your Rust version is up to date.

### Missing Dependencies

The FFI crate requires `cbindgen` for header generation. It will be automatically installed as a build dependency.

### Platform-Specific Notes

- **Windows**: You may need Visual Studio or MinGW for C compilation
- **macOS**: Xcode Command Line Tools required
- **Linux**: gcc or clang required
