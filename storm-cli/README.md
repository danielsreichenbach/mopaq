# storm-cli

A modern command-line interface for working with MPQ (Mo'PaQ) archives, providing StormLib-compatible functionality with a Rust implementation.

## Features

- **List** files in MPQ archives with detailed information
- **Extract** individual files or entire archives
- **Create** new MPQ archives with multiple compression options
- **Find** specific files and display hash information
- **Verify** archive integrity and report issues
- **Debug** tools for inspecting archive internals
- Multiple output formats (text, JSON, CSV)
- Cross-platform support (Windows, Linux, macOS)
- Shell completion support

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/danielsreichenbach/mopaq.git
cd mopaq

# Build and install
cargo install --path storm-cli
```

### Build from Crates.io

```bash
cargo install storm-cli
```

## Usage

Storm-cli uses a grouped command structure for better organization:

### Basic Commands

```bash
# Archive operations
storm-cli archive create game.mpq source/ --compression zlib
storm-cli archive info game.mpq
storm-cli archive verify game.mpq

# File operations
storm-cli file list game.mpq
storm-cli file list game.mpq --all              # Show ALL entries from tables
storm-cli file list game.mpq --show-hashes      # Display MPQ name hashes
storm-cli file list game.mpq -v                 # Verbose with sizes and flags
storm-cli file extract game.mpq war3map.j -o extracted/
storm-cli file find game.mpq "*.mdx"
storm-cli file info game.mpq war3map.j

# Table operations
storm-cli table show game.mpq --type hash
storm-cli table analyze game.mpq

# Hash utilities
storm-cli hash generate "war3map.j" --all
storm-cli hash compare file1.txt file2.txt
```

For detailed command structure, see [COMMAND_STRUCTURE.md](COMMAND_STRUCTURE.md).

### Advanced Options

```bash
# List with pattern filtering
storm-cli file list game.mpq --pattern "*.blp" --regex

# Create with specific version and block size
storm-cli archive create archive.mpq files/ --version 2 --block-size 4

# Extract all files preserving paths
storm-cli file extract game.mpq --preserve-path

# Output as JSON
storm-cli file list game.mpq -o json

# Verbose output
storm-cli file extract game.mpq -vv

# Exclude files when creating
storm-cli archive create archive.mpq src/ --ignore "*.tmp" --ignore "*.log"

# Verify with CRC checking
storm-cli archive verify game.mpq --check-crc --check-contents
```

### Advanced Commands

```bash
# Show specific table contents
storm-cli table show game.mpq --table-type block --limit 50

# Analyze table efficiency
storm-cli table analyze game.mpq --detailed

# Generate specific hash type
storm-cli hash generate "war3map.j" --type file-key

# Test cryptographic functions
storm-cli crypto test --test hash
```

## Output Formats

Use the `-o` flag to change output format:

- `text` (default) - Human-readable with colors
- `json` - Structured JSON for scripting
- `csv` - Comma-separated values for spreadsheets

Example:

```bash
storm-cli list game.mpq -o json | jq '.files[] | select(.size > 1000000)'
```

## Shell Completion

Generate completion scripts for your shell:

```bash
# Bash
storm-cli completion bash > ~/.bash_completion.d/storm-cli.bash

# Zsh
storm-cli completion zsh > ~/.zfunc/_storm-cli

# Fish
storm-cli completion fish > ~/.config/fish/completions/storm-cli.fish

# PowerShell
storm-cli completion powershell | Out-String | Invoke-Expression
```

For installation help:

```bash
# Linux/macOS
./scripts/install-completions.sh

# Windows PowerShell
.\scripts\Install-Completions.ps1
```

## Command Reference

### Global Options

- `-v, --verbose` - Increase verbosity (use multiple times for more detail)
- `-q, --quiet` - Suppress non-essential output
- `-o, --output <format>` - Output format: text, json, csv
- `--no-color` - Disable colored output
- `-c, --config <path>` - Path to configuration file

### Command Groups

#### archive - Archive-level operations

- `create` - Create a new MPQ archive
- `info` - Show detailed archive information
- `verify` - Verify archive integrity

#### file - File operations within archives

- `list` - List files in an archive
- `extract` - Extract files from an archive
- `find` - Search for files by pattern
- `info` - Show detailed file information
- `add` - Add files to existing archive (TODO)
- `remove` - Remove files from archive (TODO)

#### table - Low-level table operations

- `show` - Display table contents
- `analyze` - Analyze table structure

#### hash - Hash utilities

- `generate` - Generate hash values
- `compare` - Compare hash values
- `jenkins` - Generate Jenkins hash

#### crypto - Cryptography utilities

- `test` - Test cryptographic functions

For detailed command documentation, run:

```bash
storm-cli <command-group> --help
```

## Enhanced File Listing

The `file list` command now provides powerful options for exploring MPQ archives:

### Show All Files (--all)

```bash
# List ALL entries from the hash/block tables, not just those in (listfile)
storm-cli file list archive.mpq --all

# This shows files as file_XXXXXXXX.dat when names are unknown
```

### Show File Hashes (--show-hashes)

```bash
# Display MPQ name hashes for each file
storm-cli file list archive.mpq --show-hashes
# Output: filename.txt [HASH1 HASH2]

# Combine with --all to map unknown files by comparing hashes
storm-cli file list archive.mpq --all --show-hashes
```

### Verbose Mode (-v, -vv)

```bash
# Show detailed file information
storm-cli file list archive.mpq -v
# Displays: Name, Size, Compressed Size, Ratio, Flags

# Very verbose mode shows additional statistics
storm-cli file list archive.mpq -vv
# Also shows: compression statistics, file type counts

# Combine with --show-hashes for complete information
storm-cli file list archive.mpq -v --show-hashes
```

### Mapping Unknown Files

When you encounter unknown files with `--all`, you can map them using hashes:

```bash
# First, get hashes of known files
storm-cli file list archive.mpq --show-hashes > known_files.txt

# Then get all files with hashes
storm-cli file list archive.mpq --all --show-hashes > all_files.txt

# Files with matching hashes are the same file
# file_00000001.dat [395B7DE8 04CF5C07] = data.txt [395B7DE8 04CF5C07]
```

## Examples

### Working with Warcraft III Maps

```bash
# Extract all files from a map
storm-cli file extract mymap.w3x -o mymap_extracted/

# Create a new map archive
storm-cli archive create mymap.w3x map_files/ --compression zlib

# Find the map script
storm-cli file find mymap.w3x "war3map.j"

# Get file information
storm-cli file info mymap.w3x war3map.j

# Verify map integrity
storm-cli archive verify mymap.w3x --check-crc
```

### Batch Processing

```bash
# Extract all MPQ files in a directory
for file in *.mpq; do
    storm-cli file extract "$file" -o "extracted/${file%.mpq}/"
done

# List contents of multiple archives as JSON
for file in *.mpq; do
    echo "=== $file ==="
    storm-cli file list "$file" -o json
done > all_contents.json

# Find all DDS textures across multiple archives
for file in *.mpq; do
    echo "=== $file ==="
    storm-cli file find "$file" "*.dds"
done
```

## License

This project is dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.
