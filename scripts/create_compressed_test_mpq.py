#!/usr/bin/env python3
"""
Create test MPQ files with compressed data for testing decompression.

This creates MPQ files with properly formatted hash and block tables
and compressed file data.

Author: Daniel S. Reichenbach <daniel@kogito.network>
"""

import struct
import zlib
import os
from pathlib import Path


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


def encrypt_data(data, key):
    """Encrypt data using MPQ encryption."""
    # Simplified - just XOR with key for testing
    encrypted = bytearray()
    key_bytes = struct.pack('<I', key)

    for i, byte in enumerate(data):
        encrypted.append(byte ^ key_bytes[i % 4])

    return bytes(encrypted)


def create_test_mpq_with_compressed_file(filename):
    """Create a test MPQ with a compressed file."""
    with open(filename, 'wb') as f:
        # MPQ Header (v1 - 32 bytes)
        header_offset = 0
        f.write(struct.pack('<I', 0x1A51504D))  # Signature 'MPQ\x1A'
        f.write(struct.pack('<I', 32))          # Header size
        f.write(struct.pack('<I', 0x2000))      # Archive size (8KB)
        f.write(struct.pack('<H', 0))           # Format version (v1)
        f.write(struct.pack('<H', 3))           # Block size (4096 bytes)
        f.write(struct.pack('<I', 0x200))       # Hash table position
        f.write(struct.pack('<I', 0x400))       # Block table position
        f.write(struct.pack('<I', 4))           # Hash table entries
        f.write(struct.pack('<I', 2))           # Block table entries

        # Test file data
        test_filename = "test.txt"
        test_content = b"Hello, World! This is a test file in an MPQ archive. " * 10

        # Compress the data with zlib
        compressed_data = bytearray([0x02])  # Compression type: zlib
        compressed_data.extend(zlib.compress(test_content))

        # File data starts at 0x600
        file_offset = 0x600
        f.seek(file_offset)
        f.write(compressed_data)

        # Hash table at 0x200
        f.seek(0x200)

        # Calculate hashes for the test file
        hash_a = hash_string(test_filename, 1)
        hash_b = hash_string(test_filename, 2)
        hash_offset = hash_string(test_filename, 0)

        # Hash table entries (4 entries, but only first is used)
        hash_entry_index = hash_offset & 3  # Table size is 4

        # Write empty entries before our file
        for i in range(hash_entry_index):
            f.write(struct.pack('<IIHHI', 0, 0, 0, 0, 0xFFFFFFFF))

        # Write our file's hash entry
        f.write(struct.pack('<II', hash_a, hash_b))
        f.write(struct.pack('<H', 0))      # Locale (neutral)
        f.write(struct.pack('<H', 0))      # Platform (default)
        f.write(struct.pack('<I', 0))      # Block index 0

        # Fill remaining hash entries
        for i in range(hash_entry_index + 1, 4):
            f.write(struct.pack('<IIHHI', 0, 0, 0, 0, 0xFFFFFFFF))

        # Block table at 0x400
        f.seek(0x400)

        # Block entry for our file
        f.write(struct.pack('<I', file_offset))           # File position
        f.write(struct.pack('<I', len(compressed_data)))  # Compressed size
        f.write(struct.pack('<I', len(test_content)))     # Uncompressed size
        f.write(struct.pack('<I', 0x80000200))            # Flags: EXISTS | COMPRESS

        # Add a (listfile) entry
        listfile_content = test_filename.encode('ascii') + b'\n'
        listfile_offset = file_offset + len(compressed_data) + 16  # Align to 16

        f.seek(listfile_offset)
        f.write(listfile_content)

        # Block entry for (listfile)
        f.seek(0x400 + 16)  # Second block entry
        f.write(struct.pack('<I', listfile_offset))       # File position
        f.write(struct.pack('<I', len(listfile_content))) # Compressed size
        f.write(struct.pack('<I', len(listfile_content))) # Uncompressed size
        f.write(struct.pack('<I', 0x80000000))            # Flags: EXISTS only

        # Pad to archive size
        f.seek(0x2000 - 1)
        f.write(b'\x00')

    print(f"Created {filename}")
    print(f"  Contains: test.txt (compressed with zlib)")
    print(f"  Contains: (listfile)")


def create_multi_file_mpq(filename):
    """Create an MPQ with multiple files of different types."""
    with open(filename, 'wb') as f:
        # MPQ Header (v1 - 32 bytes)
        f.write(struct.pack('<I', 0x1A51504D))  # Signature
        f.write(struct.pack('<I', 32))          # Header size
        f.write(struct.pack('<I', 0x4000))      # Archive size (16KB)
        f.write(struct.pack('<H', 0))           # Format version
        f.write(struct.pack('<H', 3))           # Block size
        f.write(struct.pack('<I', 0x200))       # Hash table position
        f.write(struct.pack('<I', 0x600))       # Block table position
        f.write(struct.pack('<I', 16))          # Hash table entries
        f.write(struct.pack('<I', 4))           # Block table entries

        # Files to add
        files = [
            ("readme.txt", b"This is a simple readme file.", False),
            ("data.bin", bytes(range(256)) * 4, True),  # Binary data, compressed
            ("script.lua", b"-- Lua script\nprint('Hello from MPQ!')\n", False),
        ]

        # File data starts at 0x1000
        current_offset = 0x1000
        file_entries = []

        for filename, content, compress in files:
            f.seek(current_offset)

            if compress:
                # Compress with zlib
                file_data = bytearray([0x02])  # zlib compression
                file_data.extend(zlib.compress(content))
                flags = 0x80000200  # EXISTS | COMPRESS
            else:
                file_data = content
                flags = 0x80000000  # EXISTS only

            f.write(file_data)

            file_entries.append({
                'name': filename,
                'offset': current_offset,
                'compressed_size': len(file_data),
                'uncompressed_size': len(content),
                'flags': flags,
                'hash_a': hash_string(filename, 1),
                'hash_b': hash_string(filename, 2),
                'hash_offset': hash_string(filename, 0)
            })

            current_offset += (len(file_data) + 15) & ~15  # Align to 16

        # Add (listfile)
        listfile_content = '\n'.join(entry['name'] for entry in file_entries).encode('ascii')
        f.seek(current_offset)
        f.write(listfile_content)

        file_entries.append({
            'name': '(listfile)',
            'offset': current_offset,
            'compressed_size': len(listfile_content),
            'uncompressed_size': len(listfile_content),
            'flags': 0x80000000,
            'hash_a': hash_string('(listfile)', 1),
            'hash_b': hash_string('(listfile)', 2),
            'hash_offset': hash_string('(listfile)', 0)
        })

        # Write hash table
        f.seek(0x200)
        hash_table = [None] * 16

        for i, entry in enumerate(file_entries):
            index = entry['hash_offset'] & 15
            # Linear probing for collisions
            while hash_table[index] is not None:
                index = (index + 1) & 15
            hash_table[index] = (entry, i)

        for slot in hash_table:
            if slot is None:
                # Empty entry
                f.write(struct.pack('<IIHHI', 0, 0, 0, 0, 0xFFFFFFFF))
            else:
                entry, block_index = slot
                f.write(struct.pack('<II', entry['hash_a'], entry['hash_b']))
                f.write(struct.pack('<HHI', 0, 0, block_index))

        # Write block table
        f.seek(0x600)
        for entry in file_entries:
            f.write(struct.pack('<I', entry['offset']))
            f.write(struct.pack('<I', entry['compressed_size']))
            f.write(struct.pack('<I', entry['uncompressed_size']))
            f.write(struct.pack('<I', entry['flags']))

    print(f"\nCreated {filename}")
    print("  Contains:")
    for entry in file_entries:
        compressed = " (compressed)" if entry['flags'] & 0x200 else ""
        print(f"    - {entry['name']}{compressed}")


def main():
    """Create test MPQ files."""
    os.makedirs('test-data/compressed', exist_ok=True)

    # Create simple compressed MPQ
    create_test_mpq_with_compressed_file('test-data/compressed/simple.mpq')

    # Create multi-file MPQ
    create_multi_file_mpq('test-data/compressed/multi.mpq')

    print("\nTest MPQ files created!")
    print("\nYou can now test extraction:")
    print("  cargo run --bin storm-cli -- extract test-data/compressed/simple.mpq -o output/")
    print("  cargo run --bin storm-cli -- extract test-data/compressed/multi.mpq -o output/")
    print("  cargo run --bin storm-cli -- extract test-data/compressed/simple.mpq -f test.txt")


if __name__ == "__main__":
    main()
