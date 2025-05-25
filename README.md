# StormLib-rs

A high-performance, safe Rust implementation of the MPQ (Mo'PaQ) archive format used by Blizzard Entertainment games.

[![Crates.io](https://img.shields.io/crates/v/mopaq.svg)](https://crates.io/crates/mopaq)
[![Documentation](https://docs.rs/mopaq/badge.svg)](https://docs.rs/mopaq)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![CI Status](https://github.com/danielsreichenbach/stormlib-rs/workflows/CI/badge.svg)](https://github.com/danielsreichenbach/stormlib-rs/actions)

## Features

- 🚀 **Full MPQ Format Support**: Implements all MPQ versions (v1-v4) with complete feature parity
- 🔒 **Security First**: Safe Rust implementation with comprehensive error handling
- ⚡ **High Performance**: Memory-mapped I/O, zero-copy operations, and efficient caching
- 🔧 **StormLib Compatible**: Drop-in replacement via FFI bindings
- 🗜️ **Compression Support**: All MPQ compression methods (zlib, bzip2, LZMA, sparse, etc.)
- 🔐 **Digital Signatures**: Both weak (512-bit RSA) and strong (2048-bit RSA) signature verification
- 🛠️ **Rich CLI Tool**: Comprehensive command-line interface with debugging capabilities
- 📊 **Well Tested**: Extensive test suite with fuzzing and benchmarks

## Project Structure

- **`mopaq`**: Core MPQ library (named after the original Mo'PaQ format)
- **`storm-ffi`**: StormLib-compatible C API bindings
- **`storm-cli`**: Command-line tool for MPQ operations

## Installation

### As a Rust Library

```toml
[dependencies]
mopaq = "0.1"
```

### CLI Tool

```bash
cargo install storm-cli
```

### C/C++ FFI

The `storm-ffi` crate provides a StormLib-compatible C API. See the [FFI documentation](storm-ffi/README.md) for integration details.

## Quick Start

### Rust API

```rust
use mopaq::{Archive, OpenOptions};

fn main() -> mopaq::Result<()> {
    // Open an existing MPQ archive
    let mut archive = Archive::open("StarCraft.mpq")?;

    // List all files
    for entry in archive.list()? {
        println!("{} ({} bytes)", entry.name, entry.size);
    }

    // Extract a specific file
    let data = archive.read_file("unit\\terran\\marine.grp")?;
    std::fs::write("marine.grp", data)?;

    // Create a new archive
    let mut new_archive = OpenOptions::new()
        .version(mopaq::FormatVersion::V2)
        .create("my_archive.mpq")?;

    new_archive.add_file("readme.txt", b"Hello, MPQ!")?;

    Ok(())
}
```

### CLI Usage

```bash
# List files in an archive
storm-cli list StarCraft.mpq

# Extract files
storm-cli extract StarCraft.mpq --output ./extracted

# Create a new archive
storm-cli create my_mod.mpq ./mod_files

# Verify archive integrity
storm-cli verify WarCraft3.w3m

# Debug archive structure
storm-cli debug info Diablo2.mpq
```

## Supported Games

- Diablo (1996)
- StarCraft (1998)
- Diablo II (2000)
- WarCraft III (2002)
- World of Warcraft (2004-present)
- StarCraft II (2010)
- Heroes of the Storm
- And other Blizzard games using MPQ format

## Performance

StormLib-rs is designed for high performance:

- Memory-mapped I/O for large archives
- Parallel decompression support
- Efficient hash table lookups with caching
- Zero-copy operations where possible

See [benchmarks](docs/benchmarks.md) for detailed performance comparisons.

## Architecture

The project consists of three main components:

1. **mopaq** - Core library with pure Rust implementation
2. **storm-ffi** - C-compatible FFI bindings for StormLib compatibility
3. **storm-cli** - Feature-rich command-line tool

See [architecture documentation](docs/architecture.md) for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/danielsreichenbach/stormlib-rs
cd stormlib-rs

# Run tests
cargo test --all

# Run benchmarks
cargo bench

# Build everything
cargo build --all
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- Ladislav Zezula for the original [StormLib](https://github.com/ladislav-zezula/StormLib) implementation
- The [wowdev](https://wowdev.wiki/) community for format documentation
- All contributors to MPQ format reverse engineering efforts

## Related Projects

- [StormLib](https://github.com/ladislav-zezula/StormLib) - Original C++ implementation
- [ceres-mpq](https://github.com/ceres-wc3/ceres-mpq) - Rust MPQ reader
- [JMPQ](https://github.com/IntelOrca/JMPQ) - Java implementation
