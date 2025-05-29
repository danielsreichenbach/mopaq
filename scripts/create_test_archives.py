#!/usr/bin/env python3
"""
Create test MPQ archives using the storm-cli tool.
This tests the archive creation functionality.
"""

import os
import subprocess
import tempfile
import shutil
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
    for path, content in files.items():
        full_path = temp_dir / path
        full_path.parent.mkdir(parents=True, exist_ok=True)

        if isinstance(content, bytes):
            full_path.write_bytes(content)
        else:
            full_path.write_text(content)

        created_files.append(str(full_path))

    return created_files, list(files.keys())

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
        file_paths, archive_names = create_test_files(temp_path)

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

        # Test 3: Create archive programmatically and verify
        print("\nTest 3: Creating archive programmatically...")
        test_archive = "programmatic_test.mpq"

        # Create a simple Rust program to test
        test_program = '''
use mopaq::{ArchiveBuilder, FormatVersion};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        ("readme.txt", "Test archive created programmatically"),
        ("data.bin", "Binary data here"),
    ];

    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V1);

    for (name, content) in files {
        builder = builder.add_file_data(content.as_bytes().to_vec(), name);
    }

    builder.build("programmatic_test.mpq")?;
    println!("Archive created successfully");
    Ok(())
}
        '''

        # Write and run test program
        test_file = temp_path / "test_create.rs"
        test_file.write_text(test_program)

        # Note: This would need proper Cargo setup to run
        print("(Skipping programmatic test - requires separate Cargo project)")

    print("\nArchive creation tests completed!")

    # Cleanup any remaining test files
    for archive in ['simple.mpq', 'from_files.mpq', 'custom.mpq', 'programmatic_test.mpq']:
        if os.path.exists(archive):
            os.remove(archive)

if __name__ == '__main__':
    main()
