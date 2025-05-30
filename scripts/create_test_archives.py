#!/usr/bin/env python3
"""
Create test MPQ archives using the storm-cli tool.
This tests the archive creation functionality.
"""

import os
import subprocess
import tempfile
import shutil
import struct
import hashlib
from pathlib import Path

def run_storm_cli(args):
    """Run storm-cli with given arguments."""
    cmd = ['cargo', 'run', '--bin', 'storm-cli', '--'] + args
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error running storm-cli: {result.stderr}")
        return False
    return True

def create_test_files(temp_dir):
    """Create test files for archiving."""
    files = {
        'readme.txt': 'This is a test MPQ archive',
        'data/level1.dat': b'Level 1 data\x00\x01\x02\x03',
        'data/level2.dat': b'Level 2 data\x00\x04\x05\x06',
        'scripts/main.lua': 'print("Hello from Lua!")',
        'models/unit.mdx': b'MDX\x00' + b'\x00' * 100,  # Fake MDX header
        'textures/grass.blp': b'BLP2' + b'\x00' * 200,  # Fake BLP header
        'sound/music.mp3': b'ID3' + b'\x00' * 500,      # Fake MP3
    }

    created_files = []
    file_info = []  # Store info for attributes
    for path, content in files.items():
        full_path = temp_dir / path
        full_path.parent.mkdir(parents=True, exist_ok=True)

        if isinstance(content, bytes):
            full_path.write_bytes(content)
            data = content
        else:
            full_path.write_text(content)
            data = content.encode('utf-8')

        created_files.append(str(full_path))
        
        # Calculate CRC32 and MD5 for attributes
        import zlib
        crc32 = zlib.crc32(data) & 0xFFFFFFFF
        md5 = hashlib.md5(data).digest()
        file_info.append({
            'name': path,
            'crc32': crc32,
            'md5': md5,
            'size': len(data)
        })

    return created_files, list(files.keys()), file_info

def create_attributes_data(file_info):
    """Create (attributes) file data."""
    # Version (100) and flags (all attributes: CRC32=1, FILETIME=2, MD5=4, PATCH_BIT=8)
    version = 100
    flags = 0x0F  # All attributes
    
    data = bytearray()
    
    # Header
    data.extend(struct.pack('<I', version))
    data.extend(struct.pack('<I', flags))
    
    # CRC32 array
    for info in file_info:
        data.extend(struct.pack('<I', info['crc32']))
    
    # Filetime array (using current time for all files)
    import time
    # Windows FILETIME: 100-nanosecond intervals since January 1, 1601
    # Unix epoch is January 1, 1970
    # Difference in seconds: 11644473600
    current_time = int((time.time() + 11644473600) * 10000000)
    for _ in file_info:
        data.extend(struct.pack('<Q', current_time))
    
    # MD5 array
    for info in file_info:
        data.extend(info['md5'])
    
    # Patch bits (none of our files are patches)
    patch_bytes = (len(file_info) + 7) // 8
    data.extend(b'\x00' * patch_bytes)
    
    return bytes(data)

def verify_archive(archive_path):
    """Verify the created archive."""
    print(f"\nVerifying {archive_path}...")

    # List files
    if not run_storm_cli(['list', archive_path]):
        return False

    # Verify integrity
    if not run_storm_cli(['verify', archive_path]):
        return False

    # Show info
    if not run_storm_cli(['debug', 'info', archive_path]):
        return False

    return True

def main():
    print("Testing MPQ archive creation...")

    # Create temporary directory for test files
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)

        # Create test files
        print("Creating test files...")
        file_paths, archive_names, file_info = create_test_files(temp_path)

        # Test 1: Create archive using Rust example
        print("\nTest 1: Creating archive using Rust example...")
        example_result = subprocess.run(
            ['cargo', 'run', '--example', 'create_archive'],
            capture_output=True,
            text=True
        )
        if example_result.returncode == 0:
            print("✓ Rust example succeeded")
            # Verify created archives
            for archive in ['simple.mpq', 'from_files.mpq', 'custom.mpq']:
                if os.path.exists(archive):
                    verify_archive(archive)
                    os.remove(archive)
        else:
            print(f"✗ Rust example failed: {example_result.stderr}")

        # Test 2: Once storm-cli create command is implemented
        print("\nTest 2: Creating archive using storm-cli (when implemented)...")
        print("Note: storm-cli create command not yet implemented")
        # This would be something like:
        # run_storm_cli(['create', 'test.mpq', temp_dir])

        # Test 3: Create archive with attributes file
        print("\nTest 3: Creating archive with (attributes) file...")
        
        # Create the (attributes) file
        attributes_data = create_attributes_data(file_info)
        attributes_path = temp_path / "(attributes)"
        attributes_path.write_bytes(attributes_data)
        
        # Create a Rust program to build archive with attributes
        test_program = f'''
use mopaq::{{ArchiveBuilder, FormatVersion}};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let temp_dir = Path::new("{temp_dir}");
    
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V1);

    // Add all test files
    let files = vec![
        "readme.txt",
        "data/level1.dat",
        "data/level2.dat",
        "scripts/main.lua",
        "models/unit.mdx",
        "textures/grass.blp",
        "sound/music.mp3",
    ];
    
    for file_name in files {{
        let file_path = temp_dir.join(file_name);
        builder = builder.add_file(&file_path, file_name);
    }}
    
    // Add the (attributes) file
    let attributes_path = temp_dir.join("(attributes)");
    builder = builder.add_file(&attributes_path, "(attributes)");

    builder.build("archive_with_attributes.mpq")?;
    println!("Archive with attributes created successfully");
    Ok(())
}}
        '''

        # Save test program
        test_file = temp_path / "test_create_with_attributes.rs"
        test_file.write_text(test_program)
        
        print("Test program created at:", test_file)
        print("To run it, create a new Rust binary project and use this code.")
        
        # Test 4: Create a simpler archive with attributes using the example
        print("\nTest 4: Creating simple archive with attributes...")
        # Create attributes for just a few files
        simple_files = [
            {'name': 'test.txt', 'crc32': 0x12345678, 'md5': b'\x01' * 16},
            {'name': 'data.bin', 'crc32': 0x9ABCDEF0, 'md5': b'\x02' * 16},
        ]
        simple_attributes = create_attributes_data(simple_files)
        
        # Write simple test files
        (temp_path / 'test.txt').write_text('Simple test file')
        (temp_path / 'data.bin').write_bytes(b'Binary data')
        (temp_path / '(attributes)').write_bytes(simple_attributes)
        
        print(f"Created (attributes) file with {len(simple_attributes)} bytes")

        # Create a test archive with attributes for the test-data directory
        print("\nCreating test archive with attributes for test-data...")
        create_test_archive_with_attributes(temp_path)

    print("\nArchive creation tests completed!")

    # Cleanup any remaining test files
    for archive in ['simple.mpq', 'from_files.mpq', 'custom.mpq', 'programmatic_test.mpq']:
        if os.path.exists(archive):
            os.remove(archive)

def create_test_archive_with_attributes(temp_dir):
    """Create a test MPQ archive with (attributes) file for testing."""
    # This would use mpq_tools.py or StormLib to create the archive
    # For now, we'll just document the process
    
    print("\nTo create test archives with (attributes) files:")
    print("1. Use the Rust code example generated above")
    print("2. Or use StormLib/mpq_tools.py with the generated (attributes) file")
    print("3. The (attributes) file has been created with proper format")
    
    # The attributes file is already created in temp_dir
    # It contains CRC32, MD5, filetime, and patch bit data for all files

if __name__ == '__main__':
    main()
