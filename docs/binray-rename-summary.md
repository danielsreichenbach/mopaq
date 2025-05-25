# Binary Rename Summary

This document summarizes the changes made to rename the CLI binary from `storm` to `storm-cli`.

## Why the Change?

The binary was renamed to avoid naming conflicts between:

- The library crate named `storm`
- The CLI binary (previously also named `storm`)

This conflict could cause issues with:

- Cargo's target directory organization
- Package management and distribution
- User confusion about which component they're using

## What Changed

### 1. Cargo.toml Configuration

- `storm-cli/Cargo.toml`: Binary name changed in `[[bin]]` section

### 2. Documentation Updates

- README.md: CLI usage examples updated
- docs/cli-usage.md: New comprehensive CLI guide
- docs/naming-conventions.md: Documents all naming decisions
- CHANGELOG.md: Change documented

### 3. Code Updates

- `storm-cli/src/main.rs`: Parser name updated to match binary
- Comments added explaining the naming decision

### 4. Build System

- Makefile: Updated install target description
- Scripts: Added test_cli_name.py to verify the change

### 5. Tests

- `storm-cli/tests/cli.rs`: Integration tests using correct binary name

## Usage After Change

```bash
# Install
cargo install --path storm-cli

# Use
storm-cli list archive.mpq
storm-cli extract archive.mpq --output ./extracted
storm-cli create new.mpq ./files
```

## Migration for Users

If you have scripts or documentation using the old `storm` command:

1. Replace `storm` with `storm-cli` in all commands
2. Update any PATH configurations if needed
3. Remove old `storm` binary if it exists

## Technical Impact

- No API changes
- No functionality changes
- Only the executable name is different
- Library users (Rust dependencies) are unaffected
- FFI users are unaffected
