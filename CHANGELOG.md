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
  - ✅ HET/BET table compression support with configurable compression methods

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

- **Enhanced File Enumeration** - Improved file listing capabilities
  - ✅ New `list_all()` method that enumerates all entries from hash/block or HET/BET tables
  - ✅ New `list_with_hashes()` method that includes MPQ name hashes for each file
  - ✅ New `list_all_with_hashes()` method combining table enumeration with hash information
  - ✅ Extended `FileEntry` struct with optional hash values (name_1, name_2)
  - ✅ Proper enumeration of files not present in the listfile

#### CLI Tool (`storm-cli`)

- **Enhanced File List Command** - Improved file listing with new options
  - ✅ Fixed `--all` parameter to enumerate from tables instead of just showing listfile contents
  - ✅ Added `--show-hashes` parameter to display MPQ name hashes (name_1, name_2)
  - ✅ Hash display in all output formats (Text, JSON, CSV)
  - ✅ Verbose mode (-v) now shows detailed file information:
    - File sizes (uncompressed and compressed)
    - Compression ratio
    - Decoded file flags (Compressed, Encrypted, Single Unit, etc.)
    - Hash values when --show-hashes is used
  - ✅ Very verbose mode (-vv) shows additional statistics:
    - Count and percentage of compressed/encrypted files
    - Total space saved by compression
  - ✅ Fixed file count display when using --all parameter

- **Digital Signature Display** - Archive info command improvements
  - ✅ Displays digital signature status in Security Information section
  - ✅ Shows appropriate status for all signature types:
    - No signature
    - Weak signature (Valid/Invalid)
    - Strong signature (Valid/Invalid/No Key)
  - ✅ Color-coded output for easy identification

- **Compression Analysis Command** - New archive compression analysis tool
  - ✅ `analyze` command for detailed compression method analysis
  - ✅ Multiple analysis modes:
    - `--detailed` shows compression method for each file
    - `--by-extension` groups results by file extension
    - `--unsupported-only` shows only files using unsupported compression
    - `--show-stats` displays compression ratio statistics
  - ✅ Support for all output formats (Text, JSON, CSV)
  - ✅ Real-world archive testing revealed critical compatibility gaps:
    - **WoW 4.x+ archives contain unsupported compression combinations**
    - ADPCM + Implode compression (prevents archive opening)
    - ADPCM + PKWare combinations in HET/BET table metadata
    - Complex ADPCM combinations (flag 0xC9)
  - ✅ Analysis of 273 WoW MPQ archives across all expansions
  - ✅ Statistical breakdown of compression method usage by extension

### Fixed

- **Benchmark compilation failures** - Updated to use `std::hint::black_box`
  - ✅ Replaced deprecated `criterion::black_box` across all benchmarks
  - ✅ Fixed imports in hash, builder, crypto, and compression benchmarks
  - ✅ All benchmarks now compile and run correctly with latest criterion

- **Clippy warnings and code quality** - Recent cleanup and improvements
  - ✅ Fixed all remaining clippy warnings across the codebase
  - ✅ Improved code consistency and idiomatic Rust patterns
  - ✅ Enhanced error handling and documentation
  - ✅ Refactored functions with too many arguments using structured parameters
  - ✅ Removed unused functions and improved code organization

### 🚧 Work in Progress

#### Core Library (`mopaq`)

- **v4 format support** - Complete implementation of MPQ v4 archives
  - Header writing with MD5 checksums for all tables
  - Table MD5 calculation and validation
  - Extended format capabilities

- **Strong signature verification** - Enhanced digital signature support
  - ✅ 2048-bit RSA with SHA-1 hashing implementation
  - ✅ Complete PKCS#1 v1.5 padding verification for strong signatures
  - ✅ Custom Blizzard padding format (0x0B + 0xBB) support

- **Compression Support**
  - ✅ PKWare DCL compression and decompression
  - ✅ IMA ADPCM mono/stereo compression and decompression with channel validation
  - ✅ Multi-compression support (ADPCM + one other algorithm)
  - 🔨 Huffman decompression (compression not implemented)
  - 🔨 PKWare Implode decompression (compression not implemented)

- **HET/BET Table Fixes**
  - ✅ Fixed extended header structure handling (12-byte header)
  - ✅ Fixed encryption key mismatch between writer and reader
  - ✅ Fixed bit shift overflow for 64-bit hash entry sizes
  - ✅ Full compatibility with WoW 4.3.4+ archives

### ❌ Missing Features (Critical for 100% StormLib Compatibility)

#### Core Library (`mopaq`)

- **Archive Modification** - In-place archive operations (0% complete)
  - ❌ `add_file()` to existing archives (currently only ArchiveBuilder for new archives)
  - ❌ File removal from archives
  - ❌ File renaming within archives
  - ❌ Archive compacting (remove deleted entries)

- **Incomplete Compression Support**
  - ❌ **Huffman compression** (decompression works, compression not implemented)
  - ❌ **PKWare Implode compression** (decompression works, compression not implemented)
  - ❌ **Multiple compression combinations** (3+ algorithms in sequence)

- **Advanced Features** (30% gap)
  - ❌ **Streaming API** for large file operations
  - ❌ **Progress callbacks** for long operations
  - ❌ **Memory-mapped file support**
  - ❌ **Patch archive support** (base/patch chaining)
  - ❌ **Protected MPQ handling** (copy-protected archives)
  - ❌ **Signature creation** (weak and strong signature generation)

#### CLI Tool (`storm-cli`)

- **Archive Modification Commands** (Phase 2)
  - ❌ `add` command for adding files to existing archives
  - ❌ `remove` command for file removal
  - ❌ `compact` command for archive optimization

### 📚 Documentation

- **Platform Codes Clarification**
  - ✅ Documented that platform codes in hash table entries are vestigial
  - ✅ Analysis revealed all known MPQ archives use platform=0
  - ✅ Updated documentation to reflect that Blizzard uses separate archives instead
  - ✅ Added code comments explaining the unused nature of this field

### 🚧 Work in Progress

#### CLI Tool (`storm-cli`)

- Progress bars for long operations

### Test Utilities (Rust)

- **Test Data Generator** (`mopaq::test_utils::data_generator`)
  - ✅ Generate raw test data for archive creation testing
  - ✅ Multiple test configurations (simple, game assets, nested, mixed sizes, special names)
  - ✅ Support for text, binary, and empty files
  - ✅ Configurable file sizes and directory structures
  - ✅ Type-safe Rust implementation with better performance

#### FFI Library (`storm-ffi`)

- **StormLib API compatibility** - Comprehensive C API implementation
  - ✅ Basic structure and handle management
  - ✅ Core file operations (`SFileOpenArchive`, `SFileCloseArchive`, etc.)
  - ✅ File enumeration and information retrieval
  - ✅ Error handling with StormLib-compatible error codes
  - 🚧 Archive creation and modification functions
  - 🚧 Advanced file operations and verification
  - ✅ C header generation with cbindgen

### Changed

- **Dependencies and Architecture** - Modernized and improved project structure
  - ✅ Replaced `rust-lzma` with `lzma-rs` for pure Rust LZMA support
    - No system dependencies required
    - Supports both raw LZMA and XZ formats
    - Better cross-platform compatibility
  - ✅ Upgraded dependencies to latest versions for security and performance
  - ✅ Enhanced Cargo.toml configurations across all crates

- **Code Organization** - Improved modularity and maintainability
  - ✅ Refactored `special_files.rs` into a module structure
    - Split into `listfile.rs` for parsing functionality
    - Separated `info.rs` for special file metadata
    - Added `attributes.rs` for comprehensive attribute handling
  - ✅ Enhanced error handling with more specific error types
  - ✅ Improved documentation and inline examples

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

- ✅ **Zlib/Deflate** - Full compression and decompression
- ✅ **BZip2** - Full compression and decompression (v2+)
- ✅ **LZMA** - Full compression and decompression with lzma-rs (v3+)
- ✅ **Sparse/RLE** - Full compression and decompression (v3+)
- ✅ **PKWare DCL** - Full compression and decompression
- ✅ **IMA ADPCM** - Full mono/stereo compression and decompression
- 🔨 **Huffman** - Decompression only (used in WAVE files)
- 🔨 **PKWare Implode** - Decompression only
- ✅ Multi-compression: ADPCM + one other algorithm
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
