# Project Layout

This document describes the structure of the mopaq project.

## Directory Structure

```
mopaq/
├── mopaq/                   # Core MPQ library (Mo'PaQ)
│   ├── src/
│   │   ├── lib.rs           # Library entry point
│   │   ├── archive.rs       # Archive operations
│   │   ├── compression.rs   # Compression algorithms
│   │   ├── crypto.rs        # Encryption/decryption
│   │   ├── error.rs         # Error types
│   │   ├── hash.rs          # Hash algorithms
│   │   ├── io.rs            # I/O abstractions
│   │   └── tables.rs        # MPQ tables
│   ├── benches/             # Benchmarks
│   └── tests/               # Integration tests
│
├── storm-ffi/               # C API bindings (StormLib-compatible)
│   ├── src/
│   │   └── lib.rs          # FFI exports
│   ├── build.rs            # Header generation
│   └── include/            # Generated C headers
│
├── storm-cli/              # Command-line tool
│   └── src/
│       └── main.rs         # CLI implementation
│
├── test-data/              # Test MPQ archives
│   ├── v1/                 # Version 1 archives
│   ├── v2/                 # Version 2 archives
│   ├── v3/                 # Version 3 archives
│   └── v4/                 # Version 4 archives
│
├── docs/                   # Documentation
├── scripts/                # Utility scripts
│
├── Cargo.toml             # Workspace configuration
├── TODO.md                # Task tracking
├── README.md              # Project overview
└── LICENSE                # Licensing information
```

## Module Organization

### Core Library (`mopaq`)

The core library is named after the original Mo'PaQ (Mike O'Brien Pack) format and is organized into modules based on functionality:

- **archive**: High-level archive operations (open, create, list files)
- **compression**: All compression algorithms (zlib, bzip2, LZMA, etc.)
- **crypto**: Encryption table, encryption/decryption algorithms
- **error**: Error types and result aliases
- **hash**: MPQ hash algorithms and ASCII conversion tables
- **io**: I/O abstractions for reading/writing
- **tables**: Hash table, block table, HET, and BET structures

### FFI Library (`storm-ffi`)

Provides StormLib-compatible C API:

- Exports C functions matching StormLib's API
- Handles memory management for C compatibility
- Generates C headers automatically via cbindgen
- Output library is named `libstorm` for compatibility

### CLI Tool (`storm-cli`)

Command-line interface for archive operations:

- Uses clap for argument parsing
- Provides subcommands for common operations
- Includes debugging and inspection tools
- Binary is named `storm-cli` to avoid conflicts

## Build Artifacts

After building, you'll find:

- `target/debug/` - Debug builds
- `target/release/` - Release builds
- `target/doc/` - Generated documentation
- `storm-ffi/include/StormLib.h` - Generated C header

## Naming Rationale

- **mopaq**: Core library, referencing the original format name
- **storm-ffi**: Maintains StormLib naming for compatibility
- **storm-cli**: Command-line tool with storm branding
- **libstorm**: FFI output library for C/C++ compatibility
