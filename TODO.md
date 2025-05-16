# MPQ Library Implementation TODO

## Project Setup

- [ ] Create project structure with Cargo, named `mopaq`
- [ ] Set up module organization (header, tables, crypto, compression)
- [ ] Configure Cargo.toml with dependencies and features
- [ ] Set up CI/CD for testing
- [ ] Create initial README.md with project overview

## Core Implementation

### Header Module

- [ ] Implement MpqHeader struct for format versions 1-4
- [ ] Create header parsing functions
- [ ] Implement user data detection
- [ ] Add header validation
- [ ] Write unit tests for header parsing

### Tables Module

#### Hash Table

- [ ] Implement HashEntry struct
- [ ] Create HashTable container
- [ ] Implement table reading from archive
- [ ] Add hash table validation
- [ ] Implement table decryption (with crypto module)
- [ ] Write unit tests for hash table

#### Block Table

- [ ] Implement BlockEntry struct
- [ ] Create BlockTable container
- [ ] Implement table reading from archive
- [ ] Add block table validation
- [ ] Implement table decryption (with crypto module)
- [ ] Write unit tests for block table

#### Extended Block Table

- [ ] Implement ExtBlockEntry struct
- [ ] Create ExtendedBlockTable container
- [ ] Implement table reading for v2+ archives
- [ ] Add extended block table validation
- [ ] Implement table decryption
- [ ] Write unit tests for extended block table

#### File Lookup

- [ ] Implement file lookup functions using hash table
- [ ] Add support for locale and platform specific lookups
- [ ] Implement efficient search strategies
- [ ] Write benchmarks for file lookup algorithms
- [ ] Write unit tests for lookup functionality

### Crypto Module

#### Constants

- [ ] Implement STORM_BUFFER_CRYPT table
- [ ] Define encryption constants (hash table key, block table key)
- [ ] Create helper functions for key usage

#### Hashing

- [ ] Implement HashType enum
- [ ] Create hash_string function for different hash types
- [ ] Implement compute_file_hashes function
- [ ] Add path normalization for hashing
- [ ] Write unit tests for hashing functions
- [ ] Add StormLib compatibility tests

#### Encryption

- [ ] Implement encrypt_block function
- [ ] Implement decrypt_block function
- [ ] Create key derivation functions
- [ ] Add helpers for table encryption/decryption
- [ ] Write unit tests for encryption
- [ ] Add StormLib compatibility tests

### Compression Module

#### Module Framework

- [ ] Define CompressionType enum
- [ ] Implement Compressor and Decompressor traits
- [ ] Create compression error types
- [ ] Implement multi-compression handling

#### Compression Methods

- [ ] Implement/wrap PKWARE DCL compression (priority)
- [ ] Implement/wrap Huffman compression
- [ ] Implement/wrap zlib compression
- [ ] Implement/wrap bzip2 compression
- [ ] Implement/wrap LZMA compression
- [ ] Implement sparse compression
- [ ] Implement IMA ADPCM compression
- [ ] Implement WAVE compression
- [ ] Write unit tests for each compression method
- [ ] Write benchmarks comparing compression methods

### Archive and File Handling

#### Archive Operations

- [ ] Implement MpqArchive struct
- [ ] Create archive opening function
- [ ] Implement header and table loading
- [ ] Add support for listfile parsing
- [ ] Implement file extraction methods
- [ ] Add archive validation
- [ ] Support for different MPQ versions

#### File Operations

- [ ] Implement MpqFile struct
- [ ] Create file reading functions
- [ ] Implement sector reading for large files
- [ ] Add support for single-unit files
- [ ] Implement file attribute handling
- [ ] Support for encrypted files
- [ ] Support for compressed files
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
