# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### 🚧 Work in Progress

#### Core Library (`mopaq`)

- Encryption support in ArchiveBuilder
- Sector CRC generation for file integrity
- v4 format header writing with MD5 checksums
- HET/BET table support (v3+)

#### CLI Tool (`storm-cli`)

- `create` command implementation
- Progress bars for long operations
- text, JSON and CSV output mode

#### FFI Library (`storm-ffi`)

- Basic StormLib API implementation
- C header generation
- Core API functions
  - `SFileOpenArchive`
  - `SFileCloseArchive`
  - `SFileOpenFileEx`
  - `SFileCloseFile`
  - `SFileReadFile`
  - `SFileGetFileSize`
  - `SFileSetFilePointer`
  - `SFileGetFileInfo`
  - `SFileEnumFiles`
  - `SFileGetLocale`
  - `SFileSetLocale`
- Provide usage examples

### Changed

- Replaced `rust-lzma` with `lzma-rs` for pure Rust LZMA support
  - No system dependencies required
  - Supports both raw LZMA and XZ formats
  - Better cross-platform compatibility

## [0.1.0] - 2025-06-XX (Upcoming)

### ✨ Core Library (`mopaq`)

#### Archive Reading

- **Full MPQ format support** (v1-v4)
  - ✅ Header parsing for all versions
  - ✅ User data header support
  - ✅ Header location with 512-byte alignment scanning
  - ✅ Archive size calculation for v2+ (64-bit values)

- **Table implementations**
  - ✅ Hash table parsing with encryption/decryption
  - ✅ Block table parsing with encryption/decryption
  - ✅ Hi-block table support for archives > 4GB
  - ✅ Hash table collision resolution with linear probing
  - ✅ Locale and platform support in hash entries

- **File operations**
  - ✅ File lookup by name with hash algorithm
  - ✅ Multi-sector file reading
  - ✅ Single unit file support
  - ✅ File extraction with automatic decompression
  - ✅ Sector CRC validation
  - ✅ File enumeration via (listfile)

#### Archive Creation

- **ArchiveBuilder API**
  - ✅ Create new archives (v1-v3 format)
  - ✅ Add files from disk or memory
  - ✅ Automatic hash table sizing
  - ✅ Custom compression per file
  - ✅ Multi-sector file writing
  - ✅ Atomic writes with temp file + rename

- **Listfile support**
  - ✅ Automatic listfile generation
  - ✅ External listfile support
  - ✅ Option to omit listfile

#### Compression

- ✅ **Zlib/Deflate** - Full support
- ✅ **BZip2** - Full support (v2+)
- ✅ **LZMA** - Full support with lzma-rs (v3+)
- ✅ **Sparse/RLE** - Full decompression support (v3+)
- ✅ Multiple compression detection and handling
- ✅ Compression method auto-detection

#### Cryptography

- ✅ **Encryption table generation** (compile-time const)
- ✅ **MPQ hash algorithm** with all hash types
  - Hash type 0: Table offset
  - Hash type 1: Name hash A
  - Hash type 2: Name hash B
  - Hash type 3: File key
  - Hash type 4: Key2 mix
- ✅ **Jenkins hash** for HET tables
- ✅ **Encryption/decryption algorithms**
  - Block encryption/decryption
  - Single DWORD decryption
  - Table encryption/decryption
- ✅ **File key calculation** with FIX_KEY support
- ✅ **ASCII conversion tables** for case-insensitive hashing
- ✅ **Path normalization** (forward slash to backslash)

#### Special Files

- ✅ (listfile) parsing and generation
- ✅ Special file detection and metadata

#### Error Handling

- ✅ Comprehensive error types with context
- ✅ Table-specific error types
- ✅ Corruption detection
- ✅ Recovery classification

### 🛠️ CLI Tool (`storm-cli`)

#### Commands

- ✅ **list** - List files in archive
  - With and without (listfile)
  - Verbose mode with compression ratios
  - Show all entries by index

- ✅ **find** - Find specific files
  - Detailed file information
  - Hash value display
  - Verbose debugging info

- ✅ **extract** - Extract files from archive
  - Single file extraction
  - Bulk extraction via (listfile)
  - Path normalization for OS compatibility

- ✅ **verify** - Archive integrity verification
  - Header validation
  - Table consistency checks
  - File accessibility tests
  - CRC validation

#### Debug Commands

- ✅ **info** - Detailed archive information
  - All header fields
  - Version-specific data
  - MD5 checksums (v4)

- ✅ **crypto** - Test encryption/decryption
  - Encryption table values
  - Round-trip testing

- ✅ **hash** - Hash calculation utilities
  - All hash types
  - Jenkins hash support
  - Path normalization demo

- ✅ **hash-compare** - Compare hash values
  - Collision detection
  - Multiple table size tests

- ✅ **tables** - Table content inspection
  - Hash table entries
  - Block table entries
  - Entry statistics

### 🔧 FFI Library (`storm-ffi`)

- ✅ Basic structure and type definitions
- ✅ Build configuration with cbindgen
- ✅ C header auto-generation setup
- ✅ Error code definitions

### 📊 Testing & Benchmarks

#### Unit Tests

- ✅ Comprehensive crypto tests
- ✅ Hash algorithm verification
- ✅ Table structure tests
- ✅ Compression round-trip tests
- ✅ Archive creation tests
- ✅ Error handling tests

#### Integration Tests

- ✅ Table parsing tests
- ✅ Builder functionality tests
- ✅ CLI command tests
- ✅ CRC validation tests
- ✅ Compression tests

#### Benchmarks

- ✅ Hash function performance
- ✅ Encryption/decryption performance
- ✅ Compression method comparison

### 🏗️ Infrastructure

- ✅ Workspace structure with three crates
- ✅ Comprehensive error types with thiserror
- ✅ Logging support with env_logger
- ✅ Documentation with inline examples
- ✅ CI/CD pipeline configuration
- ✅ Cross-platform build support

## Design Decisions

### Architecture

- Separation of read (`Archive`) and write (`ArchiveBuilder`) operations
- Pure Rust implementation with no system dependencies
- Const-time encryption table generation
- Zero-copy operations where possible

### Compatibility

- Full compatibility with StormLib file formats
- Support for all known MPQ versions
- Preservation of original hash algorithms and encryption

### Safety

- Safe Rust with zero unsafe blocks
- Comprehensive bounds checking
- Memory-safe table operations
- Atomic file operations (temp + rename)

---

[Unreleased]: https://github.com/danielsreichenbach/mopaq/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/danielsreichenbach/mopaq/releases/tag/v0.1.0
