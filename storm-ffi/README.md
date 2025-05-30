# storm-ffi

StormLib-compatible C API bindings for the mopaq MPQ archive library.

[![Crates.io](https://img.shields.io/crates/v/storm-ffi.svg)](https://crates.io/crates/storm-ffi)
[![Documentation](https://docs.rs/storm-ffi/badge.svg)](https://docs.rs/storm-ffi)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../LICENSE)

## Overview

`storm-ffi` provides a C-compatible API that aims to be a drop-in replacement for the popular [StormLib](https://github.com/ladislav-zezula/StormLib) library. This allows existing C/C++ applications that use StormLib to switch to the Rust-based mopaq implementation with minimal code changes.

## Features

- **StormLib API Compatibility**: Implements the same function signatures as StormLib
- **Safe Rust Implementation**: All the safety and performance benefits of the mopaq library
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Static and Dynamic Linking**: Supports both static and dynamic library builds

## Status

⚠️ **This library is currently in development.** Not all StormLib functions are implemented yet. See the [API Coverage](#api-coverage) section for details.

## Building

### Prerequisites

- Rust toolchain (1.86.0 or later)
- C compiler (for building examples)

### Build the Library

```bash
# Build dynamic library (.so/.dylib/.dll)
cargo build --package storm-ffi

# Build static library (.a/.lib)
cargo build --package storm-ffi --release
```

The built libraries will be in:

- Dynamic: `target/debug/libstorm.{so,dylib,dll}`
- Static: `target/release/libstorm.a`

### Build Examples

```bash
cd storm-ffi/examples
gcc -o basic basic.c -L../../target/debug -lstorm
./basic path/to/archive.mpq
```

## Usage

### C/C++ Integration

1. Include the header file:

```c
#include "StormLib.h"
```

2. Link against the library:
   - Dynamic: `-lstorm`
   - Static: Link the `.a` file directly

3. Use the StormLib API:

```c
HANDLE hMpq = NULL;
if (SFileOpenArchive("archive.mpq", 0, 0, &hMpq)) {
    // Work with the archive
    SFileCloseArchive(hMpq);
}
```

### CMake Integration

```cmake
# Find the storm library
find_library(STORM_LIB storm PATHS /path/to/storm-ffi/target/release)

# Link to your target
target_link_libraries(your_target ${STORM_LIB})
```

## API Coverage

### Implemented Functions

- [x] `SFileOpenArchive` - Open an MPQ archive
- [x] `SFileCloseArchive` - Close an MPQ archive
- [x] `GetLastError` - Get the last error code

### Planned Functions

- [ ] `SFileOpenFileEx` - Open a file from archive
- [ ] `SFileReadFile` - Read file data
- [ ] `SFileCloseFile` - Close an open file
- [ ] `SFileHasFile` - Check if file exists
- [ ] `SFileGetFileSize` - Get file size
- [ ] `SFileExtractFile` - Extract file to disk
- [ ] `SFileCreateArchive` - Create new archive
- [ ] `SFileAddFile` - Add file to archive
- [ ] `SFileCompactArchive` - Compact archive
- [ ] `SFileSetFileLocale` - Set file locale
- [ ] And many more...

## Examples

See the [examples](examples/) directory for sample code:

- [basic.c](examples/basic.c) - Basic archive operations
- [storm_example.c](examples/storm_example.c) - More comprehensive example

## Documentation

- [API Reference](https://docs.rs/storm-ffi)
- [StormLib Documentation](http://www.zezula.net/en/mpq/stormlib.html) (for API compatibility reference)

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
