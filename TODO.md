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

## Core Library (`mopaq`)

### Immediate Implementation Priorities

Based on our design decisions, focusing on simple, safe approach first:

1. **Archive Creation (Phase 1)**
   - [ ] `ArchiveBuilder` type (write-only, separate from `Archive`)
   - [ ] Full archive rewrite strategy (no in-place modification)
   - [ ] Explicit compression settings per file
   - [ ] User-specified hash table size
   - [ ] Temp file + atomic rename for safety
   - [ ] Basic v1 format support first

2. **API Design**
   - [ ] Clear separation: `Archive` (read), `ArchiveBuilder` (write)
   - [ ] Explicit over implicit (no automatic compression/version detection)
   - [ ] Builder pattern for configuration
   - [ ] No concurrent access support initially

3. **Error Handling**
   - [ ] Simple rollback on failure
   - [ ] Clear error messages for unsupported operations
   - [ ] Document thread-safety limitations

### New Tasks Identified

#### Immediate Follow-ups

- [ ] Add encryption support to ArchiveBuilder
  - [ ] Implement FIX_KEY flag support
  - [ ] Test encrypted file round-trips
- [ ] Add sector CRC support for file integrity
- [ ] Implement v4 header writing with MD5 checksums
- [ ] Add progress callback support for long operations

#### Builder Enhancements

- [ ] Add `add_directory` method to recursively add folders
- [ ] Add file filtering/exclusion patterns
- [ ] Add compression level configuration
- [ ] Support for adding files with specific block indices
- [ ] Validate filenames for MPQ compatibility

#### Performance Optimizations

- [ ] Parallel compression for multiple files
- [ ] Streaming API for very large files
- [ ] Memory usage limits and buffering strategies

#### Error Handling Improvements

- [ ] Better error messages for common failures
- [ ] Validate hash table collisions before writing
- [ ] Check for filesystem errors during write
- [ ] Add archive size estimation before creation

### Archive Structure Support

#### Headers and Version Support

- [x] MPQ header parsing (v1: 32 bytes)
- [x] MPQ header parsing (v2: 44 bytes) - Extended with Hi-block table support
- [x] MPQ header parsing (v3: 68 bytes) - Extended with HET/BET support
- [x] MPQ header parsing (v4: 208 bytes) - Extended with MD5 checksums
- [x] User data header support (`MPQ\x1B` signature)
- [x] Header location algorithm (512-byte aligned scanning)
- [x] Archive size calculation for v2+ (using 64-bit values)
- [x] Write header support for archive creation (v1, v2, v3)
- [ ] Write header support for v4 (MD5 checksums)

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
- [X] Sector CRC validation
- [ ] Sector-based file writing (part of ArchiveBuilder)
- [ ] Sector offset table generation (part of ArchiveBuilder)
- [ ] Patch file support (Phase 2)

#### File Management (Phase 1 - Simple Approach)

- [x] Archive creation with full rewrite strategy
  - [x] Implement `ArchiveBuilder` type (write-only)
  - [x] Add files with explicit compression settings
  - [x] Write to temp file and atomic rename
  - [x] Automatic hash table sizing based on file count
  - [x] Basic v1-v3 format support
- [x] Basic error recovery (rollback on failure via temp file)
- [x] File addition API with builder pattern
- [x] Listfile generation
  - [x] Automatic generation from added files
  - [x] External listfile support
  - [x] Option to omit listfile
- [x] File lookup by name (find_file implemented)

#### File Management (Phase 2 - Deferred)

- [ ] In-place file addition/modification
- [ ] File deletion (mark as deleted)
- [ ] File replacement
- [ ] File renaming
- [ ] Compact archive (remove deleted entries)
- [ ] `ArchiveMutator` type for read-write operations

### Special Files

- [x] `(listfile)` support
  - [x] Listfile parsing
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
- [x] Integration tests for archive creation (ArchiveBuilder)
- [x] Round-trip tests (create → write → read → verify)
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
- [ ] create - Create new archive (depends on ArchiveBuilder implementation)
- [ ] add - Add files to archive (Phase 2 - requires in-place modification)
- [ ] remove - Remove files from archive (Phase 2 - requires in-place modification)
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
- [x] Test data generator (basic MPQ headers)
- [x] Test encryption table validation script
- [x] Test data generator with compression
- [x] Test data generator with CRC validation
- [X] Test data generator (full archives)
- [ ] StormLib comparison tool
- [ ] Archive analysis scripts
- [ ] Performance comparison scripts

## Stretch Goals

- [ ] Archive repair functionality
- [ ] Archive optimization tool
- [ ] WASM support for web usage

## Deferred Design Decisions

These are design decisions and features we've identified but are deferring to later iterations:

### Archive Modification Strategy

- [ ] In-place modification support (vs current full rewrite)
- [ ] Free space tracking for in-place updates
- [ ] Fragmentation management
- [ ] Block allocation strategies (first-fit, best-fit, etc.)

### Advanced Hash Table Management

- [ ] Dynamic hash table resizing based on load factor
- [ ] Configurable load factor thresholds
- [ ] Better collision handling strategies
- [ ] Hash table optimization for pathological cases

### Compression Policies

- [ ] Automatic compression based on file type/size
- [ ] Try multiple compression methods and pick best
- [ ] Compression exclusion lists
- [ ] Per-file-type compression strategies

### Memory Management

- [ ] Streaming API for large files
- [ ] Memory-mapped file support for writing
- [ ] Chunked processing for reduced memory usage
- [ ] Configurable memory limits

### Concurrent Access

- [ ] File-based locking mechanisms
- [ ] Read-write lock support
- [ ] Multi-process safety
- [ ] Transaction log for concurrent modifications

### Advanced Error Recovery

- [ ] Journal/WAL approach for atomic operations
- [ ] Incremental backup of tables
- [ ] Corruption recovery tools
- [ ] Partial write recovery

### File Name Handling

- [ ] UTF-8 filename support (non-standard)
- [ ] Configurable path normalization
- [ ] Locale-aware case handling
- [ ] Custom filename validation rules

### Version Management

- [ ] Automatic version upgrade when needed
- [ ] Feature compatibility matrix
- [ ] Version downgrade with feature loss warnings
- [ ] Version-specific optimizations
