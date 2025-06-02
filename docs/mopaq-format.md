# MPQ Archive Format Documentation

## Table of Contents

- [Introduction](#introduction)
- [Version History](#version-history)
- [Core Structure](#core-structure)
- [Feature Matrix](#feature-matrix)
- [Locating MPQ Headers](#locating-mpq-headers)
  - [User Data Header Structure](#user-data-header-structure)
  - [Standard MPQ Header Structure](#standard-mpq-header-structure)
  - [Header Location Algorithm Pseudocode](#header-location-algorithm-pseudocode)
- [Tables and Data Storage](#tables-and-data-storage)
  - [Hash Table Structure and Contents](#hash-table-structure-and-contents)
    - [Hash Table Entry Structure](#hash-table-entry-structure)
    - [Hash Table Entry States](#hash-table-entry-states)
    - [Hash Table Organization](#hash-table-organization)
    - [Hash Calculation and File Lookup Process](#hash-calculation-and-file-lookup-process)
    - [Multiple Language Versions of Files](#multiple-language-versions-of-files)
    - [Hash Table Encryption](#hash-table-encryption)
    - [Hash Table Optimization](#hash-table-optimization)
    - [Listfile Integration](#listfile-integration)
  - [Block Table Structure](#block-table-structure)
  - [Hi-Block Table](#hi-block-table-version-2)
  - [HET Table Structure](#het-table-structure-version-3)
  - [BET Table Structure](#bet-table-structure-version-3)
  - [File Data Storage](#file-data-storage)
  - [File Hashing Algorithm](#file-hashing-algorithm)
  - [Encryption and Decryption Algorithm](#encryption-and-decryption-algorithm)
  - [Special Files](#special-files)
- [Digital Signatures](#digital-signatures)
  - [Weak Digital Signature](#weak-digital-signature)
  - [Strong Digital Signature](#strong-digital-signature)
- [Compression Method Compatibility Matrix](#compression-method-compatibility-matrix)
- [Block Table Flags](#block-table-flags)
- [Implementation Notes](#implementation-notes)
- [Notable Library Implementations](#notable-library-implementations)
- [References](#references)

## Introduction

MPQ (Mo'PaQ, short for Mike O'Brien Pack) is an archiving file format developed by Blizzard Entertainment for storing game assets including graphics, sounds, and level data. This format has evolved through several versions, each adding new capabilities while maintaining backward compatibility.

## Version History

| Version | First Appearance | Header Size | Notable Games |
|---------|-----------------|------------|---------------|
| 1       | Original/Vanilla | 32 bytes (0x20) | Diablo, Starcraft, Warcraft II, Diablo II, Warcraft III |
| 2       | The Burning Crusade | 44 bytes (0x2C) | World of Warcraft: The Burning Crusade |
| 3       | Cataclysm Beta | 68 bytes (0x44) | World of Warcraft: Cataclysm |
| 4       | Cataclysm+ | 208 bytes (0xD0) | Later World of Warcraft, Starcraft II |

## Core Structure

All MPQ archives contain some combination of these elements, depending on version:

1. Optional data before the MPQ archive
2. Optional MPQ User Data
3. MPQ Header (required)
4. Files stored in the archive (optional)
5. Special files: (listfile), (attributes), (signature), (user data) (optional)
6. Hash Table (required in v1-v2, optional in v3+)
7. Block Table (required in v1-v2, optional in v3+)
8. Hi-Block Table (added in v2)
9. HET Table (added in v3)
10. BET Table (added in v3)
11. Strong digital signature (optional)

## Feature Matrix

| Feature | v1 | v2 | v3 | v4 | Notes |
|---------|----|----|----|----|-------|
| **Header Structure** |
| Basic header fields | ✅ | ✅ | ✅ | ✅ | ID, size, archive size, etc. |
| Hi-Block Table support | ❌ | ✅ | ✅ | ✅ | For archives larger than 4GB |
| BET Table support | ❌ | ❌ | ✅ | ✅ | Replacement for block table |
| HET Table support | ❌ | ❌ | ✅ | ✅ | Replacement for hash table |
| MD5 Checksums | ❌ | ❌ | ❌ | ✅ | Added for tables and header integrity |
| Raw chunk size for MD5 | ❌ | ❌ | ❌ | ✅ | For block-level integrity |
| **Size Limitations** |
| Max files in hash table | 2¹⁶ | 2²⁰ | 2²⁰ | 2²⁰ | Limit on number of files |
| Archive size limit | 4GB | >4GB | >4GB | >4GB | v2+ supports larger archives |
| **Tables** |
| Hash table | ✅ | ✅ | ✅ | ✅ | Required in v1-v2, optional in v3+ |
| Block table | ✅ | ✅ | ✅ | ✅ | Required in v1-v2, optional in v3+ |
| Hi-Block table | ❌ | ✅ | ✅ | ✅ | Extended position data |
| HET table | ❌ | ❌ | ✅ | ✅ | More efficient file lookup |
| BET table | ❌ | ❌ | ✅ | ✅ | More efficient block data |
| Compressed hash table | ❌ | ❌ | ✅ | ✅ | v3+ can compress tables |
| Compressed block table | ❌ | ❌ | ✅ | ✅ | v3+ can compress tables |
| **Compression Methods** |
| PKWare implode | ✅ | ✅ | ✅ | ✅ | First compression algorithm |
| Huffman encoding | ✅ | ✅ | ✅ | ✅ | Added in Starcraft |
| ADPCM compression | ✅ | ✅ | ✅ | ✅ | For audio data, lossy |
| Deflate (zlib) | ✅ | ✅ | ✅ | ✅ | Added in Warcraft III |
| BZip2 | ❌ | ✅ | ✅ | ✅ | Added in WoW: The Burning Crusade |
| LZMA | ❌ | ❌ | ✅ | ✅ | Added in Starcraft II |
| Sparse compression | ❌ | ❌ | ✅ | ✅ | Added in Starcraft II |
| Multiple compression | ✅ | ✅ | ✅ | ✅ | Can chain compression methods |
| **Security Features** |
| File encryption | ✅ | ✅ | ✅ | ✅ | Base feature |
| Table encryption | ✅ | ✅ | ✅ | ✅ | Hash and block tables encrypted |
| Weak digital signature | ✅ | ✅ | ✅ | ✅ | RSASSA-PKCS1-v1_5 with MD5 and 512-bit RSA |
| Strong digital signature | ❌ | ✅ | ✅ | ✅ | Added for WoW and later games |
| **Language Support** |
| Multiple language versions | ✅ | ✅ | ✅ | ✅ | Using locale codes |
| Multiple platform versions | ✅ | ✅ | ✅ | ✅ | Platform codes (vestigial - always 0) |
| **Special Files** |
| (listfile) support | ✅ | ✅ | ✅ | ✅ | List of filenames in the archive |
| (attributes) support | ✅ | ✅ | ✅ | ✅ | File attributes data |
| (signature) support | ✅ | ✅ | ✅ | ✅ | Digital signature data |
| (user data) support | ❌ | ✅ | ✅ | ✅ | Custom metadata |

## Locating MPQ Headers

MPQ archives can be standalone files or embedded within another file (for example, as part of an executable installer). The format supports this flexibility through a specific header location mechanism:

1. MPQ headers (both standard and user data headers) must begin at file offsets aligned to 512 (0x200) bytes
2. When parsing an MPQ file, the application must scan the file at offsets 0, 0x200, 0x400, 0x600 and so on, checking for valid MPQ signatures
3. Two possible header types may be encountered during this scan:
   - Standard MPQ header (signature 'MPQ\x1A' or 0x1A51504D in little-endian)
   - MPQ user data header (signature 'MPQ\x1B' or 0x1B51504D in little-endian)
4. If a standard MPQ header is found, processing continues with this header
5. If a user data header is found, the offset specified in the user data header is added to the current offset to locate the actual MPQ header, and scanning continues from that position

### User Data Header Structure

The user data header allows custom metadata to be stored before the actual MPQ archive. It is commonly used in custom maps for Starcraft II and other later Blizzard games.

```rust
/// MPQ user data header structure
#[repr(C, packed)]
struct MpqUserData {
    /// The ID_MPQ_USERDATA ('MPQ\x1B') signature (0x1B51504D in little-endian)
    id: u32,

    /// Maximum size of the user data
    user_data_size: u32,

    /// Offset of the MPQ header, relative to the beginning of this header
    header_offset: u32,

    /// Size of user data header (commonly used in Starcraft II maps)
    user_data_header_size: u32,

    /// User data follows the header
    /// user_data: [u8; user_data_size - user_data_header_size]
}
```

### Standard MPQ Header Structure

Once located, the standard MPQ header contains essential information about the archive structure and version. The size of the header varies based on the MPQ format version.

```rust
/// MPQ file header structure
#[repr(C, packed)]
struct MpqHeader {
    /// The ID_MPQ ('MPQ\x1A') signature (0x1A51504D in little-endian)
    id: u32,

    /// Size of the archive header (32, 44, 68, or 208 bytes depending on version)
    header_size: u32,

    /// Size of MPQ archive (deprecated in v2+)
    archive_size: u32,

    /// Format version (0=v1, 1=v2, 2=v3, 3=v4)
    format_version: u16,

    /// Block size (power of two exponent for the sector size)
    block_size: u16,

    /// Offset to the hash table
    hash_table_pos: u32,

    /// Offset to the block table
    block_table_pos: u32,

    /// Number of entries in the hash table (power of 2)
    hash_table_size: u32,

    /// Number of entries in the block table
    block_table_size: u32,

    // Additional fields follow for v2, v3, and v4 headers
    // (see version-specific fields in the feature matrix)
}

/// MPQ header v2 extension
#[repr(C, packed)]
struct MpqHeaderExtV2 {
    /// Offset to the beginning of array of 16-bit high parts of file offsets
    hi_block_table_pos: u64,

    /// High 16 bits of the hash table offset for large archives
    hash_table_pos_hi: u16,

    /// High 16 bits of the block table offset for large archives
    block_table_pos_hi: u16,
}

/// MPQ header v3 extension
#[repr(C, packed)]
struct MpqHeaderExtV3 {
    /// 64-bit version of the archive size
    archive_size_64: u64,

    /// 64-bit position of the BET table
    bet_table_pos: u64,

    /// 64-bit position of the HET table
    het_table_pos: u64,
}

/// MPQ header v4 extension
#[repr(C, packed)]
struct MpqHeaderExtV4 {
    /// Compressed size of the hash table
    hash_table_size_64: u64,

    /// Compressed size of the block table
    block_table_size_64: u64,

    /// Compressed size of the hi-block table
    hi_block_table_size_64: u64,

    /// Compressed size of the HET block
    het_table_size_64: u64,

    /// Compressed size of the BET block
    bet_table_size_64: u64,

    /// Size of raw data chunk to calculate MD5
    raw_chunk_size: u32,

    /// MD5 checksums for various tables
    md5_block_table: [u8; 16],    // MD5 of the block table before decryption
    md5_hash_table: [u8; 16],     // MD5 of the hash table before decryption
    md5_hi_block_table: [u8; 16], // MD5 of the hi-block table
    md5_bet_table: [u8; 16],      // MD5 of the BET table before decryption
    md5_het_table: [u8; 16],      // MD5 of the HET table before decryption
    md5_mpq_header: [u8; 16],     // MD5 of the MPQ header from signature to (including) MD5_HetTable
}
```

### Header Location Algorithm Pseudocode

```rust
fn find_mpq_header(file: &mut File) -> Result<u64, MpqError> {
    let mut offset: u64 = 0;
    let file_size = file.metadata()?.len();

    while offset < file_size {
        file.seek(SeekFrom::Start(offset))?;
        let mut signature = [0u8; 4];
        file.read_exact(&mut signature)?;

        let signature_value = u32::from_le_bytes(signature);

        match signature_value {
            0x1A51504D => {
                // Found standard MPQ header (MPQ\x1A)
                return Ok(offset);
            },
            0x1B51504D => {
                // Found user data header (MPQ\x1B)
                file.seek(SeekFrom::Start(offset + 8))?;
                let mut header_offset_bytes = [0u8; 4];
                file.read_exact(&mut header_offset_bytes)?;

                let header_offset = u32::from_le_bytes(header_offset_bytes);
                let new_offset = offset + u64::from(header_offset);

                // Verify new offset is valid
                if new_offset < file_size {
                    file.seek(SeekFrom::Start(new_offset))?;
                    let mut new_signature = [0u8; 4];
                    file.read_exact(&mut new_signature)?;

                    if u32::from_le_bytes(new_signature) == 0x1A51504D {
                        return Ok(new_offset);
                    }
                }
            },
            _ => {},
        }

        // Move to next potential header position
        offset += 0x200;  // 512 bytes
    }

    Err(MpqError::InvalidFormat("No valid MPQ header found"))
}
```

## Tables and Data Storage

### Hash Table Structure and Contents

The hash table is the primary mechanism for locating files within an MPQ archive. Unlike many other archive formats, MPQ does not store full file paths in a central directory. Instead, it uses hash-based lookups, which makes the hash table a crucial component of the format.

**Size Constraints:**

- The hash table size must be a power of two (2ⁿ)
- **Minimum size:** 0x00000004 (4 entries)
- **Default size:** 0x00001000 (4,096 entries)
- **Maximum size:** 0x00080000 (524,288 entries)

#### Hash Table Entry Structure

```rust
/// Hash table entry structure (16 bytes)
#[repr(C, packed)]
struct MpqHashEntry {
    /// The hash of the full file name (part A)
    name_1: u32,  // Hash using method A

    /// The hash of the full file name (part B)
    name_2: u32,  // Hash using method B

    /// The language of the file (Windows LANGID)
    locale: u16,  // 0 = default/neutral

    /// The platform the file is used for (vestigial field)
    /// NOTE: This field exists in the format but is always 0 in practice.
    /// Blizzard uses separate archives (e.g., base-Win.MPQ, base-OSX.MPQ)
    /// instead of platform codes within archives.
    platform: u16,  // Always 0 in all known archives

    /// Block table index or special value:
    /// - 0xFFFFFFFF: Empty entry, has always been empty
    /// - 0xFFFFFFFE: Empty entry, was previously valid (deleted file)
    block_index: u32,  // Index into block table or special value
}

impl MpqHashEntry {
    const EMPTY_NEVER_USED: u32 = 0xFFFFFFFF;
    const EMPTY_DELETED: u32 = 0xFFFFFFFE;

    /// Returns true if this entry has never been used
    pub fn is_empty(&self) -> bool {
        self.block_index == Self::EMPTY_NEVER_USED
    }

    /// Returns true if this entry was deleted
    pub fn is_deleted(&self) -> bool {
        self.block_index == Self::EMPTY_DELETED
    }

    /// Returns true if this entry contains valid file information
    pub fn is_valid(&self) -> bool {
        self.block_index < Self::EMPTY_DELETED
    }
}
```

#### Hash Table Entry States

Each hash table entry can be in one of three states:

1. **Empty (Never Used)**: Indicated by `dwBlockIndex = 0xFFFFFFFF`
   - This entry has never contained file information
   - Hash searches terminate when encountering this type of entry

2. **Empty (Previously Used)**: Indicated by `dwBlockIndex = 0xFFFFFFFE`
   - This entry previously contained file information, but the file was deleted
   - Hash searches continue past this type of entry (allowing for collision resolution)

3. **Occupied**: Indicated by `dwBlockIndex < 0xFFFFFFFE`
   - The entry contains valid file information
   - The `dwBlockIndex` value points to the corresponding entry in the block table

#### Hash Table Organization

The hash table always has 2ⁿ entries, and it's organized to enable efficient file lookups:

1. The initial position for a file in the table is calculated using a hash of the lowercased filename modulo the table size.
2. If a collision occurs (multiple files hash to the same position), a linear probing approach is used:
   - Proceed to the next entry in the table
   - Continue until finding the correct entry or an empty/never-used entry
   - The search wraps around to the beginning of the table if necessary

#### Hash Calculation and File Lookup Process

To find a file in the MPQ archive:

1. Calculate three hash values for the filename:

   ```c
   DWORD hashA = HashString(filename, MPQ_HASH_NAME_A);
   DWORD hashB = HashString(filename, MPQ_HASH_NAME_B);
   DWORD index = HashString(filename, MPQ_HASH_TABLE_OFFSET) % hashTableSize;
   ```

2. Begin searching at the calculated index position:

   ```c
   pHashEntry = &hashTable[index];
   ```

3. Examine the hash entry at the current position:
   - If `dwName1 == hashA` and `dwName2 == hashB` and the locale/platform matches:
     - This is the desired file; use `dwBlockIndex` to find it in the block table
   - If `dwBlockIndex == 0xFFFFFFFF` (never used):
     - The file doesn't exist in the archive; terminate the search
   - Otherwise:
     - Move to the next entry (wrapping if necessary) and continue the search

This collision resolution strategy means files are not strictly stored at their hash position but may be placed at the next available slot.

#### Multiple Language Versions of Files

The MPQ format supports multiple language versions of the same file. These versions have the same filename and hash values but different `lcLocale` values. When searching for files, if a specific locale is requested, only files with matching locale should be considered.

Common locale values include:

- 0x0000: Neutral/Default (American English)
- 0x0409: English (US)
- 0x0809: English (UK)
- 0x0407: German
- 0x040c: French
- 0x0410: Italian
- 0x0405: Czech
- 0x0412: Korean
- 0x0411: Japanese

#### Hash Table Encryption

The hash table is encrypted using the MPQ encryption algorithm with the key derived from the string "(hash table)" (without quotes). Before using the hash table, an implementation must decrypt it:

```rust
/// Decrypt the hash table
fn decrypt_hash_table(hash_table: &mut [MpqHashEntry], encryption_table: &[u32; 0x500]) {
    // Calculate the key from the string "(hash table)"
    let key = hash_string("(hash table)", MPQ_HASH_FILE_KEY, encryption_table);

    // Cast to u32 array for processing
    let ptr = hash_table.as_mut_ptr() as *mut u32;
    let len = hash_table.len() * std::mem::size_of::<MpqHashEntry>() / std::mem::size_of::<u32>();

    // Create a safe slice from the raw pointer
    let data = unsafe { std::slice::from_raw_parts_mut(ptr, len) };

    // Decrypt each DWORD using the key and index
    for (i, val) in data.iter_mut().enumerate() {
        *val = decrypt_dword(*val, key.wrapping_add(i as u32), encryption_table);
    }
}
```

#### Hash Table Optimization

Some observations for implementers:

1. The hash table is typically sparse, containing many empty entries, especially in large archives
2. The hash table size is usually much larger than the number of files to minimize collisions
3. Hash searches that fail (file not found) can be expensive due to linear probing
4. To optimize lookups, many implementations create an in-memory file index based on the listfile

#### Listfile Integration

While the hash table doesn't store filenames, MPQ archives often include a special file named `(listfile)` that contains all filenames in the archive. When implementing an MPQ reader:

1. First load and process the hash table
2. Then extract and parse the `(listfile)` if present
3. Use the listfile contents to build a mapping between filenames and hash table entries

This allows for more user-friendly file access by name rather than requiring hash calculations for every access.

### Block Table Structure

The block table contains information about the actual file data storage in the archive.

```rust
/// Block table entry structure (16 bytes)
#[repr(C, packed)]
struct MpqBlockEntry {
    /// Offset of the beginning of the file data, relative to the beginning of the archive
    file_pos: u32,

    /// Compressed file size
    c_size: u32,

    /// Size of uncompressed file
    f_size: u32,

    /// Flags for the file (see Block Table Flags section)
    flags: u32,
}

impl MpqBlockEntry {
    // Block table flag constants
    pub const FLAG_IMPLODE: u32         = 0x00000100;
    pub const FLAG_COMPRESS: u32        = 0x00000200;
    pub const FLAG_ENCRYPTED: u32       = 0x00010000;
    pub const FLAG_FIX_KEY: u32         = 0x00020000;
    pub const FLAG_PATCH_FILE: u32      = 0x00100000;
    pub const FLAG_SINGLE_UNIT: u32     = 0x01000000;
    pub const FLAG_DELETE_MARKER: u32   = 0x02000000;
    pub const FLAG_SECTOR_CRC: u32      = 0x04000000;
    pub const FLAG_EXISTS: u32          = 0x80000000;

    /// Returns true if the file is compressed
    pub fn is_compressed(&self) -> bool {
        (self.flags & (Self::FLAG_IMPLODE | Self::FLAG_COMPRESS)) != 0
    }

    /// Returns true if the file is encrypted
    pub fn is_encrypted(&self) -> bool {
        (self.flags & Self::FLAG_ENCRYPTED) != 0
    }

    /// Returns true if the file is stored as a single unit
    pub fn is_single_unit(&self) -> bool {
        (self.flags & Self::FLAG_SINGLE_UNIT) != 0
    }

    /// Returns true if the file has sector CRCs
    pub fn has_sector_crc(&self) -> bool {
        (self.flags & Self::FLAG_SECTOR_CRC) != 0
    }
}
```

The block table is encrypted using a known algorithm with the string "(block table)" as the key.

### Hi-Block Table (Version 2+)

For archives larger than 4GB, the Hi-Block table stores the high 16 bits of file positions:

```rust
/// Hi-Block table consists of 16-bit values
/// One entry for each block table entry
type MpqHiBlockTable = Vec<u16>;

/// Full 64-bit file position calculation
fn get_full_file_pos(block_entry: &MpqBlockEntry, hi_block_table: &[u16], index: usize) -> u64 {
    if index < hi_block_table.len() {
        // Combine the high 16 bits with the low 32 bits to form a 48-bit value
        let high_bits = u64::from(hi_block_table[index]);
        let low_bits = u64::from(block_entry.file_pos);
        (high_bits << 32) | low_bits
    } else {
        // If there's no hi-block entry, just use the 32-bit value
        u64::from(block_entry.file_pos)
    }
}
```

This table is not encrypted, and when present it immediately follows the block table.

### HET Table Structure (Version 3+)

The HET (Hash Entry Table) is an alternative to the hash table introduced in format version 3, designed to be more efficient. The HET table is present if the `het_table_pos` member of the MPQ header is set to a non-zero value.

```rust
/// HET table header structure
#[repr(C, packed)]
struct MpqHetTable {
    /// Common header signature 'HET\x1A' (0x1A544548 in little-endian)
    signature: u32,

    /// Version (always 1)
    version: u32,

    /// Size of the contained table data
    data_size: u32,

    /// Size of the entire hash table including header
    table_size: u32,

    /// Maximum number of files in the MPQ
    max_file_count: u32,

    /// Size of the hash table in bytes
    hash_table_size: u32,

    /// Effective size of the hash entry in bits
    hash_entry_size: u32,

    /// Total size of file index in bits
    total_index_size: u32,

    /// Extra bits in the file index
    index_size_extra: u32,

    /// Effective size of the file index in bits
    index_size: u32,

    /// Size of the block index subtable in bytes
    block_table_size: u32,

    /// Followed by:
    /// - HET hash table (hash_table_size bytes)
    /// - Array of file indexes (bit-based)
}

impl MpqHetTable {
    const SIGNATURE: u32 = 0x1A544548; // "HET\x1A" in little-endian

    /// Calculates the hash mask for entry bits
    pub fn get_hash_mask(&self) -> u64 {
        (1u64 << self.hash_entry_size) - 1
    }

    /// Calculates the index mask for file indexes
    pub fn get_index_mask(&self) -> u64 {
        (1u64 << self.index_size) - 1
    }
}
```

The HET table can be encrypted and compressed.

### BET Table Structure (Version 3+)

The BET (Block Entry Table) is an alternative to the block table introduced in format version 3. The BET table is present if the `bet_table_pos` member of the MPQ header is set to a non-zero value.

```rust
/// BET table header structure
#[repr(C, packed)]
struct MpqBetTable {
    /// Common header signature 'BET\x1A' (0x1A544542 in little-endian)
    signature: u32,

    /// Version (always 1)
    version: u32,

    /// Size of the contained table data
    data_size: u32,

    /// Size of the entire table including header
    table_size: u32,

    /// Number of files in BET table
    file_count: u32,

    /// Unknown, typically set to 0x10
    unknown_08: u32,

    /// Size of one table entry in bits
    table_entry_size: u32,

    /// Bit positions and sizes for various fields
    bit_index_file_pos: u32,
    bit_index_file_size: u32,
    bit_index_cmp_size: u32,
    bit_index_flag_index: u32,
    bit_index_unknown: u32,

    bit_count_file_pos: u32,
    bit_count_file_size: u32,
    bit_count_cmp_size: u32,
    bit_count_flag_index: u32,
    bit_count_unknown: u32,

    /// BET hash information
    total_bet_hash_size: u32,
    bet_hash_size_extra: u32,
    bet_hash_size: u32,
    bet_hash_array_size: u32,

    /// Number of flags in the following array
    flag_count: u32,

    /// Followed by:
    /// - Array of file flags (flag_count * 4 bytes)
    /// - File table (bit-based)
    /// - Array of BET hashes
}

impl MpqBetTable {
    const SIGNATURE: u32 = 0x1A544542; // "BET\x1A" in little-endian

    /// Extracts a bit field from raw table data
    pub fn extract_bits(value: u64, bit_offset: u32, bit_count: u32) -> u64 {
        // Create a mask of bit_count bits
        let mask = (1u64 << bit_count) - 1;

        // Extract and return the bit field
        (value >> bit_offset) & mask
    }

    /// Gets a file's position from BET entry bits
    pub fn get_file_position(&self, entry_bits: u64) -> u64 {
        Self::extract_bits(entry_bits, self.bit_index_file_pos, self.bit_count_file_pos)
    }

    /// Gets a file's uncompressed size from BET entry bits
    pub fn get_file_size(&self, entry_bits: u64) -> u64 {
        Self::extract_bits(entry_bits, self.bit_index_file_size, self.bit_count_file_size)
    }

    /// Gets a file's compressed size from BET entry bits
    pub fn get_compressed_size(&self, entry_bits: u64) -> u64 {
        Self::extract_bits(entry_bits, self.bit_index_cmp_size, self.bit_count_cmp_size)
    }

    /// Gets a file's flag index from BET entry bits
    pub fn get_flag_index(&self, entry_bits: u64) -> u32 {
        Self::extract_bits(entry_bits, self.bit_index_flag_index, self.bit_count_flag_index) as u32
    }
}
```

The BET table can be encrypted and compressed.

### File Data Storage

Files in MPQ archives are typically split into blocks (sectors). The sector size is determined by the `wBlockSize` field in the MPQ header:

```
Sector Size = 512 * 2^wBlockSize
```

For example, if `wBlockSize` is 3, then each sector is 512 * 2³ = 4096 bytes.

If a file is flagged with `MPQ_FILE_SINGLE_UNIT`, it is stored as a single block regardless of its size. Otherwise:

1. For compressed files: a table of sector offsets is stored at the beginning of the file data
2. Each sector can be individually compressed using various compression methods
3. Each sector can be individually encrypted if the file is encrypted
4. If the `MPQ_FILE_SECTOR_CRC` flag is set, each sector has a checksum

The sector offset table format:

```
DWORD sectorOffsets[numSectors + 1];
```

The additional entry at the end is used to calculate the size of the last sector. Sector offsets are relative to the beginning of the file data in the MPQ.

### File Hashing Algorithm

The MPQ format uses multiple hash functions to locate files in the hash table:

```c
// Hash types
#define MPQ_HASH_TABLE_OFFSET   0   // For finding the correct hash table entry
#define MPQ_HASH_NAME_A         1   // For comparing the first hash value
#define MPQ_HASH_NAME_B         2   // For comparing the second hash value
#define MPQ_HASH_FILE_KEY       3   // For computing the encryption key
```

#### ASCII Conversion Tables

Before hashing filenames, the MPQ format normalizes character case using custom ASCII conversion tables rather than standard library functions. This ensures consistent behavior across different platforms and implementations.

The tables are static 256-byte arrays that map each ASCII value to its uppercase or lowercase equivalent:

```rust
/// ASCII table for uppercase conversion
static ASCII_TO_UPPER_TABLE: [u8; 256] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
    0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
    0x60, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, // a-o -> A-O
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F, // p-z -> P-Z
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F,
    0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
    0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
    0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF,
    0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
    0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF
];

/// ASCII table for lowercase conversion
static ASCII_TO_LOWER_TABLE: [u8; 256] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
    0x40, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, // A-O -> a-o
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, // P-Z -> p-z
    0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F,
    0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
    0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
    0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF,
    0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
    0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF,
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF
];
```

These tables are used primarily for:

1. **Case Normalization**: MPQ filenames are case-insensitive (e.g., "File.txt" and "FILE.TXT" should hash to the same value).
2. **Path Separator Conversion**: Forward slashes ('/') are converted to backslashes ('\\') during hashing to ensure consistent behavior.
3. **Consistency Across Platforms**: Using predefined tables ensures the same behavior regardless of the platform's locale or character set.

#### Hash Function Implementation

Here's the more detailed hash function that includes the ASCII conversion tables:

```rust
/// Hash function constants
const MPQ_HASH_TABLE_OFFSET: u32 = 0; // For finding the correct hash table entry
const MPQ_HASH_NAME_A: u32 = 1;       // For comparing the first hash value
const MPQ_HASH_NAME_B: u32 = 2;       // For comparing the second hash value
const MPQ_HASH_FILE_KEY: u32 = 3;     // For computing the encryption key
const MPQ_HASH_KEY2_MIX: u32 = 4;     // For comptuing the block encryption key

/// Hash function for MPQ filenames
fn hash_string(file_name: &str, hash_type: u32, crypt_table: &[u32; 0x500]) -> u32 {
    let mut seed1: u32 = 0x7FED7FED;
    let mut seed2: u32 = 0xEEEEEEEE;

    for &byte in file_name.as_bytes() {
        // Get the next character and normalize it
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to uppercase using the table
        ch = ASCII_TO_UPPER_TABLE[ch as usize];

        // Update the hash
        let table_idx = (hash_type * 0x100 + ch as u32) as usize;
        seed1 = crypt_table[table_idx] ^ (seed1.wrapping_add(seed2));
        seed2 = ch as u32 + seed1 + seed2 + (seed2 << 5) + 3;
    }

    seed1
}
```

For the HET hash table introduced in version 3, a modified Jenkins hash algorithm is used, but it still applies the same case normalization principles:

```rust
/// Jenkins hash function for HET tables
fn hash_string_jenkins(file_name: &str) -> u64 {
    let mut hash: u64 = 0;

    for &byte in file_name.as_bytes() {
        // Get the next character and normalize it
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to lowercase using the table
        ch = ASCII_TO_LOWER_TABLE[ch as usize];

        // Jenkins one-at-a-time hash algorithm
        hash = hash.wrapping_add(ch as u64);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }

    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);

    hash
}
```

The hash table search algorithm:

```rust
// Calculate the hash values
let index = hash_string(file_name, MPQ_HASH_TABLE_OFFSET, crypt_table) & (hash_table_size - 1);
let name_a = hash_string(file_name, MPQ_HASH_NAME_A, crypt_table);
let name_b = hash_string(file_name, MPQ_HASH_NAME_B, crypt_table);

// Start searching at the calculated index
let mut hash_entry = &hash_table[index as usize];
```

Files are then located by comparing the calculated hash values with values stored in the hash table.

### Encryption and Decryption Algorithm

The MPQ format uses a proprietary encryption algorithm for protecting file data, hash tables, and block tables. The algorithm relies on a pre-generated table of encryption values.

#### Encryption Table Generation

Before any encryption or decryption can occur, a static encryption table must be generated. This table consists of 1280 values and only needs to be computed once. Here's the pseudocode for generating the encryption table:

```rust
/// Generate the encryption/decryption table
fn generate_encryption_table() -> [u32; 0x500] {
    let mut encryption_table = [0u32; 0x500];
    let mut seed: u32 = 0x00100001;

    // Fill the table with seemingly random numbers based on a simple algorithm
    for index1 in 0..0x100 {
        for i in 0..5 {
            let index2 = index1 + i * 0x100;

            seed = (seed.wrapping_mul(125) + 3) % 0x2AAAAB;
            let temp1 = (seed & 0xFFFF) << 0x10;

            seed = (seed.wrapping_mul(125) + 3) % 0x2AAAAB;
            let temp2 = seed & 0xFFFF;

            encryption_table[index2] = temp1 | temp2;
        }
    }

    encryption_table
}
```

This generates a table of 0x500 (1280) `u32` values that will be used for all encryption and decryption operations.

#### Encryption Function

Here's the pseudocode for encrypting a block of data:

```rust
/// Encrypt a block of data
fn encrypt_data(data: &mut [u32], key: u32, encryption_table: &[u32; 0x500]) {
    // If the key is 0, don't bother encrypting
    if key == 0 {
        return;
    }

    let mut seed: u32 = 0xEEEEEEEE;

    // Process the data 4 bytes at a time
    for value in data.iter_mut() {
        // Update the seed using the encryption table and key
        seed = seed.wrapping_add(encryption_table[0x400 + (key & 0xFF) as usize]);

        // Update the current DWORD with the encryption formula
        let ch = *value;
        *value = ch ^ key.wrapping_add(seed);

        // Update the key for the next round
        key = (!key << 0x15).wrapping_add(0x11111111) | (key >> 0x0B);

        // Update the seed for the next round
        seed = ch.wrapping_add(seed).wrapping_add(seed << 5).wrapping_add(3);
    }
}
```

#### Decryption Function

The decryption algorithm is very similar to the encryption algorithm, with a small change in the order of operations:

```rust
/// Decrypt a block of data
fn decrypt_data(data: &mut [u32], key: u32, encryption_table: &[u32; 0x500]) {
    // If the key is 0, don't bother decrypting
    if key == 0 {
        return;
    }

    let mut seed: u32 = 0xEEEEEEEE;

    // Process the data 4 bytes at a time
    for value in data.iter_mut() {
        // Update the seed using the encryption table and key
        seed = seed.wrapping_add(encryption_table[0x400 + (key & 0xFF) as usize]);

        // Decrypt the current DWORD
        let ch = *value ^ key.wrapping_add(seed);
        *value = ch;

        // Update the key for the next round
        key = (!key << 0x15).wrapping_add(0x11111111) | (key >> 0x0B);

        // Update the seed for the next round
        seed = ch.wrapping_add(seed).wrapping_add(seed << 5).wrapping_add(3);
    }
}

/// Decrypt a single DWORD value
fn decrypt_dword(value: u32, key: u32, encryption_table: &[u32; 0x500]) -> u32 {
    let mut seed: u32 = 0xEEEEEEEE;
    seed = seed.wrapping_add(encryption_table[0x400 + (key & 0xFF) as usize]);

    let decrypted = value ^ key.wrapping_add(seed);
    decrypted
}
```

#### Computing Encryption Keys

Different parts of the MPQ file are encrypted with different keys:

```rust
/// Compute a file's encryption key
fn compute_file_key(file_name: &str, file_pos: u32, file_size: u32, flags: u32,
                    encryption_table: &[u32; 0x500]) -> u32 {
    // Get the base key from the filename
    let mut key = hash_string(file_name, MPQ_HASH_FILE_KEY, encryption_table);

    // Apply FIX_KEY modification if flag is set
    if (flags & MpqBlockEntry::FLAG_FIX_KEY) != 0 {
        key = (key.wrapping_add(file_pos)) ^ file_size;
    }

    key
}
```

This key is then used to encrypt/decrypt file data, sector offset tables, and other file structures.

#### Encryption Order

When operating on MPQ archives:

1. Tables (hash, block) are always encrypted as a whole
2. For files, the sector offset table is encrypted first (if present)
3. Each file sector is encrypted individually
4. Encryption is always applied after compression

This algorithm ensures that MPQ data remains protected while allowing efficient access with the proper keys.

### Special Files

MPQ archives may contain several special files with predefined names:

1. `(listfile)` - Contains a list of all filenames in the archive, separated by newlines or semicolons
2. `(attributes)` - Contains additional attributes for files in the archive
3. `(signature)` - Contains digital signature information for archive verification
4. `(user data)` - Contains additional user-defined metadata

## Digital Signatures

### Weak Digital Signature

The weak digital signature uses RSASSA-PKCS1-v1_5 with the MD5 hashing algorithm and a 512-bit RSA key. The signature is stored uncompressed and unencrypted in the file `(signature)` with the following structure:

```rust
/// Weak digital signature structure
#[repr(C, packed)]
struct MpqWeakSignature {
    /// Must be 0
    unknown1: u32,

    /// Must be 0
    unknown2: u32,

    /// The digital signature (512 bits = 64 bytes)
    signature: [u8; 64],
}

/// Verify a weak signature
fn verify_weak_signature(public_key: &RsaPublicKey, signature: &[u8], digest: &[u8; 16]) -> bool {
    // The signature is stored in little-endian order, but RSA expects big-endian
    let mut reversed_signature = [0u8; 64];
    let signature_data = &signature[8..72]; // Skip the two u32 unknown fields

    // Reverse the signature bytes
    for i in 0..64 {
        reversed_signature[i] = signature_data[63 - i];
    }

    // Verify using RSA-MD5 (PKCS#1 v1.5 padding)
    rsa_verify(public_key, Md5Algorithm, digest, &reversed_signature)
}
```

The archive is hashed from its beginning to its end, with the signature file content treated as binary zeros during signing and verification.

### Strong Digital Signature

The Strong Digital Signature is an enhanced security feature introduced in MPQ format version 2 and later, designed to provide stronger archive integrity verification than the weak signature. Unlike the weak signature (which uses 512-bit RSA with MD5), the strong signature uses 2048-bit RSA with SHA-1.

#### Key Characteristics

- **Algorithm**: RSA signature with SHA-1 hash
- **Key Size**: 2048-bit RSA (compared to 512-bit for weak signature)
- **Hash Algorithm**: SHA-1 (20 bytes)
- **Location**: Appended after the end of the MPQ archive data (not stored as a file within the archive)
- **Total Size**: 260 bytes (4-byte header + 256-byte signature)

#### Structure

```rust
/// Strong digital signature structure
#[repr(C, packed)]
struct MpqStrongSignature {
    /// Magic identifier "NGIS" ("SIGN" backwards) - 0x5349474E in little-endian
    magic: [u8; 4],  // Must be ['N', 'G', 'I', 'S']

    /// The digital signature data (2048 bits = 256 bytes)
    /// Stored in little-endian format
    signature: [u8; 256],
}

/// When decrypted with the public key, the signature has this internal structure
#[repr(C, packed)]
struct DecryptedSignatureContent {
    /// Padding byte - Must be 0x0B
    padding_type: u8,

    /// Padding bytes - Must all be 0xBB
    padding_bytes: [u8; 235],

    /// SHA-1 hash of the archive (20 bytes)
    /// In standard SHA-1 byte order
    sha1_hash: [u8; 20],
}
```

#### Implementation Details

##### 1. Signature Location

The strong digital signature is stored immediately after the archive, in the containing file. This is different from the weak signature, which is stored as a file named `(signature)` within the archive.

```
[MPQ Archive Data][Strong Signature (260 bytes)]
                  ^
                  ArchiveOffset + ArchiveSize
```

##### 2. Hashing Process

The entire archive (ArchiveSize bytes, starting at ArchiveOffset in the containing file) is hashed as a single block. The process is:

1. Calculate SHA-1 hash of the entire archive from beginning to end
2. Optional: Append a signature tail to the SHA-1 digest before finalization
3. Apply custom padding to create the message to be signed

##### 3. Signature Format

The signature uses a proprietary implementation of RSA signing with specific padding:

```rust
impl MpqStrongSignature {
    const MAGIC: &'static [u8; 4] = b"NGIS";
    const SIGNATURE_SIZE: usize = 256;  // 2048 bits
    const TOTAL_SIZE: usize = 260;      // 4 + 256

    /// Verify the magic identifier
    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
    }

    /// Extract the signature for verification
    pub fn get_signature_bytes(&self) -> &[u8; 256] {
        &self.signature
    }
}

impl DecryptedSignatureContent {
    const PADDING_TYPE: u8 = 0x0B;
    const PADDING_FILL: u8 = 0xBB;
    const PADDING_LENGTH: usize = 235;

    /// Verify the padding structure
    pub fn verify_padding(&self) -> bool {
        if self.padding_type != Self::PADDING_TYPE {
            return false;
        }

        self.padding_bytes.iter().all(|&b| b == Self::PADDING_FILL)
    }
}
```

##### 4. Verification Process

```rust
/// Verify a strong signature on an MPQ archive
pub fn verify_strong_signature(
    archive_data: &[u8],
    signature: &MpqStrongSignature,
    public_key: &RsaPublicKey,
) -> Result<bool, MpqError> {
    // 1. Verify signature magic
    if !signature.is_valid() {
        return Err(MpqError::InvalidSignature("Invalid signature magic"));
    }

    // 2. Calculate SHA-1 hash of the archive
    let mut hasher = Sha1::new();
    hasher.update(archive_data);
    let archive_hash = hasher.finalize();

    // 3. Decrypt the signature using RSA public key
    // Note: The signature is stored in little-endian format
    let mut signature_bytes = [0u8; 256];
    for i in 0..256 {
        signature_bytes[i] = signature.signature[255 - i];  // Reverse for big-endian
    }

    // 4. Perform RSA decryption
    let decrypted = rsa_decrypt(&signature_bytes, public_key)?;

    // 5. Verify padding structure
    if decrypted[0] != 0x0B {
        return Ok(false);
    }

    for i in 1..236 {
        if decrypted[i] != 0xBB {
            return Ok(false);
        }
    }

    // 6. Compare hash values
    let signature_hash = &decrypted[236..256];
    Ok(archive_hash.as_slice() == signature_hash)
}
```

#### Public Keys

All known Blizzard keys are 2048-bit (strong) RSA keys. A default key is stored in Storm.dll, but different games and archive types use different public keys:

- **Default Storm Key**: A default 2048-bit RSA key embedded in Storm.dll
- **Game-Specific Keys**: Each Blizzard game may have its own public key
- **Map-Specific Keys**: Warcraft 3 maps (.w3m and .w3x) use a specific key for map signatures

##### Warcraft 3 Map Signatures

Warcraft 3 maps have a special structure:

1. Map header (512 bytes)
2. MPQ archive at offset 512
3. Strong digital signature immediately after the archive

The SHA-1 digest for Warcraft 3 maps is calculated from the entire file content, including the map header, up to the end of the archive.

#### Differences from Weak Signature

| Feature | Weak Signature | Strong Signature |
|---------|----------------|------------------|
| **Algorithm** | RSASSA-PKCS1-v1_5 | Proprietary RSA |
| **Hash** | MD5 (16 bytes) | SHA-1 (20 bytes) |
| **Key Size** | 512-bit | 2048-bit |
| **Location** | Inside archive as `(signature)` | After archive data |
| **Padding** | Standard PKCS#1 | Custom (0x0B + 0xBB fill) |
| **Security** | Broken (key factored in 2014) | Still considered secure |
| **Storage** | Compressed/encrypted in archive | Raw data after archive |

#### Usage Notes

##### Detection

Check for "NGIS" magic at the end of the file:

```rust
/// Check if an MPQ has a strong signature
pub fn has_strong_signature(file: &mut File, archive_size: u64) -> Result<bool, std::io::Error> {
    let file_size = file.metadata()?.len();

    // Strong signature would be right after the archive
    if file_size < archive_size + 260 {
        return Ok(false);
    }

    // Seek to where the signature should be
    file.seek(SeekFrom::Start(archive_size))?;

    // Read the magic bytes
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;

    Ok(&magic == b"NGIS")
}
```

##### Important Considerations

1. **Archive Size**: The `ArchiveSize` field in the MPQ header does not include the strong signature
2. **Compatibility**: Not all MPQ readers support strong signature verification
3. **Optional Feature**: Archives can have weak, strong, both, or neither signature
4. **Byte Order**: The signature is stored in little-endian format but RSA operations expect big-endian
5. **No Compression**: Unlike the weak signature, the strong signature is never compressed or encrypted

#### Complete Example

```rust
use sha1::{Sha1, Digest};
use rsa::{RsaPublicKey, PublicKey};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub struct StrongSignatureVerifier {
    public_key: RsaPublicKey,
}

impl StrongSignatureVerifier {
    pub fn new(public_key: RsaPublicKey) -> Self {
        Self { public_key }
    }

    pub fn verify_file(&self, path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len();

        // Read MPQ header to get archive size
        let archive_size = self.read_archive_size(&mut file)?;

        // Check if strong signature exists
        if file_size < archive_size + 260 {
            return Ok(false);
        }

        // Read archive data
        file.seek(SeekFrom::Start(0))?;
        let mut archive_data = vec![0u8; archive_size as usize];
        file.read_exact(&mut archive_data)?;

        // Read signature
        file.seek(SeekFrom::Start(archive_size))?;
        let mut sig_data = [0u8; 260];
        file.read_exact(&mut sig_data)?;

        // Parse signature
        let signature = MpqStrongSignature {
            magic: [sig_data[0], sig_data[1], sig_data[2], sig_data[3]],
            signature: {
                let mut sig = [0u8; 256];
                sig.copy_from_slice(&sig_data[4..260]);
                sig
            },
        };

        // Verify
        verify_strong_signature(&archive_data, &signature, &self.public_key)
    }

    fn read_archive_size(&self, file: &mut File) -> Result<u64, Box<dyn std::error::Error>> {
        // Implementation would read MPQ header and extract archive size
        // This is simplified for the example
        unimplemented!()
    }
}
```

The implementation details of the strong signature are not fully documented publicly.

## Compression Method Compatibility Matrix

| Compression Type | ID Bit | v1 | v2 | v3 | v4 | Notes |
|-----------------|--------|----|----|----|----|-------|
| PKWare implode | 0x00000100 | ✅ | ✅ | ✅ | ✅ | Original compression method |
| Multiple compression | 0x00000200 | ✅ | ✅ | ✅ | ✅ | Combination of methods |
| Huffman encoding | 0x01 | ✅ | ✅ | ✅ | ✅ | Bit value in compression mask |
| Deflate (zlib) | 0x02 | ✅ | ✅ | ✅ | ✅ | Added in Warcraft III |
| Implode (PKWare) | 0x08 | ✅ | ✅ | ✅ | ✅ | Licensed from PKWare |
| BZip2 | 0x10 | ❌ | ✅ | ✅ | ✅ | Added in WoW: The Burning Crusade |
| Sparse compression | 0x20 | ❌ | ❌ | ✅ | ✅ | Added in Starcraft II |
| ADPCM mono | 0x40 | ✅ | ✅ | ✅ | ✅ | For audio compression |
| ADPCM stereo | 0x80 | ✅ | ✅ | ✅ | ✅ | For stereo audio compression |
| LZMA | N/A | ❌ | ❌ | ✅ | ✅ | Added in Starcraft II |

## Block Table Flags

| Flag | Value | Description |
|------|-------|-------------|
| MPQ_FILE_IMPLODE | 0x00000100 | File is compressed using PKWARE Data compression library |
| MPQ_FILE_COMPRESS | 0x00000200 | File is compressed using combination of compression methods |
| MPQ_FILE_ENCRYPTED | 0x00010000 | The file is encrypted |
| MPQ_FILE_FIX_KEY | 0x00020000 | The decryption key is altered according to file position |
| MPQ_FILE_PATCH_FILE | 0x00100000 | File contains incremental patch for existing file |
| MPQ_FILE_SINGLE_UNIT | 0x01000000 | File stored as single unit instead of split into blocks |
| MPQ_FILE_DELETE_MARKER | 0x02000000 | File is a deletion marker |
| MPQ_FILE_SECTOR_CRC | 0x04000000 | File has checksums for each sector |
| MPQ_FILE_EXISTS | 0x80000000 | File exists, reset when deleted |

## Compression Method Flags

| Flag | Value | Description |
|------|-------|-------------|
| MPQ_COMPRESSION_HUFFMANN | 0x01 | Huffmann compression (used on WAVE files only) |
| MPQ_COMPRESSION_ZLIB | 0x02 | ZLIB compression |
| MPQ_COMPRESSION_PKWARE | 0x08 | PKWARE DCL compression |
| MPQ_COMPRESSION_BZIP2 | 0x10 | BZIP2 compression (added in Warcraft III) |
| MPQ_COMPRESSION_SPARSE | 0x20 | Run-length (sparse) compression (added in Starcraft 2) |
| MPQ_COMPRESSION_ADPCM_MONO | 0x40 | IMA ADPCM compression (mono) |
| MPQ_COMPRESSION_ADPCM_STEREO | 0x80 | IMA ADPCM compression (stereo) |
| MPQ_COMPRESSION_LZMA | 0x12 | LZMA compression. Added in Starcraft 2. This value is NOT a combination of flags. |
| MPQ_COMPRESSION_NEXT_SAME | 0xFFFFFFFF | Same compression |

## Implementation Notes

1. The hash table size must be a power of two
2. Maximum number of files depends on hash table size
3. Burning Crusade format (v2) introduced support for archives larger than 4GB
4. Format v3 introduced optional HET and BET tables that can replace hash and block tables
5. Format v4 added MD5 checksums for the tables and header integrity verification
6. Archives in newer games (since 2014) have been replaced by the CASC format
7. Platform codes in hash table entries are a vestigial feature - all known MPQ archives use platform=0.
   Blizzard opted to use separate archives for platform-specific files (e.g., base-Win.MPQ, base-OSX.MPQ)
   rather than utilizing the platform field

### Test Vectors for Implementation Verification

Implementers can use the following test values to verify their MPQ algorithms:

#### Hash Function Test Vectors

| Filename | MPQ_HASH_TABLE_OFFSET (0) | MPQ_HASH_NAME_A (1) | MPQ_HASH_NAME_B (2) | MPQ_HASH_FILE_KEY (3) |
|----------|---------------------------|---------------------|---------------------|------------------------|
| "War3.mpq" | 0xB5E3BF95 | 0xAB8F548C | 0xA9CAF9C1 | 0xF4E26CAD |
| "(attribute)" | 0xD38437CB | 0x07DFEAEC | 0x1CB8E78A | 0xD3C2D58B |
| "(listfile)" | 0xFD5F6EEA | 0x7E4A7FE4 | 0xCABC04F6 | 0xD3F10625 |
| "ARCHIVE" | 0x8E2B8CED | 0x6D7F9E62 | 0xD89B5A0D | 0xB09C6288 |
| "items\\map.doo" | 0xD83EAAD5 | 0xB052F1F6 | 0x5ECCF240 | 0xCA581368 |
| "replay.dat" | 0x70A2E78D | 0x716F3B76 | 0xA00E26BD | 0x34D2D63E |

#### Case Insensitivity Test

These examples demonstrate that different cases produce the same hash values:

| Filename A | Filename B | MPQ_HASH_TABLE_OFFSET |
|------------|------------|------------------------|
| "file.txt" | "FILE.TXT" | 0x1FC54C64 |
| "path\\to\\FILE" | "PATH\\TO\\file" | 0x8844B672 |

#### Path Separator Normalization Test

These examples show that forward and backslashes produce the same hash:

| Filename A | Filename B | MPQ_HASH_TABLE_OFFSET |
|------------|------------|------------------------|
| "path\\to\\file" | "path/to/file" | 0x8844B672 |
| "interface\\glue\\mainmenu.blp" | "interface/glue/mainmenu.blp" | 0x41992D90 |

#### Encryption/Decryption Test Vectors

Test encryption table generation (first and last few values of the 1280-value table):

```
encryptionTable[0x000] = 0x1A790AA9
encryptionTable[0x001] = 0x18DF4175
encryptionTable[0x002] = 0x3C064005
encryptionTable[0x003] = 0x0D66C89C
encryptionTable[0x004] = 0x24C5C5A9
...
encryptionTable[0x4FB] = 0x3C9740B0
encryptionTable[0x4FC] = 0x3C579B79
encryptionTable[0x4FD] = 0x1A3C54E7
encryptionTable[0x4FE] = 0x21B86B73
encryptionTable[0x4FF] = 0x16FEF546
```

Test data encryption/decryption:

```
// Original data (32 bytes)
DWORD originalData[8] = {
    0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0,
    0xFEDCBA98, 0x76543210, 0xF0DEBC9A, 0xE1C3A597
};

// Key = 0xC1EB1CEF
// Expected encrypted data
DWORD encryptedData[8] = {
    0x6DBB9D94, 0x20F0AF34, 0x3A73EA6F, 0x8E82A467,
    0x5F11FC9B, 0xD9BE74FF, 0x82071B61, 0xF1E4D305
};
```

#### Hash Table Entry Examples

Hash table entries for a file "unit\\neutral\\chicken.mdx":

```
// Hash values
dwName1 = 0xB785DF90
dwName2 = 0x0936D252

// Hash table index
dwIndex = 0x4F0C (assuming hash table size 0x1000)

// Hash table entry (before encryption)
{
    dwName1 = 0xB785DF90,
    dwName2 = 0x0936D252,
    lcLocale = 0x0000,
    wPlatform = 0x0000,
    dwBlockIndex = 0x00000123
}
```

#### HET/BET Table Test Vectors

For version 3+ HET tables, Jenkins hash for the same test file:

```
// Jenkins hash for "unit\\neutral\\chicken.mdx"
ULONGLONG jenkins_hash = 0x0E47BAE570E8D3CA
```

These test vectors provide reference values that implementers can use to verify their MPQ handling code. A working implementation should produce identical results when processing these inputs.

## Notable Library Implementations

- **StormLib**: Full-featured C++ library by Ladislav Zezula supporting all MPQ versions
- **JMPQ**: Java implementation with partial support for newer versions
- **mpq.d**: D language implementation
- **libmpq**: C library
- **mpq**: Go library for parsing MPQ files

## References

1. Zezula, Ladislav. "MPQ Format Documentation." [www.zezula.net](http://www.zezula.net/en/mpq/mpqformat.html)
2. StormLib Repository: [github.com/ladislav-zezula/StormLib](https://github.com/ladislav-zezula/StormLib)
3. Olbrantz, Justin and Roy, Jean-Francois. "The MoPaQ Archive Format."
