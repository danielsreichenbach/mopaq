# MPQ Library Implementation TODO

## Project Setup

- [X] Create project structure with Cargo
- [X] Set up module organization (header, tables, crypto, compression)
- [X] Configure Cargo.toml with dependencies and features
- [X] Set up CI/CD for testing
- [X] Create initial README.md with project overview

## Core Implementation

### Header Module

- [X] Implement MpqHeader struct for format versions 1-4
- [X] Create header parsing functions
- [X] Implement user data detection
- [X] Add header validation
- [X] Write unit tests for header parsing

### Tables Module

#### Hash Table

- [X] Implement HashEntry struct
- [X] Create HashTable container
- [X] Implement table reading from archive
- [X] Add hash table validation
- [X] Implement table decryption (with crypto module)
- [X] Write unit tests for hash table

#### Block Table

- [X] Implement BlockEntry struct
- [X] Create BlockTable container
- [X] Implement table reading from archive
- [X] Add block table validation
- [X] Implement table decryption (with crypto module)
- [X] Write unit tests for block table

#### Extended Block Table

- [X] Implement ExtBlockEntry struct
- [X] Create ExtendedBlockTable container
- [X] Implement table reading for v2+ archives
- [X] Add extended block table validation
- [X] Implement table decryption
- [X] Write unit tests for extended block table

#### File Lookup

- [X] Implement file lookup functions using hash table
- [X] Add support for locale and platform specific lookups
- [X] Implement efficient search strategies
- [X] Write benchmarks for file lookup algorithms
- [X] Write unit tests for lookup functionality

### Crypto Module

#### Constants

- [X] Implement STORM_BUFFER_CRYPT table
- [X] Define encryption constants (hash table key, block table key)
- [X] Create helper functions for key usage

#### Hashing

- [X] Implement HashType enum
- [X] Create hash_string function for different hash types
- [X] Implement compute_file_hashes function
- [X] Add path normalization for hashing
- [X] Write unit tests for hashing functions
- [X] Add StormLib compatibility tests

#### Encryption

- [X] Implement encrypt_block function
- [X] Implement decrypt_block function
- [X] Create key derivation functions
- [X] Add helpers for table encryption/decryption
- [X] Write unit tests for encryption
- [X] Add StormLib compatibility tests

### Compression Module

#### Module Framework

- [X] Define CompressionType enum
- [X] Implement Compressor and Decompressor traits
- [X] Create compression error types
- [X] Implement multi-compression handling

#### Compression Methods

- [X] Implement/wrap PKWARE DCL compression (priority)
- [X] Implement/wrap Huffman compression
- [X] Implement/wrap zlib compression
- [X] Implement/wrap bzip2 compression
- [X] Implement/wrap LZMA compression
- [ ] Implement sparse compression
- [ ] Implement IMA ADPCM compression
- [ ] Implement WAVE compression
- [X] Write unit tests for each compression method
- [ ] Write benchmarks comparing compression methods

### Archive and File Handling

#### Archive Operations

- [X] Implement MpqArchive struct
- [X] Create archive opening function
- [X] Implement header and table loading
- [X] Add support for listfile parsing
- [X] Implement file extraction methods
- [X] Add archive validation
- [ ] Support for different MPQ versions

#### File Operations

- [X] Implement MpqFile struct
- [X] Create file reading functions
- [X] Implement sector reading for large files
- [X] Add support for single-unit files
- [x] Implement file attribute handling
- [X] Support for encrypted files
- [X] Support for compressed files
- [ ] Write unit tests for file operations

## Advanced Features

### HET/BET Tables (v3+)

- [ ] Implement HET (Hash Entry Table) structure
- [ ] Implement BET (Block Entry Table) structure
- [ ] Create loading functions for HET/BET
- [ ] Implement file lookup using HET
- [ ] Add file info retrieval from BET
- [ ] Write unit tests for HET/BET functionality

### Patch Support

- [ ] Implement patch file detection
- [ ] Create patch file handling
- [ ] Implement patch application
- [ ] Add support for incremental patching
- [ ] Write unit tests for patch functionality

### Advanced MPQ Features

- [ ] Implement support for attributes
- [ ] Add signature verification
- [ ] Implement solid compression (v4)
- [ ] Support for checksums (v4)
- [ ] Add weak signature verification
- [ ] Write unit tests for advanced features

### Archive Creation

- [ ] Implement archive creation functions
- [ ] Add file addition methods
- [ ] Implement table creation
- [ ] Support for setting file attributes
- [ ] Add file compression on addition
- [ ] Support for file encryption
- [ ] Write unit tests for archive creation

## Testing, Documentation and Optimization

### Comprehensive Testing

- [ ] Write integration tests with real MPQ files
- [ ] Create test suite for StormLib compatibility
- [ ] Add fuzz testing for robustness
- [ ] Implement error case testing
- [ ] Create benchmarks for key operations

### Documentation

- [ ] Write module-level documentation
- [ ] Add function-level documentation
- [ ] Create usage examples
- [ ] Document MPQ format details
- [ ] Add tutorials for common operations

### Performance Optimization

- [ ] Profile and optimize file lookup
- [ ] Improve compression/decompression speed
- [ ] Optimize memory usage
- [ ] Add parallel processing where applicable
- [ ] Create benchmarks to verify improvements
