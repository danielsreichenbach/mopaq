# Naming Conventions

This document describes the naming conventions used in the StormLib-rs project.

## Crate Names

- `mopaq` - Core library crate (named after Mo'PaQ format)
- `storm-ffi` - FFI bindings crate (# Naming Conventions

This document describes the naming conventions used in the StormLib-rs project.

## Crate Names

- `storm` - Core library crate
- `storm-ffi` - FFI bindings crate
- `storm-cli` - Command-line interface crate

## Binary Names

- `storm-cli` - The command-line tool binary
  - Named to avoid conflicts with the `storm` library crate
  - Installed via `cargo install --path storm-cli`

## Library Names

- `libstorm` - The FFI library output
  - `libstorm.so` on Linux
  - `libstorm.dylib` on macOS
  - `storm.dll` on Windows

## Module Structure

Within the `storm` crate:

- `archive` - Archive operations
- `compression` - Compression algorithms
- `crypto` - Cryptographic functions
- `error` - Error types
- `hash` - Hash algorithms
- `io` - I/O abstractions
- `tables` - MPQ table structures

## File Naming

- Source files use snake_case: `archive.rs`, `hash_table.rs`
- Test files: `<module>_test.rs` or in `tests/` directory
- Benchmark files: `<feature>_bench.rs` in `benches/` directory

## Constants and Types

- Constants: `UPPER_SNAKE_CASE` (e.g., `MPQ_ARCHIVE`, `HASH_TABLE_SIZE`)
- Types: `PascalCase` (e.g., `Archive`, `HashEntry`, `FormatVersion`)
- Functions: `snake_case` (e.g., `open_archive`, `calculate_hash`)

## FFI Functions

FFI functions maintain StormLib compatibility:

- `SFileOpenArchive` - PascalCase with 'S' prefix
- Parameters use camelCase to match C conventions

## Command-Line Interface

Commands use kebab-case:

- `storm-cli list`
- `storm-cli extract`
- `storm-cli create`

Options use kebab-case:

- `--output-dir`
- `--compression-type`
