# TODO - mopaq Implementation Tasks

## Core Library (`mopaq`)

### High Priority - Archive Writing

- [ ] **Encryption support in ArchiveBuilder**
  - [ ] Implement file encryption in `write_file` method
  - [ ] Add FIX_KEY flag support
  - [ ] Test encrypted file round-trips

- [ ] **Sector CRC support**
  - [ ] Generate CRC table for multi-sector files
  - [ ] Add CRC generation to ArchiveBuilder
  - [ ] Test CRC validation round-trips

- [ ] **Version 4 format support**
  - [ ] Implement v4 header writing with MD5 checksums
  - [ ] Calculate MD5 for tables (hash, block, hi-block)
  - [ ] Add MD5 header validation

### Medium Priority - Advanced Tables

- [X] **HET Table Reading (v3+)** ✅
  - [X] Complete HET table header parsing
  - [X] Implement HET hash table parsing
  - [X] Add bit-based file index parsing
  - [X] HET table encryption support
  - [X] HET table compression support

- [ ] **HET Table Writing (v3+)**
  - [ ] Implement HET table creation in ArchiveBuilder
  - [ ] Add Jenkins hash generation for new files
  - [ ] Implement bit-packing for file indices
  - [ ] Add HET table compression during write
  - [ ] Add HET table encryption during write

- [X] **BET Table Reading (v3+)** ✅
  - [X] Complete BET table header parsing
  - [X] Implement BET table entry bit extraction
  - [X] Add flag array parsing
  - [X] BET hash array parsing
  - [X] BET table encryption support
  - [X] BET table compression support

- [ ] **BET Table Writing (v3+)**
  - [ ] Implement BET table creation in ArchiveBuilder
  - [ ] Calculate optimal bit widths for fields
  - [ ] Implement bit-packing for table entries
  - [ ] Add BET table compression during write
  - [ ] Add BET table encryption during write

### Medium Priority - Digital Signatures

- [ ] **Weak Signature (v1+)**
  - [ ] RSASSA-PKCS1-v1_5 verification
  - [ ] MD5 hashing implementation
  - [ ] 512-bit RSA support
  - [ ] Signature file handling

- [ ] **Strong Signature (v2+)**
  - [ ] Strong signature detection
  - [ ] SHA-1 hashing implementation
  - [ ] 2048-bit RSA support
  - [ ] Custom padding verification (0x0B + 0xBB)
  - [ ] Little-endian to big-endian conversion

### Low Priority - Remaining Compression

- [ ] PKWare implode (0x00000100)
- [ ] Huffman encoding (0x01)
- [ ] PKWare DCL (0x08)
- [ ] ADPCM mono (0x40)
- [ ] ADPCM stereo (0x80)

### Archive Modification (Phase 2)

- [ ] In-place file addition/modification
- [ ] File deletion (mark as deleted)
- [ ] File replacement
- [ ] File renaming
- [ ] Archive compaction (remove deleted entries)
- [ ] `ArchiveMutator` type for read-write operations

### Special Files Support

- [X] `(attributes)` support ✅
  - [X] Attribute parsing ✅
  - [X] Per-file attribute access API ✅
  - [X] Manual attributes loading via `load_attributes()` ✅
  - [ ] Automatic attributes loading on archive open
  - [ ] Attribute generation in ArchiveBuilder
  - [ ] CRC32 calculation during file writing
  - [ ] MD5 calculation during file writing
- [ ] `(signature)` support (beyond basic parsing)
  - [ ] Weak signature generation
  - [ ] Strong signature generation
- [ ] `(user data)` support

### Performance & I/O

- [ ] Memory-mapped file support for writing
- [ ] Async I/O support
- [ ] Streaming API for large files
- [ ] Parallel compression for multiple files
- [ ] Encryption key caching
- [ ] Decompressed sector caching
- [ ] Hash lookup caching

### Builder Enhancements

- [ ] Add `add_directory` method to recursively add folders
- [ ] Add file filtering/exclusion patterns
- [ ] Add compression level configuration
- [ ] Support for adding files with specific block indices
- [ ] Validate filenames for MPQ compatibility
- [ ] Progress callback support for long operations
- [ ] Archive size estimation before creation

## CLI Tool (`storm-cli`)

### High Priority

- [X] Implement `create` command ✅
- [X] Add progress bars for long operations ✅
- [X] Add JSON output mode for scripting ✅
  - [X] JSON output infrastructure implemented ✅
  - [X] `list` command supports JSON output ✅
  - [X] `extract` command supports JSON output ✅
  - [ ] `create` command JSON output support

### Medium Priority

- [ ] `add` command - Add files to existing archive (Phase 2)
- [ ] `remove` command - Remove files from archive (Phase 2)
- [ ] Additional debug commands:
  - [ ] `headers` - Display all headers in detail
  - [ ] `decrypt` - Decrypt and display table/file
  - [ ] `hexdump` - Hex dump of archive sections
- [ ] Batch operations support
- [ ] Wildcard/glob pattern support

### Low Priority

- [ ] Performance statistics output
- [ ] Archive optimization tool
- [ ] Archive repair functionality

## FFI Library (`storm-ffi`)

### Core API Functions

- [X] `SFileOpenArchive` ✅
- [ ] `SFileCreateArchive`
- [X] `SFileCloseArchive` ✅
- [X] `SFileOpenFileEx` ✅
- [X] `SFileCloseFile` ✅
- [X] `SFileReadFile` ✅
- [X] `SFileGetFileSize` ✅
- [X] `SFileSetFilePointer` ✅
- [X] `SFileGetFileInfo` ✅
- [X] `SFileHasFile` ✅
- [X] `SFileGetArchiveName` ✅
- [X] `SFileGetFileName` ✅

### File Operations

- [ ] `SFileExtractFile`
- [ ] `SFileAddFile`
- [ ] `SFileAddFileEx`
- [ ] `SFileRemoveFile`
- [ ] `SFileRenameFile`
- [ ] `SFileCompactArchive`

### Verification & Enumeration

- [ ] `SFileVerifyFile`
- [ ] `SFileVerifyArchive`
- [X] `SFileEnumFiles` ✅
- [X] `SFileGetLocale` ✅
- [X] `SFileSetLocale` ✅
- [X] `GetLastError` ✅
- [X] `SetLastError` ✅

### Infrastructure

- [ ] C-compatible memory allocation
- [X] Handle management system ✅
- [X] Error code compatibility ✅
- [X] String handling (C strings) ✅
- [X] Function declarations in header (StormLib.h exists) ✅
- [ ] Constants and enums export

## Project-Level Tasks

### Documentation

- [ ] Architecture documentation
- [ ] Performance guide
- [ ] StormLib migration guide
- [ ] API examples and cookbook
- [ ] Format specification updates

### Testing & Quality

- [ ] Integration tests with real game archives
- [ ] Cross-validation with StormLib
- [ ] Fuzzing tests for security
- [ ] Performance benchmarks against StormLib
- [ ] Cross-platform testing (Windows, Linux, macOS)

### Project Infrastructure

- [ ] CI/CD pipeline improvements
- [ ] Release automation
- [ ] Performance regression tests
- [ ] Security audit process
- [ ] WASM support investigation

### Python Scripts

- [ ] StormLib comparison tool
- [ ] Archive analysis scripts
- [ ] Performance comparison scripts
- [ ] Test data generator (full archives with all features)

## Deferred Design Decisions

These design decisions are postponed until core functionality is complete:

### Archive Modification Strategy

- In-place modification support (vs current full rewrite)
- Free space tracking for in-place updates
- Fragmentation management
- Block allocation strategies

### Advanced Features

- Dynamic hash table resizing
- Automatic compression selection
- Streaming API design
- Concurrent access support
- Journal/WAL for atomic operations

### Compatibility Extensions

- UTF-8 filename support (non-standard)
- Custom filename validation rules
- Version upgrade/downgrade mechanisms

## Notes

Priority levels:

- **High Priority**: Features needed for v0.1 release
- **Medium Priority**: Features for v0.2-0.3 releases
- **Low Priority**: Nice-to-have features
- **Phase 2**: Requires architectural decisions

Current focus areas:

1. Complete archive creation (encryption, CRC, v4)
2. Begin FFI implementation for StormLib compatibility
3. Finish CLI tool basic functionality
