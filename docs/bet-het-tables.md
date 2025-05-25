# MPQ BET/HET Tables and Bit Manipulation Documentation

## Overview

The HET (Hash Extended Table) and BET (Block Extended Table) were introduced in MPQ format version 3 as more efficient alternatives to the traditional hash and block tables. These tables use bit-packed storage to reduce memory usage and improve performance, especially for archives with many files.

## Key Concepts

### Why BET/HET Tables?

Traditional MPQ tables have limitations:

- **Hash Table**: Fixed 16-byte entries, many wasted bits
- **Block Table**: Fixed 16-byte entries, doesn't scale well for large files

BET/HET tables solve these issues by:

- Using variable bit widths based on actual data ranges
- Eliminating redundant information
- Supporting larger archives more efficiently

### Table Relationship

```
File Lookup Process:
1. Hash filename → HET table → File Index
2. File Index → BET table → File Location/Size/Flags
```

## HET Table (Hash Extended Table)

The HET table provides fast file lookups using a hash-based approach similar to the traditional hash table but with better space efficiency.

### HET Table Structure

```rust
/// HET table header structure
#[repr(C, packed)]
struct HetTable {
    /// Signature 'HET\x1A' (0x1A544548 in little-endian)
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
    hash_entry_size: u32,  // How many bits per hash entry

    /// Total size of file index in bits
    total_index_size: u32,

    /// Extra bits in the file index
    index_size_extra: u32,

    /// Effective size of the file index in bits
    index_size: u32,

    /// Size of the block index subtable in bytes
    block_table_size: u32,
}
```

### HET Table Data Layout

After the header, the HET table contains:

1. **Hash Table**: Array of hash entries (bit-packed)
2. **File Index Array**: Maps hash table positions to file indices (bit-packed)

```
[HET Header][Hash Table (bit-packed)][File Index Array (bit-packed)]
```

### HET Hash Calculation

```rust
/// Calculate HET hash using Jenkins hash algorithm
fn calculate_het_hash(filename: &str) -> u64 {
    let mut hash: u64 = 0;

    for &byte in filename.as_bytes() {
        let mut ch = byte;

        // Normalize path separators
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to lowercase (HET uses lowercase, unlike traditional hash)
        ch = ASCII_TO_LOWER_TABLE[ch as usize];

        // Jenkins one-at-a-time hash
        hash = hash.wrapping_add(ch as u64);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }

    // Final mixing
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);

    hash
}
```

### HET Bit Extraction

```rust
impl HetTable {
    /// Extract hash entry from bit-packed data
    fn get_hash_entry(&self, data: &[u8], index: usize) -> u64 {
        let bit_offset = index * self.hash_entry_size as usize;
        extract_bits(data, bit_offset, self.hash_entry_size)
    }

    /// Extract file index from bit-packed data
    fn get_file_index(&self, data: &[u8], index: usize) -> u32 {
        let hash_table_bits = self.hash_table_size * 8;
        let bit_offset = hash_table_bits as usize + (index * self.index_size as usize);
        extract_bits(data, bit_offset, self.index_size) as u32
    }

    /// Find a file in the HET table
    fn find_file(&self, data: &[u8], filename: &str) -> Option<u32> {
        // Calculate hash
        let hash = calculate_het_hash(filename);

        // Create hash mask based on entry size
        let hash_mask = (1u64 << self.hash_entry_size) - 1;
        let index_mask = (1u64 << self.index_size) - 1;

        // Get table size in entries
        let table_entries = (self.hash_table_size * 8) / self.hash_entry_size;

        // Start position in hash table
        let start_index = (hash & (table_entries - 1) as u64) as usize;

        // Linear probing for collision resolution
        for i in 0..table_entries as usize {
            let index = (start_index + i) % table_entries as usize;

            // Get hash entry
            let entry_hash = self.get_hash_entry(data, index);

            // Check if this is our file
            if (entry_hash & hash_mask) == (hash & hash_mask) {
                // Found potential match, get file index
                let file_index = self.get_file_index(data, index);

                // Check if valid (not empty)
                if file_index != index_mask as u32 {
                    return Some(file_index);
                }
            }

            // Check for empty slot (end of chain)
            if entry_hash == hash_mask {
                break;
            }
        }

        None
    }
}
```

## BET Table (Block Extended Table)

The BET table stores file metadata (position, size, flags) using bit-packed fields for efficiency.

### BET Table Structure

```rust
/// BET table header structure
#[repr(C, packed)]
struct BetTable {
    /// Signature 'BET\x1A' (0x1A544542 in little-endian)
    signature: u32,

    /// Version (always 1)
    version: u32,

    /// Size of the contained table data
    data_size: u32,

    /// Size of the entire table including header
    table_size: u32,

    /// Number of files in BET table
    file_count: u32,

    /// Unknown field (typically 0x10)
    unknown_08: u32,

    /// Size of one table entry in bits
    table_entry_size: u32,

    /// Bit position of file position field
    bit_index_file_pos: u32,

    /// Bit position of file size field
    bit_index_file_size: u32,

    /// Bit position of compressed size field
    bit_index_cmp_size: u32,

    /// Bit position of flag index field
    bit_index_flag_index: u32,

    /// Bit position of unknown field
    bit_index_unknown: u32,

    /// Number of bits for file position
    bit_count_file_pos: u32,

    /// Number of bits for file size
    bit_count_file_size: u32,

    /// Number of bits for compressed size
    bit_count_cmp_size: u32,

    /// Number of bits for flag index
    bit_count_flag_index: u32,

    /// Number of bits for unknown field
    bit_count_unknown: u32,

    /// Hash-related fields
    total_bet_hash_size: u32,
    bet_hash_size_extra: u32,
    bet_hash_size: u32,
    bet_hash_array_size: u32,

    /// Number of flag combinations
    flag_count: u32,
}
```

### BET Table Data Layout

```
[BET Header][Flag Array][File Table (bit-packed)][Hash Array]
```

1. **Flag Array**: Common flag combinations (4 bytes each)
2. **File Table**: Bit-packed file entries
3. **Hash Array**: Name hashes for verification

### BET Entry Bit Layout

Each BET entry is a bit-packed structure containing:

```
[FilePos][FileSize][CmpSize][FlagIndex][Unknown]
 \______/ \______/ \_____/  \______/   \_____/
  N bits   M bits   O bits   P bits    Q bits
```

The exact bit positions and counts are specified in the header.

### BET Bit Manipulation

```rust
/// Generic bit extraction function
fn extract_bits(data: &[u8], bit_offset: usize, bit_count: u32) -> u64 {
    if bit_count == 0 || bit_count > 64 {
        return 0;
    }

    let byte_offset = bit_offset / 8;
    let bit_shift = bit_offset % 8;

    // Read up to 9 bytes to handle up to 64 bits with any alignment
    let mut value: u64 = 0;
    let bytes_needed = ((bit_shift + bit_count as usize + 7) / 8).min(8);

    for i in 0..bytes_needed {
        if byte_offset + i < data.len() {
            value |= (data[byte_offset + i] as u64) << (i * 8);
        }
    }

    // Shift to align and mask to extract exact bits
    value >>= bit_shift;
    value &= (1u64 << bit_count) - 1;

    value
}

/// Insert bits into a byte array
fn insert_bits(data: &mut [u8], bit_offset: usize, bit_count: u32, value: u64) {
    if bit_count == 0 || bit_count > 64 {
        return;
    }

    let byte_offset = bit_offset / 8;
    let bit_shift = bit_offset % 8;

    // Create mask for the bits we're setting
    let mask = (1u64 << bit_count) - 1;
    let masked_value = value & mask;

    // Write the bits
    let bytes_needed = ((bit_shift + bit_count as usize + 7) / 8).min(8);

    for i in 0..bytes_needed {
        if byte_offset + i < data.len() {
            let byte_bits_start = if i == 0 { bit_shift } else { 0 };
            let byte_bits_end = ((bit_shift + bit_count as usize).min((i + 1) * 8)) - i * 8;
            let byte_bits = byte_bits_end - byte_bits_start;

            if byte_bits > 0 {
                let byte_mask = ((1u64 << byte_bits) - 1) << byte_bits_start;
                let byte_value = (masked_value >> (i * 8 - if i == 0 { 0 } else { bit_shift })) << byte_bits_start;

                data[byte_offset + i] &= !(byte_mask as u8);
                data[byte_offset + i] |= byte_value as u8;
            }
        }
    }
}

impl BetTable {
    /// Get file information from BET table
    fn get_file_info(&self, data: &[u8], file_index: u32) -> Option<FileInfo> {
        if file_index >= self.file_count {
            return None;
        }

        // Calculate bit offset for this entry
        let entry_bit_offset = file_index as usize * self.table_entry_size as usize;

        // Extract fields using bit positions and counts
        let file_pos = extract_bits(
            data,
            entry_bit_offset + self.bit_index_file_pos as usize,
            self.bit_count_file_pos
        );

        let file_size = extract_bits(
            data,
            entry_bit_offset + self.bit_index_file_size as usize,
            self.bit_count_file_size
        ) as u32;

        let compressed_size = extract_bits(
            data,
            entry_bit_offset + self.bit_index_cmp_size as usize,
            self.bit_count_cmp_size
        ) as u32;

        let flag_index = extract_bits(
            data,
            entry_bit_offset + self.bit_index_flag_index as usize,
            self.bit_count_flag_index
        ) as u32;

        // Get actual flags from flag array
        let flags = if flag_index < self.flag_count {
            self.get_flags(data, flag_index)
        } else {
            0
        };

        Some(FileInfo {
            file_position: file_pos,
            file_size,
            compressed_size,
            flags,
        })
    }

    /// Get flags from the flag array
    fn get_flags(&self, data: &[u8], flag_index: u32) -> u32 {
        // Flags are stored as 4-byte values after the header
        let flag_offset = std::mem::size_of::<BetTable>() + (flag_index as usize * 4);

        if flag_offset + 4 <= data.len() {
            u32::from_le_bytes([
                data[flag_offset],
                data[flag_offset + 1],
                data[flag_offset + 2],
                data[flag_offset + 3],
            ])
        } else {
            0
        }
    }
}
```

## Complete File Lookup Example

```rust
/// Complete example of finding and reading a file using HET/BET tables
struct MpqV3Reader {
    het_table: HetTable,
    bet_table: BetTable,
    het_data: Vec<u8>,
    bet_data: Vec<u8>,
}

impl MpqV3Reader {
    /// Find a file and get its metadata
    fn find_file(&self, filename: &str) -> Option<FileInfo> {
        // Step 1: Look up file index in HET table
        let file_index = self.het_table.find_file(&self.het_data, filename)?;

        // Step 2: Get file info from BET table using the index
        self.bet_table.get_file_info(&self.bet_data, file_index)
    }

    /// Read a file's data
    fn read_file(&mut self, filename: &str, archive: &mut File) -> Result<Vec<u8>, MpqError> {
        // Get file info
        let info = self.find_file(filename)
            .ok_or(MpqError::FileNotFound)?;

        // Seek to file position
        archive.seek(SeekFrom::Start(info.file_position))?;

        // Read based on compression
        if info.compressed_size < info.file_size {
            // File is compressed
            let mut compressed = vec![0u8; info.compressed_size as usize];
            archive.read_exact(&mut compressed)?;

            // Decompress based on flags
            self.decompress_file(compressed, info.file_size, info.flags)
        } else {
            // File is not compressed
            let mut data = vec![0u8; info.file_size as usize];
            archive.read_exact(&mut data)?;
            Ok(data)
        }
    }
}
```

## Optimization Techniques

### 1. Bit Width Calculation

The bit widths in BET/HET tables are chosen to minimize space:

```rust
/// Calculate minimum bits needed to store a value
fn calculate_bit_width(max_value: u64) -> u32 {
    if max_value == 0 {
        return 1;
    }
    64 - max_value.leading_zeros()
}

/// Example: Calculating BET table bit widths
fn calculate_bet_bit_widths(files: &[FileEntry]) -> BetBitWidths {
    let max_pos = files.iter().map(|f| f.position).max().unwrap_or(0);
    let max_size = files.iter().map(|f| f.size).max().unwrap_or(0);
    let max_cmp = files.iter().map(|f| f.compressed_size).max().unwrap_or(0);

    BetBitWidths {
        file_pos_bits: calculate_bit_width(max_pos),
        file_size_bits: calculate_bit_width(max_size),
        cmp_size_bits: calculate_bit_width(max_cmp),
        // Flag index typically needs fewer bits
        flag_index_bits: 4,  // Supports up to 16 flag combinations
    }
}
```

### 2. Memory-Mapped I/O

For large archives, consider memory-mapping the BET/HET tables:

```rust
use memmap2::Mmap;

struct MpqV3ReaderMapped {
    het_mmap: Mmap,
    bet_mmap: Mmap,
    het_table: HetTable,
    bet_table: BetTable,
}

impl MpqV3ReaderMapped {
    fn new(file: &File, het_offset: u64, bet_offset: u64) -> Result<Self, MpqError> {
        unsafe {
            // Memory map the HET table
            let het_mmap = Mmap::map(file)?;
            let het_table = Self::read_het_header(&het_mmap[het_offset as usize..])?;

            // Memory map the BET table
            let bet_mmap = Mmap::map(file)?;
            let bet_table = Self::read_bet_header(&bet_mmap[bet_offset as usize..])?;

            Ok(Self {
                het_mmap,
                bet_mmap,
                het_table,
                bet_table,
            })
        }
    }
}
```

### 3. Caching Decoded Entries

For frequently accessed files, cache the decoded bit values:

```rust
use std::collections::HashMap;

struct CachedBetTable {
    bet_table: BetTable,
    bet_data: Vec<u8>,
    cache: HashMap<u32, FileInfo>,
}

impl CachedBetTable {
    fn get_file_info(&mut self, file_index: u32) -> Option<&FileInfo> {
        if !self.cache.contains_key(&file_index) {
            if let Some(info) = self.bet_table.get_file_info(&self.bet_data, file_index) {
                self.cache.insert(file_index, info);
            }
        }
        self.cache.get(&file_index)
    }
}
```

## Common Pitfalls and Solutions

### 1. Bit Alignment Issues

Always remember that bit offsets don't align with byte boundaries:

```rust
// WRONG: Assuming byte alignment
let byte_offset = bit_offset / 8;
let value = data[byte_offset];  // This misses bits!

// CORRECT: Handle bit alignment
let value = extract_bits(data, bit_offset, bit_count);
```

### 2. Endianness

BET/HET data is little-endian, but bit operations are typically big-endian within bytes:

```rust
// Bits within a byte: 76543210 (MSB first)
// Bytes in multi-byte values: Little-endian
```

### 3. Hash Collisions

Both HET and traditional hash tables use linear probing:

```rust
// Keep searching until:
// 1. Found matching hash
// 2. Found empty slot (0xFFFFFFFF or hash_mask)
// 3. Wrapped around entire table
```

## Performance Considerations

1. **Bit Extraction Overhead**: Consider SIMD operations for bulk extraction
2. **Cache Locality**: Process files in order when possible
3. **Memory Usage**: BET/HET tables are more memory-efficient than traditional tables
4. **Compression**: Tables can be compressed - check header flags

## Debugging Tips

### Hex Dump Analyzer

```rust
/// Debug function to visualize bit-packed data
fn dump_bet_entry(data: &[u8], entry_index: u32, bet: &BetTable) {
    let bit_offset = entry_index as usize * bet.table_entry_size as usize;

    println!("BET Entry {} (bit offset: {}):", entry_index, bit_offset);
    println!("  File Pos:  {:016b} ({})",
        extract_bits(data, bit_offset + bet.bit_index_file_pos as usize, bet.bit_count_file_pos),
        extract_bits(data, bit_offset + bet.bit_index_file_pos as usize, bet.bit_count_file_pos)
    );
    println!("  File Size: {:016b} ({})",
        extract_bits(data, bit_offset + bet.bit_index_file_size as usize, bet.bit_count_file_size),
        extract_bits(data, bit_offset + bet.bit_index_file_size as usize, bet.bit_count_file_size)
    );
    // ... continue for other fields
}
```

## References

1. StormLib source code - Reference implementation (partial BET/HET support)
2. MPQ format documentation by Ladislav Zezula
3. WoW Dev Wiki - Additional format details
4. The MoPaQ Archive Format specification
