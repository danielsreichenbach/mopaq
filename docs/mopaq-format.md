# MPQ Format Documentation

## Table of Contents

1. [Introduction](#introduction)
2. [MPQ Archive Structure](#mpq-archive-structure)
   - [Archive Header](#archive-header)
   - [Hash Table](#hash-table)
   - [Block Table](#block-table)
   - [Extended Block Table](#extended-block-table)
   - [Archive Files](#archive-files)
3. [Algorithms](#algorithms)
   - [Hash Functions](#hash-functions)
   - [Encryption and Decryption](#encryption-and-decryption)
   - [Compression](#compression)
   - [Name Search](#name-search)
4. [Rust Implementation Examples](#rust-implementation-examples)
   - [Hash Functions Implementation](#hash-functions-implementation)
   - [Encryption Implementation](#encryption-implementation)
   - [File Extraction](#file-extraction)
   - [Creating an MPQ Archive](#creating-an-mpq-archive)
5. [Performance Benchmarks](#performance-benchmarks)
6. [Common File Formats](#common-file-formats)
7. [References](#references)

## Introduction

The MPQ (Mike O'Brien Pack, or Mo'PaQ) is an archive file format developed by
Blizzard Entertainment for their games, including Diablo, StarCraft, Warcraft
III, and World of Warcraft. It was designed to efficiently store and access game
data, providing features such as:

- Strong encryption
- Multiple compression methods
- File name hashing for fast lookup
- Optional file checksums for integrity verification
- Support for file patches

This format has been used extensively in Blizzard games since 1996 and remains
relevant for modding and game development in the Blizzard ecosystem.

## MPQ Archive Structure

An MPQ archive consists of:

1. An archive header
2. A hash table
3. A block table
4. An extended block table (optional)
5. The actual file data

### Archive Header

The MPQ header identifies the file as an MPQ archive and contains information
about the archive's structure.

```rust
/// MPQ archive header for version 1
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MpqHeader {
    /// Magic number, must be 'MPQ\x1A' (0x1A51504D)
    pub magic: u32,
    /// Size of the archive header
    pub header_size: u32,
    /// Size of the whole archive
    pub archive_size: u32,
    /// MPQ format version (0 for original, 1 for extended)
    pub format_version: u16,
    /// Size of a sector in bytes (usually 4096)
    pub sector_size: u16,
    /// Offset to the hash table
    pub hash_table_offset: u32,
    /// Offset to the block table
    pub block_table_offset: u32,
    /// Number of entries in the hash table
    pub hash_table_entries: u32,
    /// Number of entries in the block table
    pub block_table_entries: u32,
}

/// MPQ archive header for version 2 (extended)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MpqHeaderExt {
    /// Basic header
    pub header: MpqHeader,
    /// Offset to the extended block table
    pub extended_block_table_offset: u64,
    /// High 16 bits of the hash table offset
    pub hash_table_offset_high: u16,
    /// High 16 bits of the block table offset
    pub block_table_offset_high: u16,
    /// Additional fields for version 3 and above...
}
```

The magic number for MPQ archives is always 'MPQ\x1A' (0x1A51504D in little-endian
format). This is used to identify the file as an MPQ archive.

### Hash Table

The hash table is used for fast file lookup using the file names. Each entry in
the hash table is 16 bytes and has the following structure:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HashEntry {
    /// Hash of the file name (using method A)
    pub name_hash_a: u32,
    /// Hash of the file name (using method B)
    pub name_hash_b: u32,
    /// Language of the file
    pub language: u16,
    /// Platform the file is used for
    pub platform: u16,
    /// Index into the block table
    pub block_index: u32,
}
```

The hash table is always encrypted using the hash of the string "(hash table)"
as the key.

### Block Table

The block table contains information about each file in the archive, including
its position, size, and flags.

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BlockEntry {
    /// Offset of the file in the archive
    pub offset: u32,
    /// Compressed size of the file
    pub compressed_size: u32,
    /// Uncompressed size of the file
    pub file_size: u32,
    /// Flags (compression method, encrypted, etc.)
    pub flags: u32,
}
```

Common flags include:

- `0x00000100` - File is compressed
- `0x00000200` - File is encrypted
- `0x00000400` - File has a patch
- `0x00010000` - Block is a sector (used in single-unit files)
- `0x01000000` - File exists
- `0x02000000` - File is a deletion marker

The block table is always encrypted using the hash of the string "(block table)"
as the key.

### Extended Block Table

In MPQ format version 2 and above, an extended block table may be present to
support archives larger than 4GB. This table contains the high bits of the file
offsets.

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExtBlockEntry {
    /// High 16 bits of the file offset
    pub offset_high: u16,
    /// Reserved fields
    pub reserved: [u16; 3],
}
```

### Archive Files

Files within an MPQ archive can be:

1. **Stored as is** - No compression or encryption
2. **Compressed** - Using multiple possible compression methods
3. **Encrypted** - Using a key derived from the file name
4. **Imploded** - A specialized compression format
5. **Stored with sectors** - Large files are divided into sectors for better
   access

## Algorithms

### Hash Functions

MPQ uses two hash functions, known as "hash A" and "hash B", to compute hash
values for file names. A third hash, "hash C", is used as an encryption key.

```rust
/// Computes hash value using MPQ hash algorithm
pub fn hash_string(input: &str, hash_type: HashType) -> u32 {
    let seed1: u32 = match hash_type {
        HashType::TableOffset => 0x1505,
        HashType::NameA => 0x7FED7FED,
        HashType::NameB => 0xEEEEEEEE,
        HashType::FileKey => 0x7EED,
    };

    let seed2: u32 = match hash_type {
        HashType::TableOffset => 0,
        HashType::NameA => 0xEEEEEEEE,
        HashType::NameB => 0xEEEEEEEE,
        HashType::FileKey => 0xEEEE,
    };

    let mut hash: u32 = seed1;

    // Convert to uppercase for case-insensitive hashing
    for c in input.to_uppercase().chars() {
        // Only process ASCII characters to ensure consistent hashing
        let character = c as u8;

        // Update hash value
        hash = hash.wrapping_shl(5).wrapping_add(hash) ^ character as u32;

        // Update seed2 (for NameA, NameB, and FileKey)
        if hash_type != HashType::TableOffset {
            seed2 = seed2.wrapping_add(
                (seed2.wrapping_shl(5)).wrapping_add(character as u32)
            );
        }
    }

    match hash_type {
        HashType::NameA => hash ^ 0xEEEEEEEE,
        HashType::NameB => hash,
        HashType::FileKey | HashType::TableOffset => hash % 0x1000 + seed2 % 0x1000 * 0x1000,
    }
}

pub enum HashType {
    TableOffset, // Used for creating table offsets
    NameA,       // First hash in the hash table
    NameB,       // Second hash in the hash table
    FileKey,     // Used for file encryption
}
```

### Encryption and Decryption

MPQ uses a modified version of Blizzard's proprietary encryption algorithm, which
is a simple XOR-based cipher.

```rust
/// Decrypts an MPQ encrypted data block
pub fn decrypt_data(data: &mut [u8], key: u32) {
    let mut seed1: u32 = 0xEEEEEEEE;
    let mut ch: u32;

    // Process data in 32-bit (4-byte) chunks
    let mut data_u32 = unsafe {
        std::slice::from_raw_parts_mut(
            data.as_mut_ptr() as *mut u32,
            data.len() / 4
        )
    };

    for value in data_u32.iter_mut() {
        seed1 = seed1.wrapping_add(ENCRYPTION_TABLE[((key & 0xFF) as usize)]);
        ch = *value ^ (key.wrapping_add(seed1));

        *value = ch;

        key = ((!(key << 21)).wrapping_add(0x11111111)) | (key >> 11);
        seed1 = ch.wrapping_add(seed1).wrapping_add(seed1 << 5).wrapping_add(3);
    }

    // Process any remaining bytes
    let remaining_start = data_u32.len() * 4;
    if remaining_start < data.len() {
        let mut last_value: u32 = 0;
        let mut shift: u32 = 0;

        for i in remaining_start..data.len() {
            seed1 = seed1.wrapping_add(ENCRYPTION_TABLE[((key & 0xFF) as usize)]);

            // Process each byte
            let byte = data[i];
            let encrypted_byte = byte as u32 ^ ((key.wrapping_add(seed1) >> (shift * 8)) & 0xFF);
            data[i] = encrypted_byte as u8;

            last_value |= (encrypted_byte << (shift * 8));
            shift += 1;

            if shift == 4 {
                key = ((!(key << 21)).wrapping_add(0x11111111)) | (key >> 11);
                seed1 = last_value.wrapping_add(seed1).wrapping_add(seed1 << 5).wrapping_add(3);

                last_value = 0;
                shift = 0;
            }
        }

        // Final key update if we processed any odd bytes
        if shift > 0 {
            key = ((!(key << 21)).wrapping_add(0x11111111)) | (key >> 11);
            seed1 = last_value.wrapping_add(seed1).wrapping_add(seed1 << 5).wrapping_add(3);
        }
    }
}

/// Encryption is the same as decryption in this algorithm (XOR-based)
pub fn encrypt_data(data: &mut [u8], key: u32) {
    decrypt_data(data, key);
}

/// Generates the encryption key for a file based on its name
pub fn generate_file_key(filename: &str, offset: u32, size: u32) -> u32 {
    let filename_uppercased = filename.to_uppercase();

    // Use hash algorithm to create a key
    let mut key = hash_string(&filename_uppercased, HashType::FileKey);

    // Adjust the key based on file offset and size
    key = (key + offset) ^ size;

    key
}
```

The `ENCRYPTION_TABLE` is a 256-entry array of randomly generated values that's
used in the encryption algorithm. It's a constant table that's hardcoded into
the implementation.

### Compression

MPQ supports multiple compression methods, which can be combined. The supported
methods are:

1. **Huffman encoding** (type 0x01)
2. **zlib/deflate** (type 0x02)
3. **PKWare DCL compression** (type 0x08)
4. **bzip2 compression** (type 0x10)
5. **LZMA compression** (type 0x12)
6. **Sparse compression** (type 0x20)
7. **IMA ADPCM mono compression** (type 0x40)
8. **IMA ADPCM stereo compression** (type 0x80)

Each compression type is represented by a bit in the compression field. Multiple
compression methods can be applied by OR-ing their values.

```rust
pub enum CompressionType {
    Huffman = 0x01,
    Zlib = 0x02,
    PKWare = 0x08,
    BZip2 = 0x10,
    Lzma = 0x12,
    Sparse = 0x20,
    ImaAdpcmMono = 0x40,
    ImaAdpcmStereo = 0x80,
}

/// Decompresses a data block based on the compression types used
pub fn decompress_data(
    input: &[u8],
    output_size: usize,
    compression_mask: u8
) -> Result<Vec<u8>, MpqError> {
    let mut result = Vec::with_capacity(output_size);
    let mut temp_buffer = input.to_vec();

    // Apply decompressions in reverse order
    for i in (0..8).rev() {
        let compression_bit = 1 << i;

        if compression_mask & compression_bit == 0 {
            continue;
        }

        match compression_bit {
            0x01 => {
                // Huffman decompression
                temp_buffer = decompress_huffman(&temp_buffer, output_size)?;
            },
            0x02 => {
                // zlib decompression
                temp_buffer = decompress_zlib(&temp_buffer, output_size)?;
            },
            0x08 => {
                // PKWare DCL decompression (not implemented in this example)
                return Err(MpqError::UnsupportedCompression("PKWare DCL not implemented".into()));
            },
            // ... other decompression methods
            _ => {
                return Err(MpqError::UnsupportedCompression(
                    format!("Compression type 0x{:02X} not supported", compression_bit)
                ));
            }
        }
    }

    result = temp_buffer;

    // Ensure output size is correct
    if result.len() != output_size {
        return Err(MpqError::DecompressionFailed(
            format!("Expected {} bytes, got {}", output_size, result.len())
        ));
    }

    Ok(result)
}
```

### Name Search

To find a file in an MPQ archive, the client computes two hashes of the file
name and searches for these hashes in the hash table.

```rust
/// Search for a file in the MPQ archive using its name
pub fn find_file(
    archive: &MpqArchive,
    filename: &str
) -> Result<Option<(HashEntry, BlockEntry)>, MpqError> {
    // Calculate both hash values for the file name
    let hash_a = hash_string(filename, HashType::NameA);
    let hash_b = hash_string(filename, HashType::NameB);

    // Get the starting position in the hash table
    let start_pos = (hash_a & (archive.header.hash_table_entries - 1)) as usize;

    // Search through the hash table
    let mut current_pos = start_pos;

    loop {
        let hash_entry = &archive.hash_table[current_pos];

        // Check if we've reached an empty slot
        if hash_entry.block_index == 0xFFFFFFFF {
            break;
        }

        // Check if this is our file
        if hash_entry.name_hash_a == hash_a &&
           hash_entry.name_hash_b == hash_b {
            // Found the file
            if hash_entry.block_index >= archive.header.block_table_entries as u32 {
                return Err(MpqError::InvalidBlockIndex(hash_entry.block_index));
            }

            let block_entry = archive.block_table[hash_entry.block_index as usize];

            return Ok(Some((hash_entry.clone(), block_entry.clone())));
        }

        // Move to the next position (with wrap-around)
        current_pos = (current_pos + 1) % archive.header.hash_table_entries as usize;

        // If we've checked all entries, break
        if current_pos == start_pos {
            break;
        }
    }

    // File not found
    Ok(None)
}
```

## Rust Implementation Examples

Here are some examples of implementing MPQ functionality in Rust.

### Hash Functions Implementation

```rust
use std::collections::HashMap;

// Lazy static for the encryption table (pre-computed)
lazy_static! {
    static ref ENCRYPTION_TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut seed = 0x00100001;

        for i in 0..256 {
            let mut index = i;
            for j in 0..5 {
                seed = (seed * 125 + 3) % 0x2AAAAB;
                let temp1 = (seed & 0xFFFF) << 0x10;

                seed = (seed * 125 + 3) % 0x2AAAAB;
                let temp2 = seed & 0xFFFF;

                table[index as usize] = temp1 | temp2;
                index += 0x100;
            }
        }

        table
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    TableOffset,
    NameA,
    NameB,
    FileKey,
}

/// Computes a hash value from a string using the MPQ hash algorithm
pub fn hash_string(input: &str, hash_type: HashType) -> u32 {
    let seed1: u32 = match hash_type {
        HashType::TableOffset => 0x1505,
        HashType::NameA => 0x7FED7FED,
        HashType::NameB => 0xEEEEEEEE,
        HashType::FileKey => 0x7EED,
    };

    let seed2: u32 = match hash_type {
        HashType::TableOffset => 0,
        HashType::NameA => 0xEEEEEEEE,
        HashType::NameB => 0xEEEEEEEE,
        HashType::FileKey => 0xEEEE,
    };

    let mut hash: u32 = seed1;
    let mut seed = seed2;

    // Convert to uppercase for case-insensitive hashing
    let input_upper = input.to_uppercase();

    for c in input_upper.bytes() {
        hash = (hash << 5).wrapping_add(hash) ^ (c as u32);

        if hash_type != HashType::TableOffset {
            seed = seed.wrapping_add((seed << 5).wrapping_add(c as u32));
        }
    }

    match hash_type {
        HashType::NameA => hash ^ 0xEEEEEEEE,
        HashType::NameB => hash,
        HashType::TableOffset | HashType::FileKey => (hash % 0x1000) + (seed % 0x1000) * 0x1000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_functions() {
        // Test cases based on known hash values
        assert_eq!(hash_string("(hash table)", HashType::FileKey), 0xC3AF3770);
        assert_eq!(hash_string("(block table)", HashType::FileKey), 0xEC83B3A3);

        // Test name hash functions
        assert_eq!(hash_string("War3.mpq\\(listfile)", HashType::NameA), 0x95775582);
        assert_eq!(hash_string("War3.mpq\\(listfile)", HashType::NameB), 0xC4ED1798);

        // Test case insensitivity
        assert_eq!(
            hash_string("WAR3.MPQ\\(LISTFILE)", HashType::NameA),
            hash_string("War3.mpq\\(listfile)", HashType::NameA)
        );
    }
}
```

### Encryption Implementation

```rust
/// Decrypt a block of data using the MPQ encryption algorithm
pub fn decrypt_block(data: &mut [u8], key: u32) -> Result<(), MpqError> {
    if data.len() % 4 != 0 {
        return Err(MpqError::InvalidDataSize(
            "Data size must be a multiple of 4 for decryption".into()
        ));
    }

    let mut seed = 0xEEEEEEEE;
    let mut current_key = key;

    // Process data in 32-bit chunks
    let chunks = data.len() / 4;
    let data_ptr = data.as_mut_ptr() as *mut u32;
    let data_slice = unsafe { std::slice::from_raw_parts_mut(data_ptr, chunks) };

    for chunk in data_slice.iter_mut() {
        seed = seed.wrapping_add(ENCRYPTION_TABLE[(current_key & 0xFF) as usize]);
        let value = *chunk ^ current_key.wrapping_add(seed);

        // Update key for next iteration
        current_key = ((!(current_key << 21)).wrapping_add(0x11111111))
            | (current_key >> 11);

        // Update seed for next iteration
        seed = value.wrapping_add(seed).wrapping_add(seed << 5).wrapping_add(3);

        *chunk = value;
    }

    Ok(())
}

/// Encrypt a block of data using the MPQ encryption algorithm (identical to decrypt)
pub fn encrypt_block(data: &mut [u8], key: u32) -> Result<(), MpqError> {
    decrypt_block(data, key)
}

/// Calculate the encryption key for a file
pub fn calculate_file_key(filename: &str, offset: u32, size: u32) -> u32 {
    let name_hash = hash_string(filename, HashType::FileKey);
    (name_hash.wrapping_add(offset) ^ size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption() {
        // Create test data
        let mut test_data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let key = 0xDEADBEEF;

        // Make a copy for verification
        let original_data = test_data.clone();

        // Encrypt the data
        encrypt_block(&mut test_data, key).unwrap();

        // Data should be different after encryption
        assert_ne!(test_data, original_data);

        // Decrypt the data (should restore original)
        decrypt_block(&mut test_data, key).unwrap();

        // Verify the data is restored
        assert_eq!(test_data, original_data);
    }
}
```

### File Extraction

```rust
/// Extract a file from an MPQ archive
pub fn extract_file(
    archive: &MpqArchive,
    filename: &str
) -> Result<Vec<u8>, MpqError> {
    // Find the file in the archive
    let file_info = match find_file(archive, filename)? {
        Some(info) => info,
        None => return Err(MpqError::FileNotFound(filename.to_string())),
    };

    let (_, block_entry) = file_info;

    // Check if the file exists
    if block_entry.flags & 0x01000000 == 0 {
        return Err(MpqError::FileNotFound(filename.to_string()));
    }

    // Calculate the file position
    let file_pos = block_entry.offset;

    // Create a file reader at the correct position
    let mut reader = io::Cursor::new(&archive.data[file_pos as usize..]);

    // Read the file based on its flags
    let mut result = Vec::with_capacity(block_entry.file_size as usize);

    if block_entry.flags & 0x00000100 != 0 {
        // File is compressed
        if block_entry.flags & 0x00010000 != 0 {
            // Single unit file
            let compressed_size = block_entry.compressed_size as usize;
            let mut compressed_data = vec![0u8; compressed_size];
            reader.read_exact(&mut compressed_data)?;

            // Check if file is encrypted
            if block_entry.flags & 0x00000200 != 0 {
                // Calculate encryption key
                let key = calculate_file_key(filename, file_pos, block_entry.file_size);
                decrypt_block(&mut compressed_data, key)?;
            }

            // Decompress the data
            let compression_mask = compressed_data[0]; // First byte is compression mask
            result = decompress_data(&compressed_data[1..], block_entry.file_size as usize, compression_mask)?;
        } else {
            // File divided into sectors
            let sector_size = archive.header.sector_size as u32;
            let sectors = (block_entry.file_size + sector_size - 1) / sector_size;

            // Read sector offsets table
            let mut sector_offsets = vec![0u32; sectors as usize + 1];
            for i in 0..=sectors as usize {
                sector_offsets[i] = reader.read_u32::<LittleEndian>()?;
            }

            // Process each sector
            for i in 0..sectors as usize {
                let sector_start = sector_offsets[i] as usize;
                let sector_end = sector_offsets[i + 1] as usize;
                let sector_bytes = sector_end - sector_start;

                // Read the sector data
                let mut sector_data = vec![0u8; sector_bytes];
                reader.seek(io::SeekFrom::Start((file_pos + sector_start) as u64))?;
                reader.read_exact(&mut sector_data)?;

                // Check if sector is encrypted
                if block_entry.flags & 0x00000200 != 0 {
                    // Calculate encryption key
                    let key = calculate_file_key(
                        filename,
                        file_pos + sector_start as u32,
                        block_entry.file_size
                    );
                    decrypt_block(&mut sector_data, key)?;
                }

                // Check if sector is compressed
                if sector_bytes < sector_size as usize {
                    // Sector is compressed
                    let compression_mask = sector_data[0]; // First byte is compression mask
                    let decompressed = decompress_data(
                        &sector_data[1..],
                        std::cmp::min(sector_size as usize, block_entry.file_size as usize - i * sector_size as usize),
                        compression_mask
                    )?;
                    result.extend_from_slice(&decompressed);
                } else {
                    // Sector is not compressed
                    result.extend_from_slice(&sector_data);
                }
            }
        }
    } else {
        // File is stored as-is
        let mut file_data = vec![0u8; block_entry.file_size as usize];
        reader.read_exact(&mut file_data)?;

        // Check if file is encrypted
        if block_entry.flags & 0x00000200 != 0 {
            // Calculate encryption key
            let key = calculate_file_key(filename, file_pos, block_entry.file_size);
            decrypt_block(&mut file_data, key)?;
        }

        result = file_data;
    }

    Ok(result)
}
```

### Creating an MPQ Archive

```rust
pub struct MpqBuilder {
    files: HashMap<String, Vec<u8>>,
    sector_size: u16,
    hash_table_size: u32,
}

impl MpqBuilder {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            sector_size: 4096, // Default sector size
            hash_table_size: 1024, // Default hash table size (must be power of 2)
        }
    }

    pub fn with_sector_size(mut self, sector_size: u16) -> Self {
        self.sector_size = sector_size;
        self
    }

    pub fn with_hash_table_size(mut self, size: u32) -> Self {
        // Ensure size is a power of 2
        let size = size.next_power_of_two();
        self.hash_table_size = size;
        self
    }

    pub fn add_file<S: AsRef<str>>(mut self, name: S, data: Vec<u8>) -> Self {
        self.files.insert(name.as_ref().to_string(), data);
        self
    }

    pub fn build(self) -> Result<Vec<u8>, MpqError> {
        if self.files.is_empty() {
            return Err(MpqError::NoFilesToArchive);
        }

        // Calculate sizes and offsets
        let header_size = std::mem::size_of::<MpqHeader>();
        let hash_table_size = self.hash_table_size as usize * std::mem::size_of::<HashEntry>();
        let block_table_size = self.files.len() * std::mem::size_of::<BlockEntry>();

        // Start with header, hash table and block table
        let mut total_size = header_size + hash_table_size + block_table_size;

        // Align to sector size
        total_size = (total_size + self.sector_size as usize - 1) & !(self.sector_size as usize - 1);

        // Initial file offset
        let mut file_offset = total_size as u32;

        // Create the block table entries
        let mut block_entries = Vec::with_capacity(self.files.len());
        let mut file_data = Vec::new();

        for (name, data) in &self.files {
            let file_size = data.len() as u32;

            // For simplicity, store files uncompressed in this example
            let flags = 0x01000000; // FILE_EXISTS flag

            // Add the block entry
            block_entries.push(BlockEntry {
                offset: file_offset,
                compressed_size: file_size,
                file_size,
                flags,
            });

            // Add file data
            file_data.extend_from_slice(data);

            // Update offset for next file
            file_offset += file_size;

            // Align to sector size
            file_offset = (file_offset + self.sector_size as u32 - 1) & !(self.sector_size as u32 - 1);
        }

        // Calculate final archive size
        let archive_size = file_offset;

        // Create hash table (initialize all entries to DELETED)
        let mut hash_table = vec![
            HashEntry {
                name_hash_a: 0xFFFFFFFF,
                name_hash_b: 0xFFFFFFFF,
                language: 0xFFFF,
                platform: 0xFFFF,
                block_index: 0xFFFFFFFF,
            };
            self.hash_table_size as usize
        ];

        // Fill the hash table
        for (i, (name, _)) in self.files.iter().enumerate() {
            let hash_a = hash_string(name, HashType::NameA);
            let hash_b = hash_string(name, HashType::NameB);

            // Find position in hash table
            let mut pos = (hash_a & (self.hash_table_size - 1)) as usize;

            while hash_table[pos].block_index != 0xFFFFFFFF &&
                  hash_table[pos].name_hash_a != 0xFFFFFFFF {
                pos = (pos + 1) % self.hash_table_size as usize;
            }

            // Add entry to hash table
            hash_table[pos] = HashEntry {
                name_hash_a: hash_a,
                name_hash_b: hash_b,
                language: 0, // Default language
                platform: 0, // Default platform
                block_index: i as u32,
            };
        }

        // Create the file buffer
        let mut result = Vec::with_capacity(archive_size as usize);

        // Write the header
        let header = MpqHeader {
            magic: 0x1A51504D, // 'MPQ\x1A'
            header_size: header_size as u32,
            archive_size,
            format_version: 0,
            sector_size: self.sector_size,
            hash_table_offset: header_size as u32,
            block_table_offset: (header_size + hash_table_size) as u32,
            hash_table_entries: self.hash_table_size,
            block_table_entries: block_entries.len() as u32,
        };

        // Write header to result
        result.extend_from_slice(unsafe {
            std::slice::from_raw_parts(
                &header as *const _ as *const u8,
                std::mem::size_of::<MpqHeader>()
            )
        });

        // Encrypt and write hash table
        let mut hash_table_bytes = unsafe {
            std::slice::from_raw_parts(
                hash_table.as_ptr() as *const u8,
                hash_table.len() * std::mem::size_of::<HashEntry>()
            )
        }.to_vec();

        encrypt_block(&mut hash_table_bytes, hash_string("(hash table)", HashType::FileKey))?;
        result.extend_from_slice(&hash_table_bytes);

        // Encrypt and write block table
        let mut block_table_bytes = unsafe {
            std::slice::from_raw_parts(
                block_entries.as_ptr() as *const u8,
                block_entries.len() * std::mem::size_of::<BlockEntry>()
            )
        }.to_vec();

        encrypt_block(&mut block_table_bytes, hash_string("(block table)", HashType::FileKey))?;
        result.extend_from_slice(&block_table_bytes);

        // Pad to file_offset
        result.resize(total_size, 0);

        // Write file data
        result.extend_from_slice(&file_data);

        // Ensure final size matches expected size
        assert_eq!(result.len(), archive_size as usize);

        Ok(result)
    }
}
```

## Performance Benchmarks

Here are some benchmarks for the key MPQ operations:

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_hash_string(b: &mut Bencher) {
        b.iter(|| {
            hash_string("War3.mpq\\(listfile)", HashType::NameA);
            hash_string("War3.mpq\\(listfile)", HashType::NameB);
            hash_string("War3.mpq\\(listfile)", HashType::FileKey);
        });
    }

    #[bench]
    fn bench_decrypt_small_block(b: &mut Bencher) {
        let mut data = [0u8; 128];
        for i in 0..data.len() {
            data[i] = i as u8;
        }

        b.iter(|| {
            let mut test_data = data.clone();
            decrypt_block(&mut test_data, 0xDEADBEEF).unwrap();
        });
    }

    #[bench]
    fn bench_decrypt_large_block(b: &mut Bencher) {
        let mut data = [0u8; 4096];
        for i in 0..data.len() {
            data[i] = i as u8;
        }

        b.iter(|| {
            let mut test_data = data.clone();
            decrypt_block(&mut test_data, 0xDEADBEEF).unwrap();
        });
    }

    #[bench]
    fn bench_find_file(b: &mut Bencher) {
        // Create a small test archive with 100 files
        let mut builder = MpqBuilder::new().with_hash_table_size(256);

        for i in 0..100 {
            let filename = format!("test/file{}.txt", i);
            let content = vec![i as u8; 100];
            builder = builder.add_file(filename, content);
        }

        let archive_data = builder.build().unwrap();
        let archive = MpqArchive::from_data(archive_data).unwrap();

        b.iter(|| {
            find_file(&archive, "test/file50.txt").unwrap();
        });
    }
}
```

## StormLib Implementation Comparison

StormLib, maintained by Ladislav Zezula, is considered the reference implementation
for MPQ handling. This section compares our Rust implementation with StormLib's
approach and highlights important differences implementers should be aware of.

### Encryption Table Generation

In StormLib, the encryption table is generated once at initialization:

```cpp
// From StormLib/src/SBaseCommon.cpp
static DWORD InitializeMpqCryptography()
{
    DWORD dwSeed = 0x00100001;
    DWORD index1 = 0;
    DWORD index2 = 0;
    DWORD i;

    // Initialize the decryption tables
    for(index1 = 0; index1 < 0x100; index1++)
    {
        for(index2 = index1, i = 0; i < 5; i++, index2 += 0x100)
        {
            DWORD temp1, temp2;

            dwSeed = (dwSeed * 125 + 3) % 0x2AAAAB;
            temp1  = (dwSeed & 0xFFFF) << 0x10;

            dwSeed = (dwSeed * 125 + 3) % 0x2AAAAB;
            temp2  = (dwSeed & 0xFFFF);

            StormBuffer[index2] = (temp1 | temp2);
        }
    }

    // Success
    bCryptographyInitialized = TRUE;
    return ERROR_SUCCESS;
}
```

Our Rust implementation uses a `lazy_static` approach that generates the table on first use. The mathematical algorithm is identical, but implementers should be aware that:

1. The table is always the same - it's a static set of values
2. StormLib initializes it at the start of the program
3. Our Rust implementation generates it on first use

### Hash Functions

StormLib's implementation of hash functions closely matches our Rust version, with a few nuances:

```cpp
// From StormLib/src/SBaseCommon.cpp
DWORD HashString(const char * szFileName, DWORD dwHashType)
{
    DWORD dwSeed1 = 0x7FED7FED;
    DWORD dwSeed2 = 0xEEEEEEEE;
    DWORD ch;

    while(*szFileName != 0)
    {
        ch = toupper(*szFileName++);

        dwSeed1 = StormBuffer[dwHashType + ch] ^ (dwSeed1 + dwSeed2);
        dwSeed2 = ch + dwSeed1 + dwSeed2 + (dwSeed2 << 5) + 3;
    }

    return dwSeed1;
}
```

Key differences:

1. StormLib uses a pre-computed table lookup for faster calculation
2. The basic algorithm is mathematically equivalent but uses a different implementation approach
3. StormLib's version is more optimized for speed but less clear about what's happening

Implementers should ensure their hash function produces identical results, regardless of implementation approach, as the hash values must match exactly for file lookups to succeed.

### Encryption/Decryption

StormLib's encryption and decryption functions use an optimized approach:

```cpp
// From StormLib/src/SBaseCommon.cpp
void DecryptBlock(void * pvDataBlock, DWORD dwLength, DWORD dwKey)
{
    DWORD * pdwDataBlock = (DWORD *)pvDataBlock;
    DWORD dwSeed1 = 0xEEEEEEEE;
    DWORD dwSeed2 = 0xEEEEEEEE;
    DWORD ch;

    // Round to DWORDs
    dwLength >>= 2;

    // Decrypt the data block
    for(DWORD i = 0; i < dwLength; i++)
    {
        dwSeed2 += StormBuffer[0x400 + (dwKey & 0xFF)];
        ch = pdwDataBlock[i];
        ch = ch ^ (dwKey + dwSeed1);
        pdwDataBlock[i] = ch;
        dwKey = ((~dwKey << 0x15) + 0x11111111) | (dwKey >> 0x0B);
        dwSeed1 = ch + dwSeed1 + (dwSeed1 << 5) + 3;
    }
}
```

Important differences:

1. StormLib pre-shifts the data length by 2 (divide by 4) to process DWORDs
2. It uses the encryption table offset by 0x400 (1024) for the seed calculation
3. The key rotation is expressed slightly differently but mathematically equivalent

Our Rust version handles non-aligned data sizes more explicitly, which StormLib doesn't do in this core function. Instead, StormLib has separate functions for handling odd-sized buffers.

### Compression Handling

StormLib supports multiple compression methods and chains them in a specific order:

```cpp
// From StormLib/src/SCommon.cpp
int SCompDecompress(void * pvOutBuffer, int * pcbOutBuffer, void * pvInBuffer, int cbInBuffer)
{
    // Get the compression type from the first byte of the input buffer
    BYTE * pbInBuffer = (BYTE *)pvInBuffer;
    BYTE * pbOutBuff = (BYTE *)pvOutBuffer;
    int cbOutBuffer = *pcbOutBuffer;
    int nResult = ERROR_SUCCESS;

    // Is it compressed by PKWARE Data Compression Library?
    if(cbInBuffer > 1 && *pbInBuffer == 'P')
    {
        // We need to decompress the data using Pkware DCL
        if(DecompressPklib(pbOutBuff, cbOutBuffer, pbInBuffer+1, cbInBuffer-1) == false)
            nResult = ERROR_FILE_CORRUPT;
    }
    // Is it compressed by Blizzard's multiple compression ?
    else if(cbInBuffer > 2 && *pbInBuffer == 'B' && *(pbInBuffer+1) <= 5)
    {
        // We need to decompress the data using Blizzard compression
        nResult = DecompressMulti(pbOutBuff, pcbOutBuffer, pbInBuffer+1, cbInBuffer-1);
    }
    else
    // Is it compressed by ZLIB ?
    if(cbInBuffer > 2 && *pbInBuffer == 'Z')
    {
        // We need to decompress the data using ZLIB
        nResult = DecompressZlib(pbOutBuff, pcbOutBuffer, pbInBuffer+1, cbInBuffer-1);
    }
    // Is it compressed by BZIP2 ?
    else if(cbInBuffer > 2 && *pbInBuffer == '2')
    {
        // We need to decompress the data using BZIP2
        nResult = DecompressBzip2(pbOutBuff, pcbOutBuffer, pbInBuffer+1, cbInBuffer-1);
    }
    // Is it a SPARSE file?
    else if(cbInBuffer > 2 && *pbInBuffer == 'S')
    {
        // We need to decompress the sparse file
        nResult = DecompressSparse(pbOutBuff, pcbOutBuffer, pbInBuffer+1, cbInBuffer-1);
    }
    // Not compressed, just copy the data
    else if(cbOutBuffer >= cbInBuffer)
    {
        memcpy(pbOutBuff, pbInBuffer, cbInBuffer);
        *pcbOutBuffer = cbInBuffer;
    }
    else
    {
        *pcbOutBuffer = 0;
        nResult = ERROR_INSUFFICIENT_BUFFER;
    }

    return nResult;
}
```

Key differences:

1. StormLib uses a letter-based prefix system ('P', 'B', 'Z', '2', 'S') rather than bit flags
2. StormLib has dedicated functions for each compression method
3. The multi-compression ('B') approach in StormLib chains multiple algorithms

Our Rust implementation uses a bit-flag approach which is closer to the MPQ specification. Implementers should be aware that real MPQ files might use either approach, so a robust implementation should handle both.

### File Lookup and Handling

StormLib's file lookup mechanism uses a more complex approach to handle locale settings and platform-specific files:

```cpp
// From StormLib/src/SFileFind.cpp
TMPQFile * CreateMpqFile(TMPQArchive * ha)
{
    TMPQFile * hf;

    hf = STORM_ALLOC(TMPQFile, 1);
    if(hf != NULL)
    {
        memset(hf, 0, sizeof(TMPQFile));
        hf->filename = NULL;
        hf->pStream = NULL;
        hf->hFile = SFILE_INVALID_HANDLE;
        hf->ha = ha;
    }

    return hf;
}

BOOL SFileOpenFileEx(HANDLE hMpq, const char * szFileName, DWORD dwSearchScope, HANDLE * phFile)
{
    // ... (error checking code)

    // Find the file within the MPQ
    dwErrCode = FindFile(ha, szFileName, &pFileEntry, lcLocale);

    // ... (more implementation)
}
```

Important implementation details:

1. StormLib tracks locale settings and platform specifics more thoroughly
2. It has optimizations for partial file loading
3. It manages file handles differently than our more straightforward Rust approach

Implementers should consider these additional complexities for a production-ready implementation.

### Sector-based Reading

StormLib has a complex mechanism for reading sector-based files:

```cpp
// From StormLib/src/SFileReadFile.cpp
static int ReadMPQFileSectors(TMPQFile * hf, void * pvBuffer, DWORD dwStartSector, DWORD dwSectorCount, LPDWORD pdwBytesRead)
{
    // ... (complicated implementation)
}
```

Key differences:

1. StormLib handles partial sector reads more robustly
2. It has optimizations for reading specific sectors rather than the whole file
3. It manages sector checksums and verification

Our Rust implementation provides a simpler approach that reads entire files, which is sufficient for most use cases but may not be as optimized for large files with partial reads.

### Extended Attributes Support

StormLib has evolved to support multiple versions of the MPQ format:

```cpp
// From StormLib/src/SFileAttributes.cpp
int SFileGetAttributes(HANDLE hMpq)
{
    TMPQArchive * ha = (TMPQArchive *)hMpq;

    // Check valid parameters
    if(!IsValidMpqHandle(ha))
        return SFILE_INVALID_ATTRIBUTES;

    return ha->dwFlags;
}

int SFileSetAttributes(HANDLE hMpq, DWORD dwFlags)
{
    TMPQArchive * ha = (TMPQArchive *)hMpq;
    DWORD dwOldFlags;
    DWORD dwNewFlags;

    // Check valid parameters
    if(!IsValidMpqHandle(ha))
        return ERROR_INVALID_PARAMETER;

    // Not all flags can be set directly by the user
    dwNewFlags = dwFlags & MPQ_ATTRIBUTE_ALL;
    dwOldFlags = ha->dwFlags;

    // Set the attributes
    ha->dwFlags = dwNewFlags;

    // Return the old attributes
    return dwOldFlags;
}
```

StormLib supports:

1. MPQ format versions 1-4
2. HET and BET tables (high-efficiency tables)
3. Archive attributes and extended attributes
4. Strong signatures and checksums

Our Rust implementation focuses on the core version 1 format for simplicity, but a production implementation might need to handle these extended features for full compatibility.

### Practical Considerations

When implementing a Rust MPQ library with StormLib compatibility in mind:

1. **Test against real game files**: Test your implementation against actual MPQ files from Blizzard games
2. **Verify hash outputs**: Ensure your hash functions produce identical results to StormLib
3. **Handle edge cases**: MPQ archives have many edge cases involving compression, encryption, and file format variations
4. **Check error handling**: Compare your error cases with StormLib's to ensure consistent behavior
5. **Benchmark against StormLib**: Compare performance to identify potential optimization opportunities

### Example: Validating Hash Function Compatibility

To verify your hash function produces StormLib-compatible results:

```rust
#[test]
fn test_stormlib_hash_compatibility() {
    // Known values from StormLib
    let test_cases = [
        ("(hash table)", HashType::FileKey, 0xC3AF3770),
        ("(block table)", HashType::FileKey, 0xEC83B3A3),
        ("(listfile)", HashType::NameA, 0x1DA8B0CF),
        ("(attributes)", HashType::NameA, 0x29AECE40),
        // Add more test cases from StormLib source
    ];

    for (input, hash_type, expected) in test_cases.iter() {
        let result = hash_string(input, *hash_type);
        assert_eq!(result, *expected,
            "Hash mismatch for '{}' using {:?}: expected 0x{:08X}, got 0x{:08X}",
            input, hash_type, expected, result);
    }
}
```

## References

1. [MPQ Format Documentation](http://www.zezula.net/en/mpq/mpqformat.html)
2. [StormLib](https://github.com/ladislav-zezula/StormLib)
3. [ceres-mpq](https://github.com/ceres-wc3/ceres-mpq)
4. [World of Warcraft development wiki](https://wowdev.wiki/)
5. [image-blp](https://github.com/zloy-tulen/image-blp)
6. [libwarcraft](https://github.com/WowDevTools/libwarcraft)
7. [StormLib Source Code](https://github.com/ladislav-zezula/StormLib/tree/master/src)
