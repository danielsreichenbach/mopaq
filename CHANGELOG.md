# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Core Library (`mopaq`)

- **HET/BET Table Support (v3+)** - Full read and write support for modern MPQ archives
  - ✅ Complete HET (Hash Entry Table) implementation
    - Header parsing with version and data size support
    - Jenkins hash-based lookups for improved performance
    - Bit-packed index array parsing
    - Full compression and encryption support
    - V3 table size calculation for archives without V4 data
    - HET table creation in ArchiveBuilder with bit-packing
    - Automatic Jenkins hash generation for new files
  - ✅ Complete BET (Block Entry Table) implementation
    - Header parsing with dynamic bit widths
    - Bit-packed field extraction (file position, size, flags, etc.)
    - Optional name hash array support
    - Full compression and encryption support
    - V3 table size calculation for archives without V4 data
    - BET table creation with optimal bit width calculation
    - Bit-packed table entry writing with flag arrays
  - ✅ Automatic fallback to classic hash/block tables
  - ✅ Transparent integration with existing Archive API
  - ✅ V3 archives now create both HET/BET and classic tables for compatibility

- **(attributes) File Support** - Complete implementation for file metadata
  - ✅ Full parsing of (attributes) special file format
  - ✅ Support for all attribute types:
    - CRC32 checksums of uncompressed data
    - Windows FILETIME timestamps
    - MD5 hashes of uncompressed data
    - Patch file bit indicators
  - ✅ Dynamic attribute loading from archives
  - ✅ Per-file attribute access API
  - ✅ Comprehensive test coverage

- **Encryption Support in ArchiveBuilder** - Full file encryption during archive creation
  - ✅ Complete encryption implementation for both single-unit and multi-sector files
  - ✅ Support for FIX_KEY encryption (key adjusted by file position)
  - ✅ Proper encryption key calculation from filenames
  - ✅ Encrypted sector offset table handling
  - ✅ Per-sector encryption with correct key adjustment
  - ✅ New API methods:
    - `add_file_with_encryption()` for encrypted files with custom options
    - `add_file_data_with_encryption()` for encrypted in-memory data
  - ✅ Full test coverage including mixed encrypted/unencrypted archives

- **Sector CRC Generation** - File integrity checksums during archive creation
  - ✅ CRC32 generation for both single-unit and multi-sector files
  - ✅ Proper CRC table placement after sector offset table
  - ✅ FLAG_SECTOR_CRC flag handling
  - ✅ `generate_crcs()` builder method for enabling CRC generation
  - ✅ Compatibility with existing CRC validation in archive reading
  - ✅ Test coverage for CRC round-trip validation

- **Hi-block Table Writing (v2+)** - Support for archives larger than 4GB
  - ✅ 64-bit file position tracking throughout the builder
  - ✅ Automatic Hi-block table creation for v2+ archives
  - ✅ Proper storage of high 16 bits of file positions
  - ✅ Hi-block table writing after block table
  - ✅ Header updates with hi_block_table_pos and high position bits
  - ✅ Backward compatibility - table only written when needed
  - ✅ Test coverage for v2 and v3 format archives
  - ✅ Automatic CRC calculation for file sectors
  - ✅ CRC table generation for multi-sector files
  - ✅ Single-unit file CRC support
  - ✅ New builder method: `generate_crcs(bool)` to enable/disable CRC generation
  - ✅ FLAG_SECTOR_CRC properly set in block table
  - ✅ CRC validation tested with original Blizzard MPQ archives
  - ✅ 100% validation success rate across 2,613 files tested from WoW archives

- **Weak Signature Verification (v1+)** - Digital signature support for archive integrity
  - ✅ RSASSA-PKCS1-v1_5 verification with 512-bit RSA
  - ✅ MD5 hashing of archive contents
  - ✅ Blizzard public key embedded for verification
  - ✅ Automatic signature detection and validation
  - ✅ Proper little-endian to big-endian conversion
  - ✅ PKCS#1 v1.5 padding verification
  - ✅ Integration with archive info API
  - ✅ Example program for signature verification

### Fixed

- **Benchmark compilation failures** - Updated to use `std::hint::black_box`
  - ✅ Replaced deprecated `criterion::black_box` across all benchmarks
  - ✅ Fixed imports in hash, builder, crypto, and compression benchmarks
  - ✅ All benchmarks now compile and run correctly with latest criterion

### 🚧 Work in Progress

#### Core Library (`mopaq`)

- v4 format header writing with MD5 checksums
- HET/BET table compression support (tables are currently written uncompressed)

#### CLI Tool (`storm-cli`)

- **Archive list alias** - Added `archive list` as an alias for `file list` command
  - ✅ Provides more intuitive command structure
  - ✅ Both `storm-cli archive list` and `storm-cli file list` work identically
  - ✅ All options and filters are supported in both commands

### 📚 Documentation

- **Platform Codes Clarification**
  - ✅ Documented that platform codes in hash table entries are vestigial
  - ✅ Analysis revealed all known MPQ archives use platform=0
  - ✅ Updated documentation to reflect that Blizzard uses separate archives instead
  - ✅ Added code comments explaining the unused nature of this field

### 🚧 Work in Progress

#### CLI Tool (`storm-cli`)

- Progress bars for long operations

### Scripts

- **Test Data Generator** (`scripts/generate_test_data.py`)
  - ✅ Generate raw test data for storm-cli archive creation testing
  - ✅ Multiple test configurations (simple, game assets, nested, mixed sizes, special names)
  - ✅ Support for text, binary, and empty files
  - ✅ Configurable file sizes and directory structures

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
- Refactored `special_files.rs` into a module structure
  - Split into `listfile.rs` for parsing functionality
  - Separated `info.rs` for special file metadata
  - Improved code organization and maintainability

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
  - ✅ HET (Hash Entry Table) reading for v3+ archives
  - ✅ BET (Block Entry Table) reading for v3+ archives

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

- ✅ **create** - Archive creation
  - configurable archive settings
  - custom and generated listfile

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
  - HET table information (v3+)
  - BET table information (v3+)

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
- ✅ Jenkins hash tests

#### Integration Tests

- ✅ Table parsing tests
- ✅ Builder functionality tests
- ✅ CLI command tests
- ✅ CRC validation tests
- ✅ Compression tests
- ✅ HET/BET table tests

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
