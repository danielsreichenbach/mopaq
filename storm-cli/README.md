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

### Basic Commands

```bash
# List files in an archive
storm-cli list game.mpq

# Extract all files
storm-cli extract game.mpq -t extracted/

# Extract specific file
storm-cli extract game.mpq -f "war3map.j" -t output/

# Create new archive
storm-cli create new.mpq source_folder/

# Find a file
storm-cli find game.mpq "Units\\Human\\Footman\\Footman.mdx"

# Verify archive integrity
storm-cli verify game.mpq
```

### Advanced Options

```bash
# List with all entries (including unnamed)
storm-cli list game.mpq --all

# Create with specific compression
storm-cli create archive.mpq files/ --compression zlib

# Output as JSON
storm-cli list game.mpq -o json

# Verbose output
storm-cli extract game.mpq -vv

# Exclude files when creating
storm-cli create archive.mpq src/ --ignore "*.tmp" --ignore "*.log"
```

### Debug Commands

```bash
# Show detailed archive information
storm-cli debug info game.mpq

# Display hash tables
storm-cli debug tables game.mpq

# Calculate hash for a filename
storm-cli debug hash "war3map.j"

# Compare hashes between filenames
storm-cli debug hash-compare "file1.txt" "file2.txt"
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

### list

List files in an MPQ archive.

```bash
storm-cli list <archive> [options]
```

Options:

- `--all` - Show all entries including unnamed files

### extract

Extract files from an MPQ archive.

```bash
storm-cli extract <archive> [options]
```

Options:

- `-f, --file <name>` - Extract specific file
- `-t, --target <dir>` - Target directory (default: current)

### create

Create a new MPQ archive.

```bash
storm-cli create <archive> <inputs...> [options]
```

Options:

- `-c, --compression <type>` - Compression: none, zlib, bzip2, lzma
- `-f, --format <version>` - MPQ format version (1-4)
- `-i, --ignore <pattern>` - Ignore files matching pattern

### find

Find a specific file in an archive.

```bash
storm-cli find <archive> <filename>
```

### verify

Verify archive integrity.

```bash
storm-cli verify <archive> [options]
```

Options:

- `--verbose` - Show detailed verification information

## Examples

### Working with Warcraft III Maps

```bash
# Extract all files from a map
storm-cli extract mymap.w3x -t mymap_extracted/

# Create a new map archive
storm-cli create mymap.w3x map_files/ --compression zlib

# Find the map script
storm-cli find mymap.w3x "war3map.j"

# Verify map integrity
storm-cli verify mymap.w3x --verbose
```

### Batch Processing

```bash
# Extract all MPQ files in a directory
for file in *.mpq; do
    storm-cli extract "$file" -t "extracted/${file%.mpq}/"
done

# List contents of multiple archives as JSON
for file in *.mpq; do
    echo "=== $file ==="
    storm-cli list "$file" -o json
done > all_contents.json
```

## License

This project is dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.
