# Storm CLI Command Structure

This document describes the improved command structure for storm-cli, organized into logical groups for better discoverability and extensibility.

## Command Groups

### Archive Operations (`storm-cli archive`)

Commands for archive-level operations:

- `create` - Create a new MPQ archive
- `info` - Show detailed archive information
- `verify` - Verify archive integrity

### File Operations (`storm-cli file`)

Commands for working with files within archives:

- `list` - List files in an archive
- `extract` - Extract files from an archive
- `add` - Add files to an existing archive (TODO)
- `remove` - Remove files from an archive (TODO)
- `find` - Search for files by pattern
- `info` - Show detailed file information

### Table Operations (`storm-cli table`)

Low-level operations for MPQ tables:

- `show` - Display table contents
- `analyze` - Analyze table structure and efficiency

### Hash Utilities (`storm-cli hash`)

Hash generation and comparison:

- `generate` - Generate hash values for a filename
- `compare` - Compare hash values for two filenames
- `jenkins` - Generate Jenkins hash (for HET tables)

### Cryptography Utilities (`storm-cli crypto`)

Cryptographic testing tools:

- `test` - Test cryptographic functions

### Other Commands

- `completion` - Generate shell completion scripts

## Examples

```bash
# Archive operations
storm-cli archive create game.mpq source/ --compression zlib
storm-cli archive info game.mpq
storm-cli archive verify game.mpq --check-crc

# File operations
storm-cli file list game.mpq --pattern "*.mdx"
storm-cli file extract game.mpq war3map.j -o extracted/
storm-cli file find game.mpq "*.blp" --regex

# Table operations
storm-cli table show game.mpq --table-type hash --limit 50
storm-cli table analyze game.mpq --detailed

# Hash utilities
storm-cli hash generate "war3map.j" --all
storm-cli hash compare file1.txt file2.txt

# Generate completions
storm-cli completion bash > ~/.bash_completion.d/storm-cli.bash
```

## Configuration

Storm-cli supports configuration files in the following locations:

- `~/.storm-cli/config.toml`
- `~/.config/storm-cli/config.toml`

You can also specify a custom config file with the `-c` flag:

```bash
storm-cli -C myconfig.toml file list game.mpq
```

### Configuration Options

```toml
# Default compression method (none, zlib, bzip2, lzma)
default_compression = "zlib"

# Default MPQ version (1-4)
default_version = 1

# Default block size (0-23)
default_block_size = 3

# Default output format (text, json, csv)
default_output = "text"

# Command aliases
[aliases]
"ls" = "file list"
"x" = "file extract"
```

## Global Options

These options are available for all commands:

- `-o, --output <FORMAT>` - Output format (text, json, csv)
- `-v, --verbose` - Increase verbosity (can be used multiple times)
- `-q, --quiet` - Suppress all output except errors
- `--no-color` - Disable colored output
- `-C, --config <PATH>` - Path to config file

## Benefits of the New Structure

1. **Logical Grouping**: Related commands are grouped together
2. **Consistent Naming**: All commands follow verb-noun pattern
3. **Progressive Disclosure**: Basic operations are easy to find, advanced features are discoverable
4. **Extensibility**: Easy to add new subcommands to existing groups
5. **Better Help**: `storm-cli archive --help` shows all archive operations
6. **Configuration Support**: Default settings and aliases for common operations
7. **Flexible Output**: Support for text, JSON, and CSV output formats
