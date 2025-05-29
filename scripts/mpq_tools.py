#!/usr/bin/env python3
"""
Unified MPQ test data generator and utilities.

This script combines all MPQ-related test generation and verification tools.

Usage:
    python3 mpq_tools.py create minimal --version 1
    python3 mpq_tools.py create compressed --compression zlib
    python3 mpq_tools.py create comprehensive --all-versions
    python3 mpq_tools.py verify encryption-table
    python3 mpq_tools.py info archive.mpq

Author: Daniel S. Reichenbach <daniel@kogito.network>
"""

import argparse
import struct
import hashlib
import zlib
import bz2
import os
import json
import time
from pathlib import Path
from typing import List, Tuple, Dict, Optional, BinaryIO, Any
from dataclasses import dataclass, field
from enum import IntEnum, IntFlag


class MPQVersion(IntEnum):
    """MPQ format versions."""
    V1 = 0  # Original (32-byte header)
    V2 = 1  # Burning Crusade (44-byte header)
    V3 = 2  # Cataclysm Beta (68-byte header)
    V4 = 3  # Cataclysm+ (208-byte header)


class MPQFlags(IntFlag):
    """Block table flags."""
    EXISTS = 0x80000000
    IMPLODE = 0x00000100
    COMPRESS = 0x00000200
    ENCRYPTED = 0x00010000
    FIX_KEY = 0x00020000
    SINGLE_UNIT = 0x01000000
    DELETE_MARKER = 0x02000000
    SECTOR_CRC = 0x04000000
    PATCH_FILE = 0x00100000


class CompressionType(IntEnum):
    """Compression type bytes."""
    HUFFMAN = 0x01
    ZLIB = 0x02
    PKWARE = 0x08
    BZIP2 = 0x10
    SPARSE = 0x20
    ADPCM_MONO = 0x40
    ADPCM_STEREO = 0x80
    LZMA = 0x12


class HashType(IntEnum):
    """Hash types for MPQ operations."""
    TABLE_OFFSET = 0
    NAME_A = 1
    NAME_B = 2
    FILE_KEY = 3
    KEY2_MIX = 4


@dataclass
class MPQFile:
    """Represents a file to be added to an MPQ archive."""
    name: str
    data: bytes
    compress: bool = True
    encrypt: bool = False
    fix_key: bool = False
    single_unit: bool = False
    sector_crc: bool = False
    locale: int = 0
    compression_type: CompressionType = CompressionType.ZLIB


@dataclass
class MPQConfig:
    """Configuration for building an MPQ archive."""
    version: MPQVersion = MPQVersion.V1
    hash_table_size: int = 16  # Must be power of 2
    block_size: int = 3  # 4096 byte sectors
    files: List[MPQFile] = field(default_factory=list)
    include_userdata: bool = False
    include_signature: bool = False
    debug: bool = False  # Enable debug output


@dataclass
class HashEntry:
    """Hash table entry."""
    name_1: int
    name_2: int
    locale: int
    platform: int
    block_index: int
    preferred_index: int


class MPQCrypto:
    """MPQ encryption/decryption implementation."""

    _instance = None
    _encryption_table = None

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance._init_encryption_table()
        return cls._instance

    def _init_encryption_table(self):
        """Generate the MPQ encryption table (singleton)."""
        if MPQCrypto._encryption_table is None:
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

            MPQCrypto._encryption_table = table

    @property
    def encryption_table(self):
        return MPQCrypto._encryption_table

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
        if key == 0 or len(data) == 0:
            return

        seed = 0xEEEEEEEE

        # Process as 32-bit integers
        for i in range(0, len(data) - 3, 4):
            seed = (seed + self.encryption_table[0x400 + (key & 0xFF)]) & 0xFFFFFFFF
            dword = struct.unpack_from('<I', data, i)[0]
            encrypted = dword ^ ((key + seed) & 0xFFFFFFFF)
            struct.pack_into('<I', data, i, encrypted)
            key = (((~key << 0x15) + 0x11111111) | (key >> 0x0B)) & 0xFFFFFFFF
            seed = (dword + seed + (seed << 5) + 3) & 0xFFFFFFFF

    def decrypt_block(self, data: bytearray, key: int) -> None:
        """Decrypt data in-place (same as encrypt for this algorithm)."""
        self.encrypt_block(data, key)

    def test_encryption(self):
        """Test encryption/decryption with known vectors."""
        original = [0x12345678, 0x9ABCDEF0, 0x13579BDF, 0x2468ACE0,
                   0xFEDCBA98, 0x76543210, 0xF0DEBC9A, 0xE1C3A597]

        # Convert to bytes
        data = bytearray()
        for val in original:
            data.extend(struct.pack('<I', val))

        key = 0xC1EB1CEF

        print("Original data:")
        for i, val in enumerate(original):
            print(f"  [{i}]: 0x{val:08X}")

        # Encrypt
        self.encrypt_block(data, key)
        print("\nEncrypted data:")
        for i in range(0, len(data), 4):
            val = struct.unpack_from('<I', data, i)[0]
            print(f"  [{i//4}]: 0x{val:08X}")

        # Decrypt
        self.decrypt_block(data, key)
        print("\nDecrypted data:")
        decrypted = []
        for i in range(0, len(data), 4):
            val = struct.unpack_from('<I', data, i)[0]
            decrypted.append(val)
            print(f"  [{i//4}]: 0x{val:08X}")

        if decrypted == original:
            print("\n✓ Round-trip encryption/decryption successful!")
        else:
            print("\n✗ Round-trip failed!")


class MPQBuilder:
    """Builds MPQ archives with specified configuration."""

    def __init__(self, config: MPQConfig):
        self.config = config
        self.crypto = MPQCrypto()

    def build(self, output_path: str) -> None:
        """Build the MPQ archive."""
        with open(output_path, 'wb') as f:
            if self.config.include_userdata:
                self._write_userdata_archive(f)
            else:
                self._write_standard_archive(f)

    def _write_standard_archive(self, f: BinaryIO) -> None:
        """Write standard MPQ archive."""
        # Reserve header space
        header_size = self._get_header_size()
        header_offset = 0

        # Write some junk before MPQ if testing offset detection
        if self.config.version == MPQVersion.V1 and len(self.config.files) == 1:
            # For minimal test files, add junk before header
            f.write(b'JUNK' * 128)  # 512 bytes
            header_offset = 512

        f.write(b'\x00' * header_size)

        # Build file data and tables
        file_data_offset = max(0x1000, header_offset + header_size + 0x800)
        hash_entries = []
        block_entries = []

        # Process each file
        for mpq_file in self.config.files:
            file_data, compressed_size = self._prepare_file_data(
                mpq_file, file_data_offset)

            # Write file data
            f.seek(file_data_offset)
            f.write(file_data)

            # Create table entries
            flags = self._get_file_flags(mpq_file)

            block_entries.append({
                'file_pos': file_data_offset - header_offset,
                'compressed_size': compressed_size,
                'file_size': len(mpq_file.data),
                'flags': flags
            })

            hash_entries.append(self._create_hash_entry(
                mpq_file, len(block_entries) - 1))

            file_data_offset += len(file_data)
            file_data_offset = (file_data_offset + 511) & ~511  # Align to 512

        # Write tables
        hash_table_offset = file_data_offset - header_offset
        f.seek(file_data_offset)
        self._write_hash_table(f, hash_entries)

        block_table_offset = f.tell() - header_offset
        self._write_block_table(f, block_entries)

        # Calculate sizes
        archive_size = f.tell() - header_offset

        # Write header
        f.seek(header_offset)
        self._write_header(f, archive_size, hash_table_offset,
                          block_table_offset, len(block_entries))

    def _write_userdata_archive(self, f: BinaryIO) -> None:
        """Write MPQ with user data header."""
        # User data header
        f.write(struct.pack('<I', 0x1B51504D))  # Signature 'MPQ\x1B'
        f.write(struct.pack('<I', 512))         # User data size
        f.write(struct.pack('<I', 512))         # Header offset
        f.write(struct.pack('<I', 16))          # User data header size

        # User data content
        f.write(b'USER' * 124)  # 496 bytes to reach offset 512

        # Standard MPQ follows at offset 512
        self._write_standard_archive(f)

    def _get_header_size(self) -> int:
        """Get header size for the configured version."""
        sizes = {
            MPQVersion.V1: 32,
            MPQVersion.V2: 44,
            MPQVersion.V3: 68,
            MPQVersion.V4: 208
        }
        return sizes[self.config.version]

    def _get_file_flags(self, mpq_file: MPQFile) -> int:
        """Calculate flags for a file."""
        flags = MPQFlags.EXISTS

        if mpq_file.compress:
            flags |= MPQFlags.COMPRESS
        if mpq_file.encrypt:
            flags |= MPQFlags.ENCRYPTED
        if mpq_file.fix_key:
            flags |= MPQFlags.FIX_KEY
        if mpq_file.single_unit:
            flags |= MPQFlags.SINGLE_UNIT
        if mpq_file.sector_crc:
            flags |= MPQFlags.SECTOR_CRC

        return flags

    def _prepare_file_data(self, mpq_file: MPQFile, file_offset: int) -> Tuple[bytes, int]:
        """Prepare file data with compression/encryption/sectors."""
        if mpq_file.single_unit:
            return self._prepare_single_unit_file(mpq_file, file_offset)
        else:
            return self._prepare_sectored_file(mpq_file, file_offset)

    def _prepare_single_unit_file(self, mpq_file: MPQFile, file_offset: int) -> Tuple[bytes, int]:
        """Prepare a single unit file."""
        data = mpq_file.data

        # Compress if needed
        if mpq_file.compress:
            data = self._compress_data(data, mpq_file.compression_type)

        compressed_size = len(data)
        data = bytearray(data)

        # Add CRC if needed
        if mpq_file.sector_crc:
            crc = zlib.crc32(mpq_file.data) & 0xFFFFFFFF
            data.extend(struct.pack('<I', crc))

        # Encrypt if needed
        if mpq_file.encrypt:
            key = self.crypto.hash_string(mpq_file.name, HashType.FILE_KEY)
            if mpq_file.fix_key:
                key = (key + file_offset) ^ len(mpq_file.data)
            self.crypto.encrypt_block(data, key)

        return bytes(data), compressed_size

    def _prepare_sectored_file(self, mpq_file: MPQFile, file_offset: int) -> Tuple[bytes, int]:
        """Prepare a multi-sector file."""
        sector_size = 512 << self.config.block_size
        data = mpq_file.data

        sectors = []
        sector_crcs = []
        sector_offsets = [0]

        # Split into sectors
        for i in range(0, len(data), sector_size):
            sector_data = data[i:i + sector_size]

            # Calculate CRC if needed (on uncompressed data)
            if mpq_file.sector_crc:
                crc = zlib.crc32(sector_data) & 0xFFFFFFFF
                sector_crcs.append(crc)

            # Compress sector if needed
            if mpq_file.compress and len(sector_data) > 1:
                compressed = self._compress_data(sector_data, mpq_file.compression_type)
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
        if mpq_file.encrypt:
            key = self.crypto.hash_string(mpq_file.name, HashType.FILE_KEY)
            if mpq_file.fix_key:
                key = (key + file_offset) ^ len(mpq_file.data)
            offset_key = (key - 1) & 0xFFFFFFFF
            self.crypto.encrypt_block(offset_data, offset_key)

        result.extend(offset_data)

        # CRC table if present (not encrypted)
        if mpq_file.sector_crc:
            for crc in sector_crcs:
                result.extend(struct.pack('<I', crc))

        # Add sectors
        for i, sector in enumerate(sectors):
            sector_data = bytearray(sector)
            if mpq_file.encrypt:
                key = self.crypto.hash_string(mpq_file.name, HashType.FILE_KEY)
                if mpq_file.fix_key:
                    key = (key + file_offset) ^ len(mpq_file.data)
                sector_key = (key + i) & 0xFFFFFFFF
                self.crypto.encrypt_block(sector_data, sector_key)
            result.extend(sector_data)

        return bytes(result), len(result)

    def _compress_data(self, data: bytes, method: CompressionType) -> bytes:
        """Compress data using specified method."""
        if method == CompressionType.ZLIB:
            return bytes([CompressionType.ZLIB]) + zlib.compress(data)
        elif method == CompressionType.BZIP2:
            return bytes([CompressionType.BZIP2]) + bz2.compress(data)
        elif method == CompressionType.SPARSE:
            return bytes([CompressionType.SPARSE]) + self._sparse_compress(data)
        else:
            return data

    def _sparse_compress(self, data: bytes) -> bytes:
        """Simple RLE compression for sparse data."""
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
        return bytes(result)

    def _create_hash_entry(self, mpq_file: MPQFile, block_index: int) -> HashEntry:
        """Create a hash table entry for a file."""
        return HashEntry(
            name_1=self.crypto.hash_string(mpq_file.name, HashType.NAME_A),
            name_2=self.crypto.hash_string(mpq_file.name, HashType.NAME_B),
            locale=mpq_file.locale,
            platform=0,
            block_index=block_index,
            preferred_index=self.crypto.hash_string(mpq_file.name, HashType.TABLE_OFFSET) % self.config.hash_table_size
        )

    def _write_hash_table(self, f: BinaryIO, entries: List[HashEntry]) -> None:
        """Write and encrypt the hash table."""
        # Check if we have too many entries
        if len(entries) > self.config.hash_table_size:
            print(f"WARNING: Trying to fit {len(entries)} files into hash table of size {self.config.hash_table_size}")
            print("This is impossible! Adjusting hash table size to next power of 2...")

            # Find next power of 2 that can fit all entries
            new_size = 1
            while new_size < len(entries):
                new_size *= 2
            self.config.hash_table_size = new_size
            print(f"Hash table size adjusted to: {new_size}")

        # Create hash table with linear probing
        hash_table = [None] * self.config.hash_table_size

        # Debug: Track placement
        placement_attempts = {}

        for i, entry in enumerate(entries):
            idx = entry.preferred_index % self.config.hash_table_size
            start_idx = idx
            attempts = 0

            while hash_table[idx] is not None:
                attempts += 1
                if attempts >= self.config.hash_table_size:
                    # This should never happen now, but let's be safe
                    raise RuntimeError(f"Cannot place entry {i} ({entry.name_1:08X},{entry.name_2:08X}) - hash table full!")
                idx = (idx + 1) % self.config.hash_table_size

            hash_table[idx] = entry
            placement_attempts[i] = (start_idx, idx, attempts)

            if attempts > 0:
                print(f"  File {i}: collision at index {start_idx}, placed at {idx} after {attempts} attempts")

        # Write hash entries
        hash_data = bytearray()
        for i in range(self.config.hash_table_size):
            if hash_table[i]:
                entry = hash_table[i]
                hash_data.extend(struct.pack('<II', entry.name_1, entry.name_2))
                hash_data.extend(struct.pack('<HH', entry.locale, entry.platform))
                hash_data.extend(struct.pack('<I', entry.block_index))
            else:
                # Empty entry
                hash_data.extend(struct.pack('<IIHHI', 0, 0, 0, 0, 0xFFFFFFFF))

        # Encrypt hash table
        key = self.crypto.hash_string("(hash table)", HashType.FILE_KEY)
        self.crypto.encrypt_block(hash_data, key)
        f.write(hash_data)

    def _write_block_table(self, f: BinaryIO, entries: List[Dict[str, Any]]) -> None:
        """Write and encrypt the block table."""
        block_data = bytearray()

        for entry in entries:
            block_data.extend(struct.pack('<I', entry['file_pos']))
            block_data.extend(struct.pack('<I', entry['compressed_size']))
            block_data.extend(struct.pack('<I', entry['file_size']))
            block_data.extend(struct.pack('<I', entry['flags']))

        # Encrypt block table
        key = self.crypto.hash_string("(block table)", HashType.FILE_KEY)
        self.crypto.encrypt_block(block_data, key)
        f.write(block_data)

    def _write_header(self, f: BinaryIO, archive_size: int, hash_table_offset: int,
                     block_table_offset: int, block_count: int) -> None:
        """Write the MPQ header."""
        # Basic header (v1)
        f.write(struct.pack('<I', 0x1A51504D))  # Signature
        f.write(struct.pack('<I', self._get_header_size()))
        f.write(struct.pack('<I', archive_size))
        f.write(struct.pack('<H', self.config.version))
        f.write(struct.pack('<H', self.config.block_size))
        f.write(struct.pack('<I', hash_table_offset))
        f.write(struct.pack('<I', block_table_offset))
        f.write(struct.pack('<I', self.config.hash_table_size))
        f.write(struct.pack('<I', block_count))

        if self.config.version >= MPQVersion.V2:
            # v2 fields
            f.write(struct.pack('<Q', 0))  # Hi-block table position
            f.write(struct.pack('<H', 0))  # Hash table pos high
            f.write(struct.pack('<H', 0))  # Block table pos high

        if self.config.version >= MPQVersion.V3:
            # v3 fields
            f.write(struct.pack('<Q', archive_size))  # 64-bit archive size
            f.write(struct.pack('<Q', 0))  # BET table position
            f.write(struct.pack('<Q', 0))  # HET table position

        if self.config.version >= MPQVersion.V4:
            # v4 fields - compressed sizes
            hash_size = self.config.hash_table_size * 16
            block_size = block_count * 16

            f.write(struct.pack('<Q', hash_size))
            f.write(struct.pack('<Q', block_size))
            f.write(struct.pack('<Q', 0))  # Hi-block size
            f.write(struct.pack('<Q', 0))  # HET size
            f.write(struct.pack('<Q', 0))  # BET size
            f.write(struct.pack('<I', 0x2000))  # Raw chunk size

            # MD5 hashes (dummy values for test archives)
            for _ in range(6):
                f.write(hashlib.md5(b'dummy').digest())


class MPQReader:
    """Read and analyze MPQ archives."""

    def __init__(self):
        self.crypto = MPQCrypto()

    def read_info(self, archive_path: str) -> None:
        """Display information about an MPQ archive."""
        with open(archive_path, 'rb') as f:
            # Find MPQ header
            offset = self._find_mpq_header(f)
            if offset is None:
                print("Error: No valid MPQ header found")
                return

            print(f"MPQ Archive: {archive_path}")
            print(f"Header found at offset: 0x{offset:08X}")

            # Read header
            f.seek(offset)
            header = self._read_header(f)

            print(f"\nHeader Information:")
            print(f"  Format Version: {header['version']} (v{header['version'] + 1})")
            print(f"  Header Size: {header['header_size']} bytes")
            print(f"  Archive Size: {header['archive_size']} bytes")
            print(f"  Block Size: {header['block_size']} (sector size: {512 << header['block_size']} bytes)")
            print(f"  Hash Table: offset 0x{header['hash_table_pos']:08X}, {header['hash_table_size']} entries")
            print(f"  Block Table: offset 0x{header['block_table_pos']:08X}, {header['block_table_size']} entries")

            # Try to read tables
            if header['hash_table_size'] > 0:
                self._analyze_tables(f, offset, header)

    def _find_mpq_header(self, f: BinaryIO) -> Optional[int]:
        """Find the MPQ header in the file."""
        f.seek(0, 2)  # Seek to end
        file_size = f.tell()

        for offset in range(0, min(file_size, 0x100000), 0x200):  # Check first 1MB
            f.seek(offset)
            signature = f.read(4)

            if len(signature) < 4:
                continue

            sig_value = struct.unpack('<I', signature)[0]

            if sig_value == 0x1A51504D:  # 'MPQ\x1A'
                return offset
            elif sig_value == 0x1B51504D:  # 'MPQ\x1B' (user data)
                f.seek(offset + 8)
                header_offset = struct.unpack('<I', f.read(4))[0]
                return offset + header_offset

        return None

    def _read_header(self, f: BinaryIO) -> Dict[str, Any]:
        """Read MPQ header."""
        start_pos = f.tell()

        # Read basic header
        signature = struct.unpack('<I', f.read(4))[0]
        header_size = struct.unpack('<I', f.read(4))[0]
        archive_size = struct.unpack('<I', f.read(4))[0]
        version = struct.unpack('<H', f.read(2))[0]
        block_size = struct.unpack('<H', f.read(2))[0]
        hash_table_pos = struct.unpack('<I', f.read(4))[0]
        block_table_pos = struct.unpack('<I', f.read(4))[0]
        hash_table_size = struct.unpack('<I', f.read(4))[0]
        block_table_size = struct.unpack('<I', f.read(4))[0]

        header = {
            'signature': signature,
            'header_size': header_size,
            'archive_size': archive_size,
            'version': version,
            'block_size': block_size,
            'hash_table_pos': hash_table_pos,
            'block_table_pos': block_table_pos,
            'hash_table_size': hash_table_size,
            'block_table_size': block_table_size
        }

        # Read version-specific fields
        if version >= 1 and header_size >= 44:
            header['hi_block_table_pos'] = struct.unpack('<Q', f.read(8))[0]
            header['hash_table_pos_hi'] = struct.unpack('<H', f.read(2))[0]
            header['block_table_pos_hi'] = struct.unpack('<H', f.read(2))[0]

        if version >= 2 and header_size >= 68:
            header['archive_size_64'] = struct.unpack('<Q', f.read(8))[0]
            header['bet_table_pos'] = struct.unpack('<Q', f.read(8))[0]
            header['het_table_pos'] = struct.unpack('<Q', f.read(8))[0]

        return header

    def _analyze_tables(self, f: BinaryIO, archive_offset: int, header: Dict[str, Any]) -> None:
        """Analyze hash and block tables."""
        # Read hash table
        f.seek(archive_offset + header['hash_table_pos'])
        hash_data = f.read(header['hash_table_size'] * 16)

        # Decrypt hash table
        hash_data = bytearray(hash_data)
        key = self.crypto.hash_string("(hash table)", HashType.FILE_KEY)
        self.crypto.decrypt_block(hash_data, key)

        # Count valid entries
        valid_count = 0
        deleted_count = 0
        empty_count = 0

        for i in range(header['hash_table_size']):
            offset = i * 16
            block_index = struct.unpack_from('<I', hash_data, offset + 12)[0]

            if block_index == 0xFFFFFFFF:
                empty_count += 1
            elif block_index == 0xFFFFFFFE:
                deleted_count += 1
            else:
                valid_count += 1

        print(f"\nHash Table Analysis:")
        print(f"  Valid entries: {valid_count}")
        print(f"  Deleted entries: {deleted_count}")
        print(f"  Empty entries: {empty_count}")
        print(f"  Load factor: {valid_count / header['hash_table_size'] * 100:.1f}%")


class MPQTools:
    """Main tool interface."""

    def __init__(self):
        self.crypto = MPQCrypto()

    def create_minimal(self, version: int, output_dir: str):
        """Create minimal test MPQ for specified version."""
        config = MPQConfig(version=MPQVersion(version))

        # Add a simple test file
        config.files.append(MPQFile(
            name="test.txt",
            data=b"Hello, MPQ!",
            compress=False,
            encrypt=False
        ))

        # Add (listfile) for v1 example
        if version == 0:
            config.files.append(MPQFile(
                name="(listfile)",
                data=b"test.txt\n",
                compress=False
            ))

        builder = MPQBuilder(config)
        output_path = f"{output_dir}/v{version + 1}/simple.mpq"
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        builder.build(output_path)
        print(f"Created: {output_path}")

    def create_compressed(self, compression: str, output_dir: str):
        """Create MPQ with compressed files."""
        config = MPQConfig()

        # Add various compressed files
        test_data = b"This is test data that should compress well because it has repeated patterns. " * 50

        if compression == "all":
            compressions = [
                (CompressionType.ZLIB, "zlib"),
                (CompressionType.BZIP2, "bzip2"),
                (CompressionType.SPARSE, "sparse")
            ]
        else:
            comp_type = CompressionType[compression.upper()]
            compressions = [(comp_type, compression)]

        for comp_type, comp_name in compressions:
            config.files.append(MPQFile(
                name=f"compressed_{comp_name}.dat",
                data=test_data,
                compress=True,
                compression_type=comp_type
            ))

        # Add special case for sparse
        if any(c[0] == CompressionType.SPARSE for c in compressions):
            sparse_data = b'\x00' * 1000 + b'SPARSE_DATA' + b'\x00' * 1000
            config.files.append(MPQFile(
                name="sparse_special.dat",
                data=sparse_data,
                compress=True,
                compression_type=CompressionType.SPARSE
            ))

        # Add (listfile)
        listfile = '\n'.join(f.name for f in config.files)
        config.files.append(MPQFile("(listfile)", listfile.encode('ascii')))

        builder = MPQBuilder(config)
        output_path = f"{output_dir}/compressed/{compression}.mpq"
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        builder.build(output_path)
        print(f"Created: {output_path}")

    def create_crc(self, output_dir: str):
        """Create MPQ files with CRC validation."""
        # Multi-sector file with CRCs
        config = MPQConfig(block_size=2)  # 2048 byte sectors

        test_data = b"This is test data for CRC validation. " * 100  # ~3900 bytes
        config.files.append(MPQFile(
            name="test_crc.txt",
            data=test_data,
            compress=True,
            sector_crc=True,
            single_unit=False
        ))

        builder = MPQBuilder(config)
        output_path = f"{output_dir}/crc/sectors.mpq"
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        builder.build(output_path)
        print(f"Created: {output_path} (multi-sector with CRCs)")

        # Single unit file with CRC
        config2 = MPQConfig()
        config2.files.append(MPQFile(
            name="single_crc.txt",
            data=b"This is a single unit file with CRC validation.",
            compress=True,
            single_unit=True,
            sector_crc=True
        ))

        builder2 = MPQBuilder(config2)
        output_path2 = f"{output_dir}/crc/single.mpq"
        builder2.build(output_path2)
        print(f"Created: {output_path2} (single unit with CRC)")

    def create_comprehensive(self, all_versions: bool, version: Optional[int], output_dir: str):
        """Create comprehensive test archives."""
        versions = range(4) if all_versions else [version if version is not None else 0]

        for ver in versions:
            config = MPQConfig(version=MPQVersion(ver))

            # Add various test files
            self._add_comprehensive_files(config, ver)

            builder = MPQBuilder(config)
            output_path = f"{output_dir}/comprehensive/test_v{ver + 1}.mpq"
            os.makedirs(os.path.dirname(output_path), exist_ok=True)
            builder.build(output_path)
            print(f"Created: {output_path}")
            print(f"  Files: {len(config.files)}")
            print(f"  Features tested:")
            print(f"    - Compression: zlib", end='')
            if ver >= 1:
                print(", bzip2", end='')
            if ver >= 2:
                print(", sparse", end='')
            print()
            print(f"    - Encryption: standard, fixed key")
            print(f"    - Sector CRC validation")
            print(f"    - Multiple locales")
            print(f"    - Special files: (listfile), (attributes)")

    def create_edge_cases(self, output_dir: str):
        """Create edge case test archives."""
        print("\nCreating edge case test archives...")

        # Hash collision test - realistic scenario
        print("\n1. Creating hash collision test...")
        config = MPQConfig(hash_table_size=8)  # Small but not impossible

        # These filenames are designed to create collisions
        collision_files = [
            "file1.txt",
            "file2.txt",
            "file3.txt",
            "test1.dat",
            "test2.dat",
            "data.bin",
            "info.txt"
        ]

        print(f"  Hash table size: {config.hash_table_size}")
        print(f"  Number of files: {len(collision_files)}")
        print("  Hash distribution:")

        for i, filename in enumerate(collision_files):
            hash_offset = self.crypto.hash_string(filename, HashType.TABLE_OFFSET)
            preferred_idx = hash_offset % config.hash_table_size
            print(f"    {filename}: hash=0x{hash_offset:08X}, preferred_index={preferred_idx}")

            config.files.append(MPQFile(
                name=filename,
                data=f"File content for {filename}".encode()
            ))

        builder = MPQBuilder(config)
        output_path = f"{output_dir}/comprehensive/hash_collisions.mpq"
        os.makedirs(os.path.dirname(output_path), exist_ok=True)

        print("  Building archive...")
        builder.build(output_path)
        print(f"  Created: {output_path}")

        # Empty archive
        print("\n2. Creating empty archive...")
        config2 = MPQConfig()
        builder2 = MPQBuilder(config2)
        output_path2 = f"{output_dir}/comprehensive/empty.mpq"
        builder2.build(output_path2)
        print(f"  Created: {output_path2}")

        # Large hash table
        print("\n3. Creating large hash table test...")
        config3 = MPQConfig(hash_table_size=1024)  # More reasonable than 65536
        config3.files.append(MPQFile("single.txt", b"One file in large table"))

        print(f"  Hash table size: {config3.hash_table_size}")
        print(f"  Number of files: 1")
        print(f"  Load factor: {1/config3.hash_table_size*100:.2f}%")

        builder3 = MPQBuilder(config3)
        output_path3 = f"{output_dir}/comprehensive/large_hashtable.mpq"
        builder3.build(output_path3)
        print(f"  Created: {output_path3}")

        # User data header
        print("\n4. Creating archive with user data header...")
        config4 = MPQConfig(include_userdata=True)
        config4.files.append(MPQFile("test.txt", b"Archive with user data"))
        config4.files.append(MPQFile("(listfile)", b"test.txt\n"))

        builder4 = MPQBuilder(config4)
        output_path4 = f"{output_dir}/v1/userdata.mpq"
        os.makedirs(os.path.dirname(output_path4), exist_ok=True)
        builder4.build(output_path4)
        print(f"  Created: {output_path4}")

        print("\nAll edge case archives created successfully!")

    def verify_encryption_table(self):
        """Verify the encryption table implementation."""
        print("MPQ Encryption Table Test")
        print("========================")
        print()

        # Test known values
        known_values = {
            0x000: 0x55C636E2,
            0x001: 0x02BE0170,
            0x002: 0x584B71D4,
            0x003: 0x2984F00E,
            0x004: 0xB682C809,
            0x100: 0x76F8C1B1,
            0x200: 0x3DF6965D,
            0x300: 0x15F261D3,
            0x400: 0x193AA698,
            0x4FB: 0x6149809C,
            0x4FC: 0xB0099EF4,
            0x4FD: 0xC5F653A5,
            0x4FE: 0x4C10790D,
            0x4FF: 0x7303286C,
        }

        table = self.crypto.encryption_table
        all_correct = True

        print("Verifying known values:")
        for index, expected in known_values.items():
            actual = table[index]
            if actual == expected:
                print(f"  ✓ [0x{index:03X}]: 0x{actual:08X}")
            else:
                print(f"  ✗ [0x{index:03X}]: got 0x{actual:08X}, expected 0x{expected:08X}")
                all_correct = False

        if all_correct:
            print("\n✓ All encryption table values are correct!")
        else:
            print("\n✗ Some values don't match!")

        # Test encryption/decryption
        print("\n" + "="*50)
        self.crypto.test_encryption()

    def verify_hash_function(self):
        """Verify hash function with known test vectors."""
        print("MPQ Hash Function Test")
        print("======================")
        print()

        test_vectors = [
            ("(listfile)", HashType.TABLE_OFFSET, 0xFD5F6EEA),
            ("(hash table)", HashType.FILE_KEY, 0xC3AF3770),
            ("(block table)", HashType.FILE_KEY, 0xEC83B3A3),
        ]

        print("Testing known hash values:")
        all_correct = True

        for filename, hash_type, expected in test_vectors:
            actual = self.crypto.hash_string(filename, hash_type)
            if actual == expected:
                print(f"  ✓ hash_string('{filename}', {hash_type.name}): 0x{actual:08X}")
            else:
                print(f"  ✗ hash_string('{filename}', {hash_type.name}): got 0x{actual:08X}, expected 0x{expected:08X}")
                all_correct = False

        if all_correct:
            print("\n✓ All hash values are correct!")
        else:
            print("\n✗ Some hash values don't match!")

        # Test case insensitivity
        print("\nTesting case insensitivity:")
        test1 = self.crypto.hash_string("file.txt", HashType.TABLE_OFFSET)
        test2 = self.crypto.hash_string("FILE.TXT", HashType.TABLE_OFFSET)
        if test1 == test2:
            print(f"  ✓ 'file.txt' and 'FILE.TXT' produce same hash: 0x{test1:08X}")
        else:
            print(f"  ✗ Case insensitivity failed: 0x{test1:08X} != 0x{test2:08X}")

        # Test path separator normalization
        print("\nTesting path separator normalization:")
        test3 = self.crypto.hash_string("path/to/file", HashType.TABLE_OFFSET)
        test4 = self.crypto.hash_string("path\\to\\file", HashType.TABLE_OFFSET)
        if test3 == test4:
            print(f"  ✓ Forward and backslashes produce same hash: 0x{test3:08X}")
        else:
            print(f"  ✗ Path separator normalization failed")

    def _add_comprehensive_files(self, config: MPQConfig, version: int):
        """Add comprehensive test files based on version."""
        # Basic files for all versions
        config.files.extend([
            MPQFile("readme.txt", b"Test archive for mopaq implementation"),
            MPQFile("data/binary.dat", bytes(range(256)) * 16, compress=True),
            MPQFile("scripts/test.lua", b"-- Lua script\nprint('Hello from MPQ!')\n",
                   compress=True, encrypt=True),
            MPQFile("images/test.blp", b"BLP2" + b'\x00' * 148,
                   encrypt=True, fix_key=True),
            MPQFile("sounds/test.wav", b"RIFF" + b'\x00' * 40 + b"data" + b'\x00' * 1000,
                   compress=True, single_unit=True, sector_crc=True),
            MPQFile("large_file.bin", b"X" * 10000, compress=True, sector_crc=True),
            MPQFile("unicode_test.txt", 'Test unicode: café, Москва, 東京'.encode('utf-8')),
        ])

        # Version-specific files
        if version >= 1:  # v2+
            config.files.append(
                MPQFile("zero_file.dat", b'\x00' * 5000,
                       compress=True, compression_type=CompressionType.BZIP2)
            )

        if version >= 2:  # v3+
            config.files.append(
                MPQFile("sparse_test.dat", b'\x00' * 1000 + b'DATA' + b'\x00' * 1000,
                       compress=True, compression_type=CompressionType.SPARSE)
            )

        # Multiple locales
        config.files.extend([
            MPQFile("locale_test.txt", b"English version", locale=0x0409),
            MPQFile("locale_test.txt", b"German version", locale=0x0407),
        ])

        # Special files
        listfile_content = '\n'.join(f.name for f in config.files)
        config.files.append(MPQFile("(listfile)", listfile_content.encode('ascii')))

        # Create attributes
        attributes = []
        for f in config.files:
            attrs = {
                'name': f.name,
                'crc32': zlib.crc32(f.data) & 0xFFFFFFFF,
                'timestamp': int(time.time()),
                'md5': hashlib.md5(f.data).hexdigest()
            }
            attributes.append(attrs)

        attr_data = json.dumps(attributes, indent=2).encode('utf-8')
        config.files.append(MPQFile("(attributes)", attr_data, compress=True))


def main():
    parser = argparse.ArgumentParser(
        description="MPQ test data generator and utilities",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s create minimal --version 1
  %(prog)s create compressed --compression zlib
  %(prog)s create comprehensive --all-versions
  %(prog)s verify encryption-table
  %(prog)s info test.mpq
        """
    )

    subparsers = parser.add_subparsers(dest='command', help='Commands')

    # Create command
    create_parser = subparsers.add_parser('create', help='Create test MPQ archives')
    create_parser.add_argument('type',
        choices=['minimal', 'compressed', 'comprehensive', 'crc', 'edge-cases'],
        help='Type of test archive to create')
    create_parser.add_argument('--version', type=int, choices=[0, 1, 2, 3],
        help='MPQ version (0=v1, 1=v2, 2=v3, 3=v4)')
    create_parser.add_argument('--compression',
        choices=['zlib', 'bzip2', 'sparse', 'all'],
        default='zlib', help='Compression type')
    create_parser.add_argument('--all-versions', action='store_true',
        help='Create archives for all versions')
    create_parser.add_argument('--output-dir', default='test-data',
        help='Output directory')

    # Verify command
    verify_parser = subparsers.add_parser('verify', help='Verify MPQ components')
    verify_parser.add_argument('type', choices=['encryption-table', 'hash-function'],
        help='Component to verify')

    # Info command
    info_parser = subparsers.add_parser('info', help='Show MPQ archive information')
    info_parser.add_argument('archive', help='Path to MPQ archive')

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return

    tools = MPQTools()

    if args.command == 'create':
        os.makedirs(args.output_dir, exist_ok=True)

        if args.type == 'minimal':
            if args.all_versions:
                for v in range(4):
                    tools.create_minimal(v, args.output_dir)
            else:
                version = args.version if args.version is not None else 0
                tools.create_minimal(version, args.output_dir)

        elif args.type == 'compressed':
            tools.create_compressed(args.compression, args.output_dir)

        elif args.type == 'comprehensive':
            tools.create_comprehensive(args.all_versions, args.version, args.output_dir)

        elif args.type == 'crc':
            tools.create_crc(args.output_dir)

        elif args.type == 'edge-cases':
            tools.create_edge_cases(args.output_dir)

        print("\nTest archives created successfully!")
        print("\nYou can now test with:")
        print("  cargo test")
        print("  cargo run --bin storm-cli -- list <archive>")
        print("  cargo run --bin storm-cli -- verify <archive>")
        print("  cargo run --bin storm-cli -- extract <archive>")

    elif args.command == 'verify':
        if args.type == 'encryption-table':
            tools.verify_encryption_table()
        elif args.type == 'hash-function':
            tools.verify_hash_function()

    elif args.command == 'info':
        reader = MPQReader()
        reader.read_info(args.archive)


if __name__ == "__main__":
    main()
