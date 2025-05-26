//! Verify command implementation

use anyhow::{Context, Result};
use mopaq::Archive;

/// Verify the integrity of an MPQ archive
pub fn verify(archive_path: &str, verbose: bool) -> Result<()> {
    println!("Verifying archive: {}", archive_path);
    println!();

    let mut archive = Archive::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path))?;

    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // Check header
    verify_header(&archive, &mut issues, &mut warnings);

    // Check tables
    verify_tables(&archive, &mut issues, &mut warnings)?;

    // Check files if we have a listfile
    if let Ok(Some(_)) = archive.find_file("(listfile)") {
        verify_files(&mut archive, &mut issues, &mut warnings, verbose)?;
    } else {
        warnings.push("No (listfile) found - cannot verify individual files".to_string());
    }

    // Report results
    println!("Verification Results:");
    println!("====================");

    if issues.is_empty() && warnings.is_empty() {
        println!("✓ Archive appears to be valid");
    } else {
        if !issues.is_empty() {
            println!();
            println!("Issues found ({}):", issues.len());
            for issue in &issues {
                println!("  ✗ {}", issue);
            }
        }

        if !warnings.is_empty() {
            println!();
            println!("Warnings ({}):", warnings.len());
            for warning in &warnings {
                println!("  ⚠ {}", warning);
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Archive verification failed with {} issues",
            issues.len()
        ))
    }
}

/// Verify the MPQ header
fn verify_header(archive: &Archive, issues: &mut Vec<String>, warnings: &mut Vec<String>) {
    let header = archive.header();

    // Check header size
    match header.format_version {
        mopaq::FormatVersion::V1 => {
            if header.header_size < 32 {
                issues.push(format!(
                    "Invalid header size {} for v1 (expected >= 32)",
                    header.header_size
                ));
            }
        }
        mopaq::FormatVersion::V2 => {
            if header.header_size < 44 {
                issues.push(format!(
                    "Invalid header size {} for v2 (expected >= 44)",
                    header.header_size
                ));
            }
        }
        mopaq::FormatVersion::V3 => {
            if header.header_size < 68 {
                issues.push(format!(
                    "Invalid header size {} for v3 (expected >= 68)",
                    header.header_size
                ));
            }
        }
        mopaq::FormatVersion::V4 => {
            if header.header_size < 208 {
                issues.push(format!(
                    "Invalid header size {} for v4 (expected >= 208)",
                    header.header_size
                ));
            }
        }
    }

    // Check block size
    if header.block_size > 23 {
        warnings.push(format!(
            "Unusually large block size: {} (sector size: {} bytes)",
            header.block_size,
            header.sector_size()
        ));
    }

    // Check table positions
    let archive_size = header.get_archive_size();
    let hash_pos = header.get_hash_table_pos();
    let block_pos = header.get_block_table_pos();

    if hash_pos >= archive_size {
        issues.push(format!(
            "Hash table position (0x{:X}) exceeds archive size (0x{:X})",
            hash_pos, archive_size
        ));
    }

    if block_pos >= archive_size {
        issues.push(format!(
            "Block table position (0x{:X}) exceeds archive size (0x{:X})",
            block_pos, archive_size
        ));
    }
}

/// Verify the hash and block tables
fn verify_tables(
    archive: &Archive,
    issues: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let hash_table = archive
        .hash_table()
        .ok_or_else(|| anyhow::anyhow!("Hash table not loaded"))?;
    let block_table = archive
        .block_table()
        .ok_or_else(|| anyhow::anyhow!("Block table not loaded"))?;

    // Check hash table size
    if !mopaq::is_power_of_two(hash_table.size() as u32) {
        issues.push(format!(
            "Hash table size {} is not a power of 2",
            hash_table.size()
        ));
    }

    // Count hash table usage
    let mut empty_count = 0;
    let mut deleted_count = 0;
    let mut valid_count = 0;
    let mut invalid_block_refs = 0;

    for entry in hash_table.entries() {
        if entry.is_empty() {
            empty_count += 1;
        } else if entry.is_deleted() {
            deleted_count += 1;
        } else if entry.is_valid() {
            valid_count += 1;

            // Check block index
            if entry.block_index as usize >= block_table.size() {
                invalid_block_refs += 1;
            }
        }
    }

    if invalid_block_refs > 0 {
        issues.push(format!(
            "{} hash entries reference invalid block indices",
            invalid_block_refs
        ));
    }

    let usage = valid_count as f64 / hash_table.size() as f64 * 100.0;
    if usage > 90.0 {
        warnings.push(format!(
            "Hash table is {:.1}% full - may cause performance issues",
            usage
        ));
    }

    // Check block table
    let mut files_exist = 0;
    let mut orphaned_blocks = 0;

    for (i, entry) in block_table.entries().iter().enumerate() {
        if entry.exists() {
            files_exist += 1;

            // Check if this block is referenced by any hash entry
            let mut found = false;
            for hash_entry in hash_table.entries() {
                if hash_entry.is_valid() && hash_entry.block_index as usize == i {
                    found = true;
                    break;
                }
            }
            if !found {
                orphaned_blocks += 1;
            }

            // Sanity checks
            if entry.compressed_size > entry.file_size && entry.is_compressed() {
                warnings.push(format!(
                    "Block {} has compressed size > uncompressed size",
                    i
                ));
            }
        }
    }

    if orphaned_blocks > 0 {
        warnings.push(format!(
            "{} blocks exist but are not referenced by hash table",
            orphaned_blocks
        ));
    }

    println!("Table Statistics:");
    println!(
        "  Hash table: {} entries ({} valid, {} deleted, {} empty)",
        hash_table.size(),
        valid_count,
        deleted_count,
        empty_count
    );
    println!(
        "  Block table: {} entries ({} exist)",
        block_table.size(),
        files_exist
    );

    Ok(())
}

/// Verify individual files
fn verify_files(
    archive: &mut Archive,
    issues: &mut Vec<String>,
    warnings: &mut Vec<String>,
    verbose: bool,
) -> Result<()> {
    let listfile_data = archive.read_file("(listfile)")?;
    let filenames = mopaq::special_files::parse_listfile(&listfile_data)?;

    println!();
    println!("Verifying {} files...", filenames.len());

    let mut verified = 0;
    let mut missing = 0;
    let mut failed = 0;
    let mut crc_protected = 0;

    for filename in &filenames {
        if verbose {
            print!("  Checking {}... ", filename);
        }

        // Check if file exists in hash table
        match archive.find_file(filename) {
            Ok(Some(file_info)) => {
                // Check if file has CRC protection
                if file_info.has_sector_crc() {
                    crc_protected += 1;
                }

                // Try to read the file to verify it's accessible
                match archive.read_file(filename) {
                    Ok(data) => {
                        // Verify size matches
                        if data.len() != file_info.file_size as usize {
                            issues.push(format!(
                                "{}: Size mismatch (expected {}, got {})",
                                filename,
                                file_info.file_size,
                                data.len()
                            ));
                            if verbose {
                                println!("SIZE MISMATCH");
                            }
                            failed += 1;
                        } else {
                            if verbose {
                                println!("OK");
                            }
                            verified += 1;
                        }
                    }
                    Err(e) => {
                        warnings.push(format!("{}: Failed to read - {}", filename, e));
                        if verbose {
                            println!("FAILED");
                        }
                        failed += 1;
                    }
                }
            }
            Ok(None) => {
                warnings.push(format!(
                    "{}: Listed in (listfile) but not found in archive",
                    filename
                ));
                if verbose {
                    println!("NOT FOUND");
                }
                missing += 1;
            }
            Err(e) => {
                issues.push(format!("{}: Error during lookup - {}", filename, e));
                if verbose {
                    println!("ERROR");
                }
                failed += 1;
            }
        }
    }

    println!();
    println!("File Verification:");
    println!("  Verified: {}", verified);
    println!("  Missing: {}", missing);
    println!("  Failed: {}", failed);
    println!("  CRC protected: {}", crc_protected);

    Ok(())
}
