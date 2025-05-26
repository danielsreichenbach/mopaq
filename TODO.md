# TODO - mopaq Implementation Tasks

## Project Setup

- [x] Workspace structure with three crates (mopaq, storm-ffi, storm-cli)
- [x] Cargo.toml configuration for all crates
- [x] Basic library structure and module layout
- [x] Error types definition
- [x] License files (MIT + Apache 2.0)
- [x] README with project overview
- [x] CI pipeline configuration
- [x] Git ignore configuration
- [x] Contributing guidelines
- [x] Code formatting configuration (rustfmt.toml)
- [x] Development Makefile
- [x] Module stub files for compilation
- [x] Test data directory structure
- [x] Editor configuration (.editorconfig)
- [x] Fix compilation issues (edition 2021)
- [x] Rename CLI binary to storm-cli to avoid conflicts
- [x] Rename core library to mopaq to avoid FFI conflicts
- [x] Changelog started

## Core Library (`storm`)

### Archive Structure Support

#### Headers and Version Support

- [x] MPQ header parsing (v1: 32 bytes)
- [x] MPQ header parsing (v2: 44 bytes) - Extended with Hi-block table support
- [x] MPQ header parsing (v3: 68 bytes) - Extended with HET/BET support
- [x] MPQ header parsing (v4: 208 bytes) - Extended with MD5 checksums
- [x] User data header support (`MPQ\x1B` signature)
- [x] Header location algorithm (512-byte aligned scanning)
- [x] Archive size calculation for v2+ (using 64-bit values)
- [ ] Write header support for archive creation

#### Table Implementations

##### Hash Table (v1+)

- [x] Hash table structure parsing
- [x] Hash table encryption/decryption
- [x] Hash entry states (empty, deleted, occupied)
- [x] Linear probing collision resolution
- [x] Locale support in hash entries
- [x] Platform support in hash entries
- [x] Hash table size validation (power of 2)
- [ ] Hash table optimization for sparse tables

##### Block Table (v1+)

- [x] Block table structure parsing
- [x] Block table encryption/decryption
- [x] Block table flags parsing
- [x] File position calculation
- [x] Compressed/uncompressed size handling

##### Hi-Block Table (v2+)

- [x] Hi-block table parsing
- [x] 64-bit file position calculation
- [x] Integration with block table

##### HET Table (v3+)

- [ ] HET table header parsing
- [ ] HET hash table parsing
- [ ] Bit-based file index parsing
- [x] Jenkins hash implementation for HET
- [ ] HET table encryption support
- [ ] HET table compression support

##### BET Table (v3+)

- [ ] BET table header parsing
- [ ] BET table entry bit extraction
- [ ] Flag array parsing
- [ ] BET hash array parsing
- [ ] BET table encryption support
- [ ] BET table compression support

### Cryptography

#### Encryption Table

- [x] Generate static encryption table (1280 values)
- [x] Implement encryption algorithm
- [x] Implement decryption algorithm
- [x] Single DWORD decryption function

#### Hash Functions

- [x] MPQ hash function implementation
  - [x] Hash type 0: Table offset
  - [x] Hash type 1: Name hash A
  - [x] Hash type 2: Name hash B
  - [x] Hash type 3: File key
  - [x] Hash type 4: Key2 mix
- [x] ASCII case conversion tables
  - [x] Uppercase conversion table
  - [x] Lowercase conversion table
- [x] Path separator normalization (/ to \\)
- [x] Jenkins hash for HET tables

#### File Encryption

- [x] File key calculation
- [x] FIX_KEY flag support
- [x] Sector encryption/decryption
- [ ] Encryption key caching

### Compression Support

#### Compression Methods

- [ ] PKWare implode (0x00000100)
- [ ] Huffman encoding (0x01)
- [x] Deflate/zlib (0x02)
- [ ] PKWare DCL (0x08)
- [x] BZip2 (0x10) - v2+
- [x] Sparse/RLE (0x20) - v3+
- [ ] ADPCM mono (0x40)
- [ ] ADPCM stereo (0x80)
- [x] LZMA (0x12) - v3+ (basic support)
- [x] Multiple compression support (partial - PKWare not implemented)

#### Compression Infrastructure

- [x] Sector-based compression
- [x] Single unit file support
- [x] Compression method detection
- [x] Decompression dispatcher

### File Operations

#### File Storage

- [x] Sector size calculation
- [x] Sector offset table parsing
- [x] Multi-sector file reading
- [x] Single unit file reading
- [ ] Sector CRC validation
- [ ] Patch file support

#### File Management

- [ ] File addition
- [ ] File deletion (mark as deleted)
- [ ] File replacement
- [ ] File renaming
- [ ] Compact archive (remove deleted entries)
- [x] File lookup by name (find_file implemented)

### Special Files

- [ ] `(listfile)` support
  - [ ] Listfile parsing
  - [ ] Listfile generation
  - [ ] Filename index building
- [ ] `(attributes)` support
  - [ ] Attribute parsing
  - [ ] Attribute generation
- [ ] `(signature)` support
  - [ ] Weak signature parsing
- [ ] `(user data)` support

### Digital Signatures

#### Weak Signature (v1+)

- [ ] RSASSA-PKCS1-v1_5 verification
- [ ] MD5 hashing
- [ ] 512-bit RSA support
- [ ] Signature file handling

#### Strong Signature (v2+)

- [ ] Strong signature detection
- [ ] SHA-1 hashing
- [ ] 2048-bit RSA support
- [ ] Custom padding verification (0x0B + 0xBB)
- [ ] Little-endian to big-endian conversion
- [ ] Signature location after archive

### I/O and Performance

#### I/O Abstractions

- [ ] Buffered reading
- [ ] Memory-mapped file support
- [ ] Async I/O support
- [ ] Zero-copy operations where possible
- [ ] Streaming support for large files

#### Caching

- [ ] Encryption key caching
- [ ] Decompressed sector caching
- [ ] Hash lookup caching
- [ ] Metadata caching

### Error Handling

- [x] Comprehensive error types
- [x] Error context propagation
- [x] Table-specific error types (hash_table, block_table)
- [ ] Recovery mechanisms
- [ ] Validation functions

### Testing

- [x] Unit tests for all hash functions
- [x] Unit tests for encryption/decryption
- [x] Unit tests for table structures
- [x] Integration tests for table parsing
- [x] Unit tests for compression methods (zlib, bzip2, sparse)
- [ ] Integration tests with test archives
- [ ] Fuzzing tests for security
- [x] Test vector validation (crypto)
- [x] Test vector validation (hash functions)
- [ ] Cross-validation with StormLib

### Benchmarks

- [x] Hash function benchmarks
- [x] Encryption/decryption benchmarks
- [ ] Compression method benchmarks
- [ ] File extraction benchmarks
- [ ] Archive creation benchmarks

## FFI Library (`storm-ffi`)

### StormLib API Compatibility

- [x] Basic FFI structure and types
- [x] Error code definitions
- [ ] SFileOpenArchive
- [ ] SFileCreateArchive
- [ ] SFileCloseArchive
- [ ] SFileOpenFileEx
- [ ] SFileCloseFile
- [ ] SFileReadFile
- [ ] SFileGetFileSize
- [ ] SFileSetFilePointer
- [ ] SFileGetFileInfo
- [ ] SFileExtractFile
- [ ] SFileAddFile
- [ ] SFileAddFileEx
- [ ] SFileRemoveFile
- [ ] SFileRenameFile
- [ ] SFileCompactArchive
- [ ] SFileVerifyFile
- [ ] SFileVerifyArchive
- [ ] SFileEnumFiles
- [ ] SFileGetLocale
- [ ] SFileSetLocale

### Memory Management

- [ ] C-compatible memory allocation
- [ ] Handle management
- [ ] Error code compatibility
- [ ] String handling (C strings)

### Header Generation

- [x] Automatic C header generation (build.rs with cbindgen)
- [x] Type definitions
- [ ] Function declarations
- [ ] Constants and enums

## CLI Tool (`storm-cli`)

### Basic Commands

- [x] CLI argument parsing structure
- [x] Basic CLI integration tests
- [x] list - List files in archive
- [x] extract - Extract files
- [ ] create - Create new archive
- [ ] add - Add files to archive
- [ ] remove - Remove files from archive
- [x] verify - Verify archive integrity

### Debug Commands

- [x] info - Show archive information
- [x] crypto - Test crypto functions
- [x] hash - Calculate hashes for filenames
- [x] hash-compare - Compare hashes between filenames
- [x] tables - Display table contents
- [ ] headers - Display all headers
- [ ] decrypt - Decrypt and display table/file
- [ ] hexdump - Hex dump of archive sections

### Features

- [ ] Progress bars for long operations
- [ ] JSON output mode
- [ ] Verbose debugging output
- [ ] Performance statistics
- [ ] Batch operations
- [ ] Wildcard support

## Documentation

- [x] API documentation (inline doc comments)
- [x] Project layout documentation
- [x] Build instructions
- [x] CLI usage guide
- [x] Naming conventions documentation
- [x] Binary rename summary
- [x] Debug info command documentation
- [x] Implementation summary
- [x] Crypto implementation documentation
- [x] Crypto implementation summary
- [x] Hash implementation documentation
- [x] Debug commands documentation
- [x] Table parsing documentation
- [ ] Architecture documentation
- [ ] Performance guide
- [ ] Debugging guide
- [ ] Migration guide from StormLib

## Infrastructure

- [x] CI/CD pipeline setup
- [ ] Release automation
- [ ] Cross-platform testing (Windows, Linux, macOS)
- [ ] Performance regression tests
- [ ] Security audit process

## Python Scripts

- [x] Build verification script
- [x] CLI name test script
- [x] Test data generator (basic MPQ headers)
- [x] Test encryption table validation script
- [ ] Test data generator (full archives)
- [ ] StormLib comparison tool
- [ ] Archive analysis scripts
- [ ] Performance comparison scripts

## Stretch Goals

- [ ] Archive repair functionality
- [ ] Archive optimization tool
- [ ] WASM support for web usage
