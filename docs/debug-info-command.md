# Debug Info Command

The `debug info` command displays detailed information about an MPQ archive's structure and headers.

## Usage

```bash
storm-cli debug info <archive>
```

## Features

- Detects archive offset (for embedded MPQs)
- Parses all MPQ format versions (v1-v4)
- Displays user data headers (common in SC2 maps)
- Shows table positions and sizes
- Displays MD5 checksums for v4 archives

## Output Sections

### Basic Information

- File path
- Archive offset in the file (useful for embedded MPQs)

### User Data Header (if present)

- User data size
- Header offset
- User data header size

### MPQ Header

- Format version with human-readable name
- Header size
- Archive size (using 64-bit value for v2+)
- Block size and calculated sector size

### Tables

- Hash table position and entry count
- Block table position and entry count
- Hi-block table position (v2+)
- HET/BET table positions (v3+)

### Version 4 Extended Data

- Compressed table sizes
- MD5 checksums for all tables and header

## Implementation Details

The command uses the 512-byte aligned header scanning algorithm to locate MPQ headers, supporting:

- Standard MPQ archives starting at offset 0
- Embedded MPQs (e.g., in installers)
- MPQs with user data headers (SC2 maps)

## Example Output

```
MPQ Archive Information
======================

File: test.mpq
Archive offset: 0x00000000 (0 bytes)

MPQ Header:
  Format version: 1 (Burning Crusade)
  Header size: 44 bytes
  Archive size: 2048 bytes
  Block size: 4 (sector size: 8192 bytes)

Tables:
  Hash table:
    Position: 0x00000400
    Entries: 32 (must be power of 2)
  Block table:
    Position: 0x00000800
    Entries: 16
  Hi-block table:
    Position: 0x00001000
```

## Future Enhancements

Once more functionality is implemented, the info command will also show:

- Total file count
- Compression statistics
- Digital signature status
- Listfile presence
- Attributes file presence
