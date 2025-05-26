# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial project structure with three crates: mopaq, storm-ffi, storm-cli
- Basic module structure for MPQ functionality
- Error types and result aliases
- CI/CD pipeline with GitHub Actions
- Documentation structure
- Development tooling (Makefile, scripts)
- MPQ header parsing for all versions (v1-v4)
- User data header support
- Header location algorithm (512-byte aligned scanning)
- CLI debug info command to display archive information
- Test MPQ file generator script
- Complete encryption table generation (1280 values)
- Encryption and decryption algorithms with test vectors
- CLI debug crypto command for testing crypto functions
- Comprehensive crypto benchmarks and tests
- Complete MPQ hash function implementation with all hash types
- ASCII case conversion tables (uppercase/lowercase)
- Path separator normalization in hash functions
- Jenkins hash implementation for HET tables
- Hash function benchmarks
- Test vector validation for hash functions
- CLI debug hash command to generate hash values
- CLI debug hash-compare command to compare hashes between files
- Hash table structure parsing and decryption
- Block table structure parsing and decryption
- Hi-block table support for archives > 4GB
- File lookup functionality (find_file method)
- CLI debug tables command to display table contents
- Table entry state tracking (valid/deleted/empty)
- Compression module implementation with multiple algorithms
- Zlib compression/decompression support
- BZip2 compression/decompression support
- Sparse/RLE decompression support
- LZMA decompression support (basic)
- Multi-sector file reading with compression
- File encryption/decryption support with key calculation
- CLI extract command (basic implementation)
- Sector offset table parsing and decryption
- Single unit and multi-sector file handling
- Compression benchmarks for performance testing
- Integration tests for compression functionality
- Refactored CLI commands into separate modules for better organization
- CLI list command with verbose and all-entries options
- CLI verify command for archive integrity checking
- CLI find command to search for specific files with detailed information
- Special file handling introduced for listfiles

### Changed

- CLI binary renamed from `storm` to `storm-cli` to avoid naming conflicts with the library crate
- Core library renamed from `storm` to `mopaq` to avoid conflicts with FFI output
- Encryption table generation changed from `once_cell::Lazy` to `const fn` for compile-time generation

### Technical Details

- Using Rust edition 2021 with MSRV 1.86
- Dual-licensed under MIT and Apache 2.0
- StormLib-compatible FFI interface planned
- Crypto implementation uses const fn for compile-time table generation
- Compression uses feature flags for optional algorithms (bzip2, lzma)
