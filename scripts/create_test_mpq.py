#!/usr/bin/env python3
"""
Create minimal test MPQ files for different format versions.

This creates simple MPQ headers without full table data, suitable for
testing the header parsing and info command.

Author: Daniel S. Reichenbach <daniel@kogito.network>
"""

import struct
import os
from pathlib import Path


def create_v1_mpq(filename):
    """Create a minimal v1 MPQ file."""
    with open(filename, 'wb') as f:
        # Write some junk data before the MPQ to test offset detection
        f.write(b'JUNK' * 128)  # 512 bytes of junk

        # MPQ Header (32 bytes)
        f.write(struct.pack('<I', 0x1A51504D))  # Signature 'MPQ\x1A'
        f.write(struct.pack('<I', 32))          # Header size
        f.write(struct.pack('<I', 1024))        # Archive size
        f.write(struct.pack('<H', 0))           # Format version (v1)
        f.write(struct.pack('<H', 3))           # Block size (512 * 2^3 = 4096)
        f.write(struct.pack('<I', 0x200))       # Hash table position
        f.write(struct.pack('<I', 0x400))       # Block table position
        f.write(struct.pack('<I', 16))          # Hash table entries
        f.write(struct.pack('<I', 8))           # Block table entries

        # Pad to end of archive
        current_pos = f.tell()
        archive_start = 512  # Where we started the MPQ
        remaining = (archive_start + 1024) - current_pos
        if remaining > 0:
            f.write(b'\x00' * remaining)

    print(f"Created {filename} (v1 format)")


def create_v2_mpq(filename):
    """Create a minimal v2 MPQ file with extended header."""
    with open(filename, 'wb') as f:
        # MPQ Header (44 bytes)
        f.write(struct.pack('<I', 0x1A51504D))  # Signature 'MPQ\x1A'
        f.write(struct.pack('<I', 44))          # Header size
        f.write(struct.pack('<I', 2048))        # Archive size (deprecated)
        f.write(struct.pack('<H', 1))           # Format version (v2)
        f.write(struct.pack('<H', 4))           # Block size
        f.write(struct.pack('<I', 0x400))       # Hash table position
        f.write(struct.pack('<I', 0x800))       # Block table position
        f.write(struct.pack('<I', 32))          # Hash table entries
        f.write(struct.pack('<I', 16))          # Block table entries

        # v2 extended fields
        f.write(struct.pack('<Q', 0x1000))      # Hi-block table position
        f.write(struct.pack('<H', 0))           # Hash table pos high
        f.write(struct.pack('<H', 0))           # Block table pos high

        # Pad to minimum size
        f.write(b'\x00' * 2000)

    print(f"Created {filename} (v2 format)")


def create_v4_mpq(filename):
    """Create a minimal v4 MPQ file with all extended fields."""
    with open(filename, 'wb') as f:
        # MPQ Header (208 bytes)
        f.write(struct.pack('<I', 0x1A51504D))  # Signature 'MPQ\x1A'
        f.write(struct.pack('<I', 208))         # Header size
        f.write(struct.pack('<I', 4096))        # Archive size (deprecated)
        f.write(struct.pack('<H', 3))           # Format version (v4)
        f.write(struct.pack('<H', 5))           # Block size
        f.write(struct.pack('<I', 0x1000))      # Hash table position
        f.write(struct.pack('<I', 0x2000))      # Block table position
        f.write(struct.pack('<I', 64))          # Hash table entries
        f.write(struct.pack('<I', 32))          # Block table entries

        # v2 extended fields
        f.write(struct.pack('<Q', 0x3000))      # Hi-block table position
        f.write(struct.pack('<H', 0))           # Hash table pos high
        f.write(struct.pack('<H', 0))           # Block table pos high

        # v3 extended fields
        f.write(struct.pack('<Q', 8192))        # Archive size 64-bit
        f.write(struct.pack('<Q', 0x4000))      # BET table position
        f.write(struct.pack('<Q', 0x5000))      # HET table position

        # v4 extended fields
        f.write(struct.pack('<Q', 1024))        # Hash table compressed size
        f.write(struct.pack('<Q', 512))         # Block table compressed size
        f.write(struct.pack('<Q', 256))         # Hi-block table compressed size
        f.write(struct.pack('<Q', 2048))        # HET table compressed size
        f.write(struct.pack('<Q', 1024))        # BET table compressed size
        f.write(struct.pack('<I', 16384))       # Raw chunk size

        # MD5 hashes (6 * 16 bytes = 96 bytes)
        for i in range(6):
            f.write(bytes([i] * 16))  # Dummy MD5 values

        # Pad to minimum size
        f.write(b'\x00' * 8000)

    print(f"Created {filename} (v4 format)")


def create_userdata_mpq(filename):
    """Create an MPQ with user data header (common in SC2 maps)."""
    with open(filename, 'wb') as f:
        # User data header
        f.write(struct.pack('<I', 0x1B51504D))  # Signature 'MPQ\x1B'
        f.write(struct.pack('<I', 512))         # User data size
        f.write(struct.pack('<I', 512))         # Header offset (MPQ starts at 512)
        f.write(struct.pack('<I', 16))          # User data header size

        # User data content
        f.write(b'USER' * 124)  # 496 bytes to reach offset 512

        # Standard MPQ header at offset 512
        f.write(struct.pack('<I', 0x1A51504D))  # Signature 'MPQ\x1A'
        f.write(struct.pack('<I', 32))          # Header size
        f.write(struct.pack('<I', 1024))        # Archive size
        f.write(struct.pack('<H', 0))           # Format version (v1)
        f.write(struct.pack('<H', 3))           # Block size
        f.write(struct.pack('<I', 0x200))       # Hash table position
        f.write(struct.pack('<I', 0x400))       # Block table position
        f.write(struct.pack('<I', 16))          # Hash table entries
        f.write(struct.pack('<I', 8))           # Block table entries

        # Pad
        f.write(b'\x00' * 1000)

    print(f"Created {filename} (with user data)")


def main():
    """Create test MPQ files."""
    # Create test-data directories
    for version in ['v1', 'v2', 'v3', 'v4']:
        os.makedirs(f'test-data/{version}', exist_ok=True)

    # Create test files
    create_v1_mpq('test-data/v1/simple.mpq')
    create_v2_mpq('test-data/v2/simple.mpq')
    create_v4_mpq('test-data/v4/simple.mpq')
    create_userdata_mpq('test-data/v1/userdata.mpq')

    print("\nTest MPQ files created successfully!")
    print("\nYou can now test the info command:")
    print("  cargo run --bin storm-cli -- debug info test-data/v1/simple.mpq")
    print("  cargo run --bin storm-cli -- debug info test-data/v2/simple.mpq")
    print("  cargo run --bin storm-cli -- debug info test-data/v4/simple.mpq")
    print("  cargo run --bin storm-cli -- debug info test-data/v1/userdata.mpq")


if __name__ == "__main__":
    main()
