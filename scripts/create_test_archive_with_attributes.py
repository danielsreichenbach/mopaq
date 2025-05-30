#!/usr/bin/env python3
"""
Create a test MPQ archive with (attributes) file for the test-data directory.
This creates a properly formatted archive that can be used for testing attribute loading.
"""

import struct
import hashlib
import subprocess
import tempfile
from pathlib import Path

def calculate_crc32(data):
    """Calculate CRC32 checksum."""
    import zlib
    return zlib.crc32(data) & 0xFFFFFFFF

def calculate_md5(data):
    """Calculate MD5 hash."""
    return hashlib.md5(data).digest()

def create_attributes_file(files_data):
    """Create a properly formatted (attributes) file."""
    # Version and flags
    version = 100
    flags = 0x0F  # All attributes (CRC32 | FILETIME | MD5 | PATCH_BIT)
    
    data = bytearray()
    
    # Header
    data.extend(struct.pack('<I', version))
    data.extend(struct.pack('<I', flags))
    
    # CRC32 array
    for file_info in files_data:
        data.extend(struct.pack('<I', file_info['crc32']))
    
    # Filetime array (using a fixed timestamp for consistency)
    # Windows FILETIME format: 100-nanosecond intervals since January 1, 1601
    filetime = 0x01D8F1A412345678  # Some arbitrary but valid timestamp
    for _ in files_data:
        data.extend(struct.pack('<Q', filetime))
    
    # MD5 array
    for file_info in files_data:
        data.extend(file_info['md5'])
    
    # Patch bits (none of our files are patches)
    patch_bytes = (len(files_data) + 7) // 8
    data.extend(b'\x00' * patch_bytes)
    
    return bytes(data)

def main():
    """Create test archive with attributes."""
    # Define test files
    test_files = [
        ("test1.txt", b"This is test file 1\n"),
        ("test2.txt", b"This is test file 2\n"),
        ("data/binary.dat", b"\x00\x01\x02\x03\x04\x05\x06\x07"),
        ("nested/deep/file.txt", b"Deeply nested file\n"),
    ]
    
    # Calculate file attributes
    files_data = []
    for name, content in test_files:
        files_data.append({
            'name': name,
            'content': content,
            'crc32': calculate_crc32(content),
            'md5': calculate_md5(content),
        })
    
    # Create attributes data
    attributes_data = create_attributes_file(files_data)
    
    # Create temporary directory for files
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)
        
        # Write test files
        for name, content in test_files:
            file_path = temp_path / name
            file_path.parent.mkdir(parents=True, exist_ok=True)
            file_path.write_bytes(content)
        
        # Write (attributes) file
        attributes_path = temp_path / "(attributes)"
        attributes_path.write_bytes(attributes_data)
        
        # Create Rust program to build the archive
        rust_code = f'''
use mopaq::{{ArchiveBuilder, FormatVersion, compression::flags}};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let temp_dir = Path::new(r"{temp_dir}");
    
    let mut builder = ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .default_compression(flags::ZLIB);
    
    // Add test files
    let files = vec![
        "test1.txt",
        "test2.txt",
        "data/binary.dat",
        "nested/deep/file.txt",
        "(attributes)",
    ];
    
    for file_name in files {{
        let file_path = temp_dir.join(file_name);
        builder = builder.add_file(&file_path, file_name);
    }}
    
    // Build the archive
    builder.build("archive_with_attributes.mpq")?;
    println!("Archive with attributes created successfully!");
    
    Ok(())
}}
'''
        
        # Save and compile the Rust program
        rust_file = temp_path / "create_archive.rs"
        rust_file.write_text(rust_code)
        
        # Create a minimal Cargo.toml
        cargo_toml = '''
[package]
name = "create_test_archive"
version = "0.1.0"
edition = "2021"

[dependencies]
mopaq = { path = "../mopaq" }
'''
        
        cargo_file = temp_path / "Cargo.toml"
        cargo_file.write_text(cargo_toml)
        
        # Create src directory and move the rust file
        src_dir = temp_path / "src"
        src_dir.mkdir()
        (src_dir / "main.rs").write_text(rust_code)
        
        print(f"Created test files in {temp_dir}")
        print(f"(attributes) file size: {len(attributes_data)} bytes")
        print("\nTo create the archive, run:")
        print(f"cd {temp_dir} && cargo build --release && cargo run --release")
        print("\nThen copy archive_with_attributes.mpq to test-data/")
        
        # Keep the directory for manual inspection
        input("Press Enter to clean up...")

if __name__ == "__main__":
    main()