#!/usr/bin/env python3
"""
Create test MPQ files with sector CRCs for testing CRC validation.

This creates MPQ files with properly formatted CRC tables.

Author: Daniel S. Reichenbach <daniel@kogito.network>
"""

import struct
import zlib
import os


def calculate_crc32(data):
    """Calculate CRC-32 checksum."""
    return zlib.crc32(data) & 0xFFFFFFFF


def hash_string(filename, hash_type):
    """MPQ hash function implementation."""
    # Simplified encryption table (just first few values for testing)
    encryption_table = [0] * 0x500
    seed = 0x00100001

    for index1 in range(0x100):
        for index2 in range(5):
            table_index = index1 + index2 * 0x100
            seed = (seed * 125 + 3) % 0x2AAAAB
            temp1 = (seed & 0xFFFF) << 0x10
            seed = (seed * 125 + 3) % 0x2AAAAB
            temp2 = seed & 0xFFFF
            encryption_table[table_index] = temp1 | temp2

    # Hash the string
    seed1 = 0x7FED7FED
    seed2 = 0xEEEEEEEE

    for ch in filename.upper().encode('ascii'):
        if ch == ord('/'):
            ch = ord('\\')

        table_idx = (hash_type * 0x100 + ch)
        seed1 = encryption_table[table_idx] ^ (seed1 + seed2)
        seed2 = ch + seed1 + seed2 + (seed2 << 5) + 3
        seed2 &= 0xFFFFFFFF

    return seed1 & 0xFFFFFFFF


def create_mpq_with_crc(filename):
    """Create a test MPQ with sector CRCs."""
    with open(filename, 'wb') as f:
        # MPQ Header (v1 - 32 bytes)
        f.write(struct.pack('<I', 0x1A51504D))  # Signature 'MPQ\x1A'
        f.write(struct.pack('<I', 32))          # Header size
        f.write(struct.pack('<I', 0x4000))      # Archive size (16KB)
        f.write(struct.pack('<H', 0))           # Format version (v1)
        f.write(struct.pack('<H', 2))           # Block size (2048 bytes sectors)
        f.write(struct.pack('<I', 0x200))       # Hash table position
        f.write(struct.pack('<I', 0x400))       # Block table position
        f.write(struct.pack('<I', 4))           # Hash table entries
        f.write(struct.pack('<I', 1))           # Block table entries

        # Test file data
        test_filename = "test_crc.txt"
        # Create content that spans multiple sectors
        sector_size = 512 << 2  # 2048 bytes
        test_content = b"This is test data for CRC validation. " * 100  # ~3900 bytes

        # Split into sectors
        sectors = []
        pos = 0
        while pos < len(test_content):
            sector_data = test_content[pos:pos + sector_size]
            sectors.append(sector_data)
            pos += sector_size

        # Compress each sector and calculate CRCs
        compressed_sectors = []
        sector_crcs = []
        sector_offsets = [0]

        for sector in sectors:
            # Calculate CRC on uncompressed data
            crc = calculate_crc32(sector)
            sector_crcs.append(crc)

            # Compress sector
            compressed = bytearray([0x02])  # zlib compression
            compressed.extend(zlib.compress(sector))
            compressed_sectors.append(compressed)

            # Track offset
            sector_offsets.append(sector_offsets[-1] + len(compressed))

        # File data starts at 0x800
        file_offset = 0x800
        f.seek(file_offset)

        # Write sector offset table
        for offset in sector_offsets:
            f.write(struct.pack('<I', offset))

        # Write CRC table
        for crc in sector_crcs:
            f.write(struct.pack('<I', crc))

        # Write compressed sectors
        for compressed in compressed_sectors:
            f.write(compressed)

        # Calculate total compressed size
        offset_table_size = len(sector_offsets) * 4
        crc_table_size = len(sector_crcs) * 4
        data_size = sum(len(s) for s in compressed_sectors)
        total_compressed_size = offset_table_size + crc_table_size + data_size

        # Hash table at 0x200
        f.seek(0x200)
        hash_a = hash_string(test_filename, 1)
        hash_b = hash_string(test_filename, 2)
        hash_offset = hash_string(test_filename, 0)

        # Write hash entry
        f.write(struct.pack('<II', hash_a, hash_b))
        f.write(struct.pack('<HHI', 0, 0, 0))  # locale, platform, block index

        # Fill rest of hash table
        for _ in range(3):
            f.write(struct.pack('<IIHHI', 0, 0, 0, 0, 0xFFFFFFFF))

        # Block table at 0x400
        f.seek(0x400)
        flags = 0x80000200 | 0x04000000  # EXISTS | COMPRESS | SECTOR_CRC
        f.write(struct.pack('<I', file_offset))           # File position
        f.write(struct.pack('<I', total_compressed_size)) # Compressed size
        f.write(struct.pack('<I', len(test_content)))     # Uncompressed size
        f.write(struct.pack('<I', flags))                 # Flags

    print(f"Created {filename}")
    print(f"  Contains: {test_filename}")
    print(f"  Sectors: {len(sectors)}")
    print(f"  Sector CRCs: {[f'0x{crc:08X}' for crc in sector_crcs]}")
    print(f"  Flags: COMPRESSED | SECTOR_CRC")


def create_single_unit_crc_mpq(filename):
    """Create a test MPQ with single unit file and CRC."""
    with open(filename, 'wb') as f:
        # MPQ Header
        f.write(struct.pack('<I', 0x1A51504D))  # Signature
        f.write(struct.pack('<I', 32))          # Header size
        f.write(struct.pack('<I', 0x2000))      # Archive size
        f.write(struct.pack('<H', 0))           # Format version
        f.write(struct.pack('<H', 3))           # Block size
        f.write(struct.pack('<I', 0x200))       # Hash table position
        f.write(struct.pack('<I', 0x400))       # Block table position
        f.write(struct.pack('<I', 4))           # Hash table entries
        f.write(struct.pack('<I', 1))           # Block table entries

        # Test file
        test_filename = "single_crc.txt"
        test_content = b"This is a single unit file with CRC validation."

        # Calculate CRC
        crc = calculate_crc32(test_content)

        # Compress
        compressed = bytearray([0x02])  # zlib
        compressed.extend(zlib.compress(test_content))

        # File at 0x600
        file_offset = 0x600
        f.seek(file_offset)
        f.write(compressed)
        f.write(struct.pack('<I', crc))  # CRC follows data

        # Hash table
        f.seek(0x200)
        hash_a = hash_string(test_filename, 1)
        hash_b = hash_string(test_filename, 2)

        f.write(struct.pack('<IIHHI', hash_a, hash_b, 0, 0, 0))
        for _ in range(3):
            f.write(struct.pack('<IIHHI', 0, 0, 0, 0, 0xFFFFFFFF))

        # Block table
        f.seek(0x400)
        flags = 0x80000200 | 0x01000000 | 0x04000000  # EXISTS | COMPRESS | SINGLE_UNIT | SECTOR_CRC
        f.write(struct.pack('<I', file_offset))
        f.write(struct.pack('<I', len(compressed)))
        f.write(struct.pack('<I', len(test_content)))
        f.write(struct.pack('<I', flags))

    print(f"\nCreated {filename}")
    print(f"  Contains: {test_filename}")
    print(f"  CRC: 0x{crc:08X}")
    print(f"  Flags: COMPRESSED | SINGLE_UNIT | SECTOR_CRC")


def main():
    """Create test MPQ files with CRCs."""
    os.makedirs('test-data/crc', exist_ok=True)

    # Create multi-sector file with CRCs
    create_mpq_with_crc('test-data/crc/sectors.mpq')

    # Create single unit file with CRC
    create_single_unit_crc_mpq('test-data/crc/single.mpq')

    print("\nCRC test MPQ files created!")
    print("\nYou can test CRC validation with:")
    print("  cargo run --bin storm-cli -- extract test-data/crc/sectors.mpq -f test_crc.txt")
    print("  cargo run --bin storm-cli -- extract test-data/crc/single.mpq -f single_crc.txt")
    print("  cargo run --bin storm-cli -- verify test-data/crc/sectors.mpq")


if __name__ == "__main__":
    main()
