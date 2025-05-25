# Storm CLI Usage Guide

The Storm command-line tool (`storm-cli`) provides a convenient interface for working with MPQ archives.

## Installation

```bash
# From the project root
cargo install --path storm-cli

# Or using Make
make install-cli
```

After installation, the `storm-cli` binary will be available in your Cargo bin directory.

## Basic Commands

### List Archive Contents

```bash
storm-cli list archive.mpq
```

### Extract Files

```bash
# Extract all files to current directory
storm-cli extract archive.mpq

# Extract to specific directory
storm-cli extract archive.mpq --output ./extracted
```

### Create New Archive

```bash
# Create archive from directory
storm-cli create new_archive.mpq ./source_files
```

### Verify Archive Integrity

```bash
storm-cli verify archive.mpq
```

## Debug Commands (Coming Soon)

### Show Archive Information

```bash
storm-cli debug info archive.mpq
```

### Display Table Contents

```bash
storm-cli debug tables archive.mpq
```

### Calculate File Hashes

```bash
storm-cli debug hash "path/to/file.txt"
```

## Examples

### Working with Warcraft III Maps

```bash
# List contents of a map
storm-cli list my_map.w3m

# Extract map scripts
storm-cli extract my_map.w3m --output ./map_data
```

### Creating a Mod Archive

```bash
# Create a new MPQ for your mod
storm-cli create my_mod.mpq ./mod_files
```

## Notes

- The binary is named `storm-cli` to avoid conflicts with the `storm` library crate
- Use `storm-cli --help` for detailed command information
- Each subcommand has its own help: `storm-cli extract --help`
