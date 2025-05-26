#!/usr/bin/env python3
"""
Generate comprehensive test MPQ archives for all versions (v1-v4).

These archives include all features from the MPQ format specification
to enable thorough testing of the mopaq implementation.

Author: Daniel S. Reichenbach <daniel@kogito.network>
"""

import struct
import hashlib
import zlib
import bz2
import os
from pathlib import Path
from typing import List, Tuple, Dict, Optional
import json


class MPQCrypto:
    """MPQ encryption/decryption implementation."""

    def __init__(self):
        self.encryption_table = self._generate_encryption_table()

    def _generate_encryption_table(self) -> List[int]:
        """Generate the MPQ encryption table."""
        table = [0] * 0x500
        seed = 0x00100001

        for index1 in range(0x100):
            for index2 in range(5):
                table_index = index1 + index2 * 0x100
                seed = (seed * 125 + 3) % 0x2AAAAB
                temp1 = (seed & 0xFFFF) << 0x10
                seed = (seed * 125 + 3) % 0x2AAAAB
                temp2 = seed & 0xFFFF
                table[table_index] = temp1 | temp2

        return table

    def hash_string(self, filename: str, hash_type: int) -> int:
        """Hash a filename using MPQ algorithm."""
        seed1 = 0x7FED7FED
        seed2 = 0xEEEEEEEE

        for ch in filename.upper().encode('ascii'):
            if ch == ord('/'):
                ch = ord('\\')

            table_idx = (hash_type * 0x100 + ch)
            seed1 = self.encryption_table[table_idx] ^ (seed1 + seed2)
            seed1 &= 0xFFFFFFFF
            seed2 = ch + seed1 + seed2 + (seed2 << 5) + 3
            seed2 &= 0xFFFFFFFF

        return seed1

    def encrypt_block(self, data: bytearray, key: int) -> None:
        """Encrypt data in-place."""
        if key == 0:
            return

        seed = 0xEEEEEEEE

        # Process as 32-bit integers
        for i in range(0, len(data), 4):
            if i + 4 <= len(data):
                # Update seed
                seed = (seed + self.encryption_table[0x400 + (key & 0xFF)]) & 0xFFFFFFFF

                # Get current dword
                dword = struct.unpack_from('<I', data, i)[0]

                # Encrypt
                encrypted = dword ^ ((key + seed) & 0xFFFFFFFF)
                struct.pack_into('<I', data, i, encrypted)

                # Update key
                key = (((~key << 0x15) + 0x11111111) | (key >> 0x0B)) & 0xFFFFFFFF

                # Update seed
                seed = (dword + seed + (seed << 5) + 3) & 0xFFFFFFFF


class MPQArchiveBuilder:
    """Build MPQ archives with comprehensive feature coverage."""

    def __init__(self, version: int):
        self.version = version
        self.crypto = MPQCrypto()
        self.files = []
        self.hash_table_size = 16  # Must be power of 2
        self.block_size = 3  # 4096 byte sectors

    def add_file(self, name: str, data: bytes,
                 compress: bool = True,
                 encrypt: bool = False,
                 fix_key: bool = False,
                 single_unit: bool = False,
                 sector_crc: bool = False,
                 locale: int = 0,
                 compression_type: str = 'zlib') -> None:
        """Add a file to the archive."""
        self.files.append({
            'name': name,
            'data': data,
            'compress': compress,
            'encrypt': encrypt,
            'fix_key': fix_key,
            'single_unit': single_unit,
            'sector_crc': sector_crc,
            'locale': locale,
            'compression_type': compression_type
        })

    def _compress_data(self, data: bytes, method: str) -> bytes:
        """Compress data using specified method."""
        if method == 'zlib':
            return b'\x02' + zlib.compress(data)
        elif method == 'bzip2':
            return b'\x10' + bz2.compress(data)
        elif method == 'sparse':
            # Simple sparse compression
            result = bytearray()
            i = 0
            while i < len(data):
                # Count non-zero bytes
                start = i
                while i < len(data) and data[i] != 0:
                    i += 1

                if i > start:
                    count = min(i - start, 127)
                    result.append(count)
                    result.extend(data[start:start + count])

                # Count zeros
                start = i
                while i < len(data) and data[i] == 0:
                    i += 1

                if i > start:
                    count = min(i - start, 127)
                    result.append(0x80 | count)

            result.append(0xFF)  # End marker
            return b'\x20' + bytes(result)
        else:
            return data

    def _create_sector_data(self, file_info: dict, file_key: int) -> Tuple[bytearray, List[int], List[int]]:
        """Create sectored file data with optional compression and CRC."""
        sector_size = 512 << self.block_size
        data = file_info['data']

        sectors = []
        sector_crcs = []
        sector_offsets = [0]

        # Split into sectors
        for i in range(0, len(data), sector_size):
            sector_data = data[i:i + sector_size]

            # Calculate CRC if needed (on uncompressed data)
            if file_info['sector_crc']:
                crc = zlib.crc32(sector_data) & 0xFFFFFFFF
                sector_crcs.append(crc)

            # Compress sector if needed
            if file_info['compress'] and len(sector_data) > 1:
                compressed = self._compress_data(sector_data, file_info['compression_type'])
                if len(compressed) < len(sector_data):
                    sector_data = compressed

            sectors.append(sector_data)
            sector_offsets.append(sector_offsets[-1] + len(sector_data))

        # Build final data
        result = bytearray()

        # Sector offset table
        offset_data = bytearray()
        for offset in sector_offsets:
            offset_data.extend(struct.pack('<I', offset))

        # Encrypt sector offset table with key-1
        if file_info['encrypt'] and file_key > 0:
            offset_key = (file_key - 1) & 0xFFFFFFFF
            self.crypto.encrypt_block(offset_data, offset_key)

        result.extend(offset_data)

        # CRC table if present (not encrypted)
        if file_info['sector_crc']:
            for crc in sector_crcs:
                result.extend(struct.pack('<I', crc))

        # Encrypt each sector individually
        for i, sector in enumerate(sectors):
            sector_data = bytearray(sector)
            if file_info['encrypt']:
                sector_key = (file_key + i) & 0xFFFFFFFF
                self.crypto.encrypt_block(sector_data, sector_key)
            result.extend(sector_data)

        return result, sector_offsets, sector_crcs

    def build(self, output_path: str) -> None:
        """Build the MPQ archive."""
        with open(output_path, 'wb') as f:
            # Reserve space for header
            header_size = {0: 32, 1: 44, 2: 68, 3: 208}[self.version]
            f.write(b'\x00' * header_size)

            # Process files and build tables
            hash_entries = []
            block_entries = []
            file_data_offset = 0x1000  # Start files at 4KB

            # Add files
            for file_info in self.files:
                # Calculate hash values
                hash_a = self.crypto.hash_string(file_info['name'], 1)
                hash_b = self.crypto.hash_string(file_info['name'], 2)
                hash_offset = self.crypto.hash_string(file_info['name'], 0)

                # Prepare file data
                if file_info['single_unit']:
                    # Single unit file
                    file_data = bytearray(file_info['data'])

                    if file_info['compress']:
                        compressed = self._compress_data(file_info['data'],
                                                        file_info['compression_type'])
                        file_data = bytearray(compressed)

                    if file_info['sector_crc']:
                        # Add CRC at end
                        crc = zlib.crc32(file_info['data']) & 0xFFFFFFFF
                        file_data.extend(struct.pack('<I', crc))

                    # Encrypt single unit files
                    if file_info['encrypt']:
                        key = self.crypto.hash_string(file_info['name'], 3)
                        if file_info['fix_key']:
                            key = (key + file_data_offset) ^ len(file_info['data'])
                        self.crypto.encrypt_block(file_data, key)

                    compressed_size = len(file_data) - (4 if file_info['sector_crc'] else 0)
                else:
                    # Multi-sector file - pass the key to sector creation
                    key = 0
                    if file_info['encrypt']:
                        key = self.crypto.hash_string(file_info['name'], 3)
                        if file_info['fix_key']:
                            key = (key + file_data_offset) ^ len(file_info['data'])

                    file_data, _, _ = self._create_sector_data(file_info, key)
                    compressed_size = len(file_data)
                    # Don't encrypt here - already encrypted per-sector

                # Encrypt if needed
                if file_info['encrypt']:
                    key = self.crypto.hash_string(file_info['name'], 3)
                    if file_info['fix_key']:
                        key = (key + file_data_offset) ^ len(file_info['data'])
                    self.crypto.encrypt_block(file_data, key)

                # Write file data
                f.seek(file_data_offset)
                f.write(file_data)

                # Create block entry
                flags = 0x80000000  # EXISTS
                if file_info['compress']:
                    flags |= 0x00000200
                if file_info['encrypt']:
                    flags |= 0x00010000
                if file_info['fix_key']:
                    flags |= 0x00020000
                if file_info['single_unit']:
                    flags |= 0x01000000
                if file_info['sector_crc']:
                    flags |= 0x04000000

                block_entries.append({
                    'file_pos': file_data_offset,
                    'compressed_size': compressed_size,
                    'file_size': len(file_info['data']),
                    'flags': flags
                })

                # Create hash entry
                hash_idx = hash_offset % self.hash_table_size
                hash_entries.append({
                    'name_1': hash_a,
                    'name_2': hash_b,
                    'locale': file_info['locale'],
                    'platform': 0,
                    'block_index': len(block_entries) - 1,
                    'preferred_index': hash_idx
                })

                file_data_offset += len(file_data)
                # Align to 512 bytes
                file_data_offset = (file_data_offset + 511) & ~511

            # Write hash table
            hash_table_offset = file_data_offset
            f.seek(hash_table_offset)

            hash_table = [None] * self.hash_table_size
            for entry in hash_entries:
                # Linear probing for collisions
                idx = entry['preferred_index']
                while hash_table[idx] is not None:
                    idx = (idx + 1) % self.hash_table_size
                hash_table[idx] = entry

            # Write hash entries
            hash_data = bytearray()
            for i in range(self.hash_table_size):
                if hash_table[i]:
                    entry = hash_table[i]
                    hash_data.extend(struct.pack('<II', entry['name_1'], entry['name_2']))
                    hash_data.extend(struct.pack('<HH', entry['locale'], entry['platform']))
                    hash_data.extend(struct.pack('<I', entry['block_index']))
                else:
                    # Empty entry
                    hash_data.extend(struct.pack('<IIHHI', 0, 0, 0, 0, 0xFFFFFFFF))

            # Encrypt hash table
            key = self.crypto.hash_string("(hash table)", 3)
            self.crypto.encrypt_block(hash_data, key)
            f.write(hash_data)

            # Write block table
            block_table_offset = f.tell()
            block_data = bytearray()

            for entry in block_entries:
                block_data.extend(struct.pack('<I', entry['file_pos']))
                block_data.extend(struct.pack('<I', entry['compressed_size']))
                block_data.extend(struct.pack('<I', entry['file_size']))
                block_data.extend(struct.pack('<I', entry['flags']))

            # Encrypt block table
            key = self.crypto.hash_string("(block table)", 3)
            self.crypto.encrypt_block(block_data, key)
            f.write(block_data)

            # Write header
            archive_size = f.tell()
            f.seek(0)

            # Basic header (v1)
            f.write(struct.pack('<I', 0x1A51504D))  # Signature
            f.write(struct.pack('<I', header_size))
            f.write(struct.pack('<I', archive_size))
            f.write(struct.pack('<H', self.version))
            f.write(struct.pack('<H', self.block_size))
            f.write(struct.pack('<I', hash_table_offset))
            f.write(struct.pack('<I', block_table_offset))
            f.write(struct.pack('<I', self.hash_table_size))
            f.write(struct.pack('<I', len(block_entries)))

            if self.version >= 1:
                # v2 fields
                f.write(struct.pack('<Q', 0))  # Hi-block table position
                f.write(struct.pack('<H', 0))  # Hash table pos high
                f.write(struct.pack('<H', 0))  # Block table pos high

            if self.version >= 2:
                # v3 fields
                f.write(struct.pack('<Q', archive_size))  # 64-bit archive size
                f.write(struct.pack('<Q', 0))  # BET table position
                f.write(struct.pack('<Q', 0))  # HET table position

            if self.version >= 3:
                # v4 fields - compressed sizes
                f.write(struct.pack('<Q', len(hash_data)))
                f.write(struct.pack('<Q', len(block_data)))
                f.write(struct.pack('<Q', 0))  # Hi-block size
                f.write(struct.pack('<Q', 0))  # HET size
                f.write(struct.pack('<Q', 0))  # BET size
                f.write(struct.pack('<I', 0x2000))  # Raw chunk size

                # MD5 hashes
                for _ in range(6):
                    f.write(hashlib.md5(b'dummy').digest())


def create_test_archives():
    """Create comprehensive test archives for all MPQ versions."""
    os.makedirs('test-data/comprehensive', exist_ok=True)

    # Test data
    test_files = {
        'readme.txt': b'This is a simple text file for testing.',
        'data/binary.dat': bytes(range(256)) * 16,  # 4KB of binary data
        'scripts/test.lua': b'-- Lua script\nprint("Hello from MPQ!")\n',
        'images/test.blp': b'BLP2' + b'\x00' * 148,  # Minimal BLP header
        'sounds/test.wav': b'RIFF' + b'\x00' * 40 + b'data' + b'\x00' * 1000,
        'large_file.bin': b'X' * 10000,  # Multi-sector file
        'zero_file.dat': b'\x00' * 5000,  # For sparse compression
        'unicode_test.txt': 'Test unicode: café, Москва, 東京'.encode('utf-8'),
    }

    # Version-specific features
    for version in range(4):
        print(f"\nCreating v{version + 1} archive...")
        builder = MPQArchiveBuilder(version)

        # Add basic files
        builder.add_file('readme.txt', test_files['readme.txt'],
                        compress=False, encrypt=False)

        # Compressed file
        builder.add_file('data/binary.dat', test_files['data/binary.dat'],
                        compress=True, compression_type='zlib')

        # Encrypted file
        builder.add_file('scripts/test.lua', test_files['scripts/test.lua'],
                        compress=True, encrypt=True)

        # Fixed key encryption
        builder.add_file('images/test.blp', test_files['images/test.blp'],
                        compress=False, encrypt=True, fix_key=True)

        # Single unit with CRC
        builder.add_file('sounds/test.wav', test_files['sounds/test.wav'],
                        compress=True, single_unit=True, sector_crc=True)

        # Multi-sector file
        builder.add_file('large_file.bin', test_files['large_file.bin'],
                        compress=True, sector_crc=True)

        if version >= 1:  # v2+
            # BZip2 compression
            builder.add_file('zero_file.dat', test_files['zero_file.dat'],
                            compress=True, compression_type='bzip2')

        if version >= 2:  # v3+
            # Sparse compression
            builder.add_file('sparse_test.dat', b'\x00' * 1000 + b'DATA' + b'\x00' * 1000,
                            compress=True, compression_type='sparse')

        # Different locales
        builder.add_file('locale_test.txt', b'English version', locale=0x0409)
        builder.add_file('locale_test.txt', b'German version', locale=0x0407)

        # Special files
        listfile_content = '\n'.join(f['name'] for f in builder.files)
        builder.add_file('(listfile)', listfile_content.encode('ascii'))

        # Create attributes file
        attributes = []
        for f in builder.files:
            attrs = {
                'name': f['name'],
                'crc32': zlib.crc32(f['data']) & 0xFFFFFFFF,
                'timestamp': 0,
                'md5': hashlib.md5(f['data']).hexdigest()
            }
            attributes.append(attrs)

        attr_data = json.dumps(attributes, indent=2).encode('utf-8')
        builder.add_file('(attributes)', attr_data, compress=True)

        # Build archive
        output_file = f'test-data/comprehensive/test_v{version + 1}.mpq'
        builder.build(output_file)
        print(f"Created: {output_file}")
        print(f"  Files: {len(builder.files)}")
        print(f"  Features tested:")
        print(f"    - Compression: zlib", end='')
        if version >= 1:
            print(", bzip2", end='')
        if version >= 2:
            print(", sparse", end='')
        print()
        print(f"    - Encryption: standard, fixed key")
        print(f"    - Sector CRC validation")
        print(f"    - Multiple locales")
        print(f"    - Special files: (listfile), (attributes)")


def create_edge_case_archives():
    """Create archives that test edge cases and error conditions."""
    print("\nCreating edge case test archives...")

    # 1. Hash table collision test
    builder = MPQArchiveBuilder(0)
    builder.hash_table_size = 4  # Very small to force collisions

    # These files will collide in a small hash table
    for i in range(10):
        builder.add_file(f'collision_{i}.txt', f'File {i}'.encode())

    builder.build('test-data/comprehensive/hash_collisions.mpq')
    print("Created: hash_collisions.mpq (tests hash table collision handling)")

    # 2. Large file test (>4GB for v2+)
    # Note: This would create a very large file, so we'll simulate it
    print("Note: Skipping 4GB+ file test (would be too large)")

    # 3. Empty archive
    builder = MPQArchiveBuilder(0)
    builder.build('test-data/comprehensive/empty.mpq')
    print("Created: empty.mpq (archive with no files)")

    # 4. Maximum hash table size
    builder = MPQArchiveBuilder(0)
    builder.hash_table_size = 65536  # Large hash table
    builder.add_file('single.txt', b'One file in huge table')
    builder.build('test-data/comprehensive/large_hashtable.mpq')
    print("Created: large_hashtable.mpq (tests sparse hash table)")


def main():
    """Generate all test archives."""
    print("MPQ Test Archive Generator")
    print("=" * 50)

    create_test_archives()
    create_edge_case_archives()

    print("\nTest archives created successfully!")
    print("\nYou can now test with:")
    print("  cargo test")
    print("  cargo run --bin storm-cli -- list test-data/comprehensive/test_v1.mpq")
    print("  cargo run --bin storm-cli -- verify test-data/comprehensive/test_v2.mpq")
    print("  cargo run --bin storm-cli -- extract test-data/comprehensive/test_v3.mpq")
    print("  cargo run --bin storm-cli -- debug info test-data/comprehensive/test_v4.mpq")

if __name__ == "__main__":
    main()
