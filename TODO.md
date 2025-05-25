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

- [ ] Hash table structure parsing
- [ ] Hash table encryption/decryption
- [ ] Hash entry states (empty, deleted, occupied)
- [ ] Linear probing collision resolution
- [ ] Locale support in hash entries
- [ ] Platform support in hash entries
- [ ] Hash table size validation (power of 2)
- [ ] Hash table optimization for sparse tables

##### Block Table (v1+)

- [ ] Block table structure parsing
- [ ] Block table encryption/decryption
- [ ] Block table flags parsing
- [ ] File position calculation
- [ ] Compressed/uncompressed size handling

##### Hi-Block Table (v2+)

- [ ] Hi-block table parsing
- [ ] 64-bit file position calculation
- [ ] Integration with block table

##### HET Table (v3+)

- [ ] HET table header parsing
- [ ] HET hash table parsing
- [ ] Bit-based file index parsing
- [ ] Jenkins hash implementation for HET
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

- [ ] MPQ hash function implementation
  - [ ] Hash type 0: Table offset
  - [ ] Hash type 1: Name hash A
  - [ ] Hash type 2: Name hash B
  - [ ] Hash type 3: File key
  - [ ] Hash type 4: Key2 mix
- [ ] ASCII case conversion tables
  - [ ] Uppercase conversion table
  - [ ] Lowercase conversion table
- [ ] Path separator normalization (/ to \\)
- [ ] Jenkins hash for HET tables

#### File Encryption

- [ ] File key calculation
- [ ] FIX_KEY flag support
- [ ] Sector encryption/decryption
- [ ] Encryption key caching

### Compression Support

#### Compression Methods

- [ ] PKWare implode (0x00000100)
- [ ] Huffman encoding (0x01)
- [ ] Deflate/zlib (0x02)
- [ ] PKWare DCL (0x08)
- [ ] BZip2 (0x10) - v2+
- [ ] Sparse/RLE (0x20) - v3+
- [ ] ADPCM mono (0x40)
- [ ] ADPCM stereo (0x80)
- [ ] LZMA (0x12) - v3+
- [ ] Multiple compression support

#### Compression Infrastructure

- [ ] Sector-based compression
- [ ] Single unit file support
- [ ] Compression method detection
- [ ] Decompression dispatcher

### File Operations

#### File Storage

- [ ] Sector size calculation
- [ ] Sector offset table parsing
- [ ] Multi-sector file reading
- [ ] Single unit file reading
- [ ] Sector CRC validation
- [ ] Patch file support

#### File Management

- [ ] File addition
- [ ] File deletion (mark as deleted)
- [ ] File replacement
- [ ] File renaming
- [ ] Compact archive (remove deleted entries)

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
- [ ] Recovery mechanisms
- [ ] Validation functions

### Testing

- [ ] Unit tests for all hash functions
- [x] Unit tests for encryption/decryption
- [ ] Unit tests for each compression method
- [ ] Integration tests with test archives
- [ ] Fuzzing tests for security
- [x] Test vector validation (crypto)
- [ ] Cross-validation with StormLib

### Benchmarks

- [ ] Hash function benchmarks
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
- [ ] list - List files in archive
- [ ] extract - Extract files
- [ ] create - Create new archive
- [ ] add - Add files to archive
- [ ] remove - Remove files from archive
- [ ] verify - Verify archive integrity

### Debug Commands

- [x] info - Show archive information
- [x] crypto - Test crypto functions
- [ ] headers - Display all headers
- [ ] tables - Display table contents
- [ ] hash - Calculate hashes for filenames
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
- [ ] Test data generator (full archives)
- [ ] StormLib comparison tool
- [ ] Archive analysis scripts
- [ ] Performance comparison scripts

## Stretch Goals

- [ ] Archive repair functionality
- [ ] Archive optimization tool
- [ ] WASM support for web usage
