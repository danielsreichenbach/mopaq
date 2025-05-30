# mopaq

A high-performance, safe Rust implementation of the MPQ (Mo'PaQ) archive format used by Blizzard Entertainment games.

[![Crates.io](https://img.shields.io/crates/v/mopaq.svg)](https://crates.io/crates/mopaq)
[![Documentation](https://docs.rs/mopaq/badge.svg)](https://docs.rs/mopaq)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![CI Status](https://github.com/danielsreichenbach/mopaq/workflows/CI/badge.svg)](https://github.com/danielsreichenbach/mopaq/actions)

## Features

- 🚀 **Full MPQ Format Support**: Implements all MPQ versions (v1-v4) with complete feature parity
- 🔒 **Security First**: Safe Rust implementation with comprehensive error handling
- ⚡ **High Performance**: Memory-mapped I/O, zero-copy operations, and efficient caching
- 🔧 **StormLib Compatible**: Drop-in replacement via FFI bindings (in development)
- 🗜️ **Compression Support**: Multiple compression methods (zlib, bzip2, LZMA, sparse)
- 🔐 **Encryption Support**: Full encryption/decryption for protected archives
- 🛠️ **Rich CLI Tool**: Comprehensive command-line interface with debugging capabilities
- 📊 **Well Tested**: Extensive test suite with fuzzing and benchmarks
- 🦀 **Pure Rust**: No system dependencies required (using lzma-rs for LZMA support)

## Project Structure

- **`mopaq`**: Core MPQ library (named after the original Mo'PaQ format)
- **`storm-ffi`**: StormLib-compatible C API bindings (in development)
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

The `storm-ffi` crate provides a StormLib-compatible C API (currently in development). See the [FFI documentation](storm-ffi/README.md) for integration details.

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

    // Note: Use ArchiveBuilder for adding files (see below)

    Ok(())
}
```

### Archive Creation

```rust
use mopaq::{ArchiveBuilder, FormatVersion, ListfileOption};

fn main() -> mopaq::Result<()> {
    // Create a new archive with files
    ArchiveBuilder::new()
        .version(FormatVersion::V2)
        .add_file("readme.txt", "readme.txt")
        .add_file_data(b"Hello, MPQ!".to_vec(), "greeting.txt")
        .default_compression(mopaq::compression::flags::ZLIB)
        .build("my_archive.mpq")?;

    Ok(())
}
```

### Crypto Example

```rust
use mopaq::crypto::{encrypt_block, decrypt_block};

// Encrypt some data
let mut data = vec![0x12345678, 0x9ABCDEF0];
let key = 0xDEADBEEF;

encrypt_block(&mut data, key);
println!("Encrypted: {:08X?}", data);

decrypt_block(&mut data, key);
println!("Decrypted: {:08X?}", data);
```

### Hash Example

```rust
use mopaq::hash::{hash_string, hash_type};

// Generate hash values for file lookup
let filename = "units\\human\\footman.mdx";
let hash_a = hash_string(filename, hash_type::NAME_A);
let hash_b = hash_string(filename, hash_type::NAME_B);
let table_index = hash_string(filename, hash_type::TABLE_OFFSET);

println!("Hash A: 0x{:08X}", hash_a);
println!("Hash B: 0x{:08X}", hash_b);
println!("Table Index: 0x{:08X}", table_index);
```

### CLI Usage

storm-cli supports tab completion for bash, zsh, fish, and PowerShell.

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

# Display table contents
storm-cli debug tables Diablo2.mpq

# Generate hash values for debugging
storm-cli debug hash "(listfile)" --all

# Compare hashes to check for collisions
storm-cli debug hash-compare "file1.txt" "file2.txt"
```

## Current Status

### Implemented ✅

- **Archive Reading**
  - ✅ All MPQ versions (v1-v4) header parsing
  - ✅ Hash table and block table reading
  - ✅ Hi-block table support for large archives
  - ✅ File extraction with all supported compression methods
  - ✅ Encryption/decryption with key calculation
  - ✅ Sector-based file reading
  - ✅ CRC validation
  - ✅ Archive integrity verification

- **Archive Creation**
  - ✅ Create new archives (v1-v3)
  - ✅ Add files with compression
  - ✅ Automatic hash table sizing
  - ✅ Listfile generation
  - ✅ Multi-sector file support

- **Compression**
  - ✅ Zlib/Deflate
  - ✅ BZip2
  - ✅ LZMA (using lzma-rs)
  - ✅ Sparse/RLE
  - ✅ Multiple compression detection

- **Cryptography**
  - ✅ Encryption table generation
  - ✅ File encryption/decryption
  - ✅ Table encryption/decryption
  - ✅ Key calculation algorithms

- **CLI Tool**
  - ✅ List, extract, find, verify commands
  - ✅ Comprehensive debug commands
  - ✅ Hash calculation and comparison
  - ✅ Table inspection

### In Progress 🚧

- 🚧 Encryption support in ArchiveBuilder
- 🚧 Sector CRC generation for new files
- 🚧 v4 format creation with MD5 checksums
- 🚧 StormLib FFI compatibility layer

### Planned 📋

- 📋 HET/BET table support (v3+)
- 📋 Digital signature support
- 📋 In-place archive modification
- 📋 PKWare DCL compression
- 📋 Huffman compression
- 📋 ADPCM audio compression

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

mopaq is designed for high performance:

- Memory-mapped I/O for large archives
- Zero-copy operations where possible
- Efficient hash table lookups with caching
- Parallel decompression support (planned)

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
git clone https://github.com/danielsreichenbach/mopaq
cd mopaq

# Run tests
cargo test --all

# Run benchmarks
cargo bench

# Build everything
cargo build --all

# Run the CLI tool
cargo run --bin storm-cli -- list test.mpq
```

### Testing

The project includes comprehensive tests:

```bash
# Run all tests
cargo test --all

# Run specific test suites
cargo test -p mopaq compression  # Compression tests
cargo test -p mopaq crypto       # Cryptography tests
cargo test -p mopaq hash         # Hash function tests
cargo test -p mopaq builder      # Archive creation tests

# Run with logging
RUST_LOG=debug cargo test
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
