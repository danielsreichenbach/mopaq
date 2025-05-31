use crate::{OutputFormat, GLOBAL_OPTS};
use colored::*;
use mopaq::{Archive, ArchiveInfo, SignatureStatus};
use serde::Serialize;
use std::io;

/// Print output according to the global format settings
#[allow(dead_code)]
pub fn print_output<T: Serialize>(data: &T) -> Result<(), io::Error> {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");

    if opts.quiet {
        return Ok(());
    }

    match opts.output {
        OutputFormat::Json => print_json(data),
        OutputFormat::Csv => print_csv(data),
        OutputFormat::Text => Ok(()), // Text output is handled by individual commands
    }
}

/// Print JSON output
pub fn print_json<T: Serialize>(data: &T) -> Result<(), io::Error> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

/// Print CSV output (simplified - you might want to use the csv crate)
#[allow(dead_code)]
pub fn print_csv<T: Serialize>(data: &T) -> Result<(), io::Error> {
    // This is a simplified version - for real CSV output, use the csv crate
    let json_value = serde_json::to_value(data)?;

    if let serde_json::Value::Array(arr) = json_value {
        // Print headers (assuming all objects have same fields)
        if let Some(serde_json::Value::Object(obj)) = arr.first() {
            println!("{}", obj.keys().cloned().collect::<Vec<_>>().join(","));
        }

        // Print rows
        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                let values: Vec<String> = obj
                    .values()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    })
                    .collect();
                println!("{}", values.join(","));
            }
        }
    }

    Ok(())
}

/// Print verbose message (only if verbose mode is on)
#[allow(dead_code)]
pub fn verbose_println(level: u8, message: &str) {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");

    if !opts.quiet && opts.verbose >= level {
        eprintln!("{} {}", "[VERBOSE]".dimmed(), message);
    }
}

/// Check if we should use color
#[allow(dead_code)]
pub fn use_color() -> bool {
    let opts = GLOBAL_OPTS.get().expect("Global options not initialized");
    !opts.no_color && opts.output == OutputFormat::Text
}

/// Print archive information
pub fn print_archive_info(archive: &mut Archive, format: OutputFormat) -> Result<(), io::Error> {
    let info = archive.get_info().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to get archive info: {}", e),
        )
    })?;

    match format {
        OutputFormat::Text => print_archive_info_text(&info),
        OutputFormat::Json => print_archive_info_json(&info),
        OutputFormat::Csv => print_archive_info_csv(&info),
    }
}

fn print_archive_info_text(info: &ArchiveInfo) -> Result<(), io::Error> {
    println!("{}", "Archive Information".bold());
    println!("{}", "=".repeat(60));

    // Basic information
    println!("{}: {}", "Archive path".bright_cyan(), info.path.display());
    println!(
        "{}: {} bytes",
        "Archive size".bright_cyan(),
        format_size(info.file_size)
    );
    println!(
        "{}: 0x{:X}",
        "Archive offset".bright_cyan(),
        info.archive_offset
    );
    println!(
        "{}: v{}",
        "Format version".bright_cyan(),
        info.format_version as u16 + 1
    );

    // File statistics
    println!("\n{}", "File Statistics".bold());
    println!("{}", "-".repeat(60));
    println!("{}: {}", "Number of files".bright_cyan(), info.file_count);
    println!(
        "{}: {}",
        "Maximum file capacity".bright_cyan(),
        info.max_file_count
    );
    println!(
        "{}: {} bytes",
        "Sector size".bright_cyan(),
        info.sector_size
    );

    // Security information
    println!("\n{}", "Security Information".bold());
    println!("{}", "-".repeat(60));
    println!(
        "{}: {}",
        "Encrypted".bright_cyan(),
        if info.is_encrypted {
            "Yes".red()
        } else {
            "No".green()
        }
    );
    println!(
        "{}: {}",
        "Digital signature".bright_cyan(),
        format_signature_status(&info.signature_status)
    );

    // Table information
    println!("\n{}", "Table Information".bold());
    println!("{}", "-".repeat(60));
    print_table_info("Hash table", &info.hash_table_info);
    print_table_info("Block table", &info.block_table_info);

    if let Some(het_info) = &info.het_table_info {
        print_table_info("HET table", het_info);
    }
    if let Some(bet_info) = &info.bet_table_info {
        print_table_info("BET table", bet_info);
    }
    if let Some(hi_block_info) = &info.hi_block_table_info {
        print_table_info("Hi-block table", hi_block_info);
    }

    // Special files
    println!("\n{}", "Special Files".bold());
    println!("{}", "-".repeat(60));
    println!(
        "{}: {}",
        "(attributes) file".bright_cyan(),
        if info.has_attributes {
            "Present".green()
        } else {
            "Not present".dimmed()
        }
    );
    println!(
        "{}: {}",
        "(listfile) file".bright_cyan(),
        if info.has_listfile {
            "Present".green()
        } else {
            "Not present".dimmed()
        }
    );

    // User data (if present)
    if let Some(user_data) = &info.user_data_info {
        println!("\n{}", "User Data".bold());
        println!("{}", "-".repeat(60));
        println!(
            "{}: {} bytes",
            "Header size".bright_cyan(),
            user_data.header_size
        );
        println!(
            "{}: {} bytes",
            "Data size".bright_cyan(),
            user_data.data_size
        );
    }

    // MD5 checksums (v4 only)
    if let Some(md5_status) = &info.md5_status {
        println!("\n{}", "MD5 Checksums (v4)".bold());
        println!("{}", "-".repeat(60));
        print_md5_status("MPQ header", md5_status.header_valid);
        print_md5_status("Hash table", md5_status.hash_table_valid);
        print_md5_status("Block table", md5_status.block_table_valid);
        print_md5_status("Hi-block table", md5_status.hi_block_table_valid);
        print_md5_status("HET table", md5_status.het_table_valid);
        print_md5_status("BET table", md5_status.bet_table_valid);
    }

    Ok(())
}

fn print_table_info(name: &str, info: &mopaq::TableInfo) {
    print!("{}: ", name.bright_cyan());

    if info.failed_to_load {
        print!("{}", "Failed to load".red());
        if let Some(compressed_size) = info.compressed_size {
            print!(" (compressed: {} bytes)", compressed_size);
        }
    } else if let Some(size) = info.size {
        print!("{} entries", size);
    } else {
        print!("Unknown size");
    }

    print!(" @ 0x{:X}", info.offset);
    if !info.failed_to_load {
        if let Some(compressed_size) = info.compressed_size {
            print!(" (compressed: {} bytes)", compressed_size);
        }
    }
    println!();
}

fn print_md5_status(name: &str, valid: bool) {
    println!(
        "{}: {}",
        name.bright_cyan(),
        if valid {
            "Valid".green()
        } else {
            "Invalid".red()
        }
    );
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{}", bytes)
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

fn format_signature_status(status: &SignatureStatus) -> ColoredString {
    match status {
        SignatureStatus::None => "No signature".dimmed(),
        SignatureStatus::WeakValid => "Weak signature (Valid)".green(),
        SignatureStatus::WeakInvalid => "Weak signature (Invalid)".red(),
        SignatureStatus::StrongValid => "Strong signature (Valid)".green(),
        SignatureStatus::StrongInvalid => "Strong signature (Invalid)".red(),
        SignatureStatus::StrongNoKey => "Strong signature (No public key)".yellow(),
    }
}

fn print_archive_info_json(info: &ArchiveInfo) -> Result<(), io::Error> {
    let json_info = serde_json::json!({
        "path": info.path.display().to_string(),
        "file_size": info.file_size,
        "archive_offset": info.archive_offset,
        "format_version": info.format_version as u16 + 1,
        "file_count": info.file_count,
        "max_file_count": info.max_file_count,
        "sector_size": info.sector_size,
        "is_encrypted": info.is_encrypted,
        "has_signature": info.has_signature,
        "signature_status": format!("{:?}", info.signature_status),
        "tables": {
            "hash_table": {
                "size": info.hash_table_info.size,
                "offset": info.hash_table_info.offset,
                "compressed_size": info.hash_table_info.compressed_size,
                "failed_to_load": info.hash_table_info.failed_to_load,
            },
            "block_table": {
                "size": info.block_table_info.size,
                "offset": info.block_table_info.offset,
                "compressed_size": info.block_table_info.compressed_size,
                "failed_to_load": info.block_table_info.failed_to_load,
            },
            "het_table": info.het_table_info.as_ref().map(|t| serde_json::json!({
                "size": t.size,
                "offset": t.offset,
                "compressed_size": t.compressed_size,
                "failed_to_load": t.failed_to_load,
            })),
            "bet_table": info.bet_table_info.as_ref().map(|t| serde_json::json!({
                "size": t.size,
                "offset": t.offset,
                "compressed_size": t.compressed_size,
                "failed_to_load": t.failed_to_load,
            })),
            "hi_block_table": info.hi_block_table_info.as_ref().map(|t| serde_json::json!({
                "size": t.size,
                "offset": t.offset,
                "compressed_size": t.compressed_size,
                "failed_to_load": t.failed_to_load,
            })),
        },
        "special_files": {
            "has_attributes": info.has_attributes,
            "has_listfile": info.has_listfile,
        },
        "user_data": info.user_data_info.as_ref().map(|ud| serde_json::json!({
            "header_size": ud.header_size,
            "data_size": ud.data_size,
        })),
        "md5_status": info.md5_status.as_ref().map(|md5| serde_json::json!({
            "header_valid": md5.header_valid,
            "hash_table_valid": md5.hash_table_valid,
            "block_table_valid": md5.block_table_valid,
            "hi_block_table_valid": md5.hi_block_table_valid,
            "het_table_valid": md5.het_table_valid,
            "bet_table_valid": md5.bet_table_valid,
        })),
    });

    print_json(&json_info)
}

fn print_archive_info_csv(info: &ArchiveInfo) -> Result<(), io::Error> {
    println!("property,value");
    println!("path,{}", info.path.display());
    println!("file_size,{}", info.file_size);
    println!("archive_offset,{}", info.archive_offset);
    println!("format_version,{}", info.format_version as u16 + 1);
    println!("file_count,{}", info.file_count);
    println!("max_file_count,{}", info.max_file_count);
    println!("sector_size,{}", info.sector_size);
    println!("is_encrypted,{}", info.is_encrypted);
    println!("has_signature,{}", info.has_signature);
    println!("signature_status,{:?}", info.signature_status);
    println!("has_attributes,{}", info.has_attributes);
    println!("has_listfile,{}", info.has_listfile);
    Ok(())
}

/// Print file list
pub fn print_file_list(
    files: &[String],
    show_all: bool,
    format: OutputFormat,
) -> Result<(), io::Error> {
    match format {
        OutputFormat::Text => {
            for file in files {
                println!("{}", file);
            }
            if !show_all {
                println!("\nTotal: {} files", files.len());
            }
        }
        OutputFormat::Json => {
            print_json(&files)?;
        }
        OutputFormat::Csv => {
            println!("filename");
            for file in files {
                println!("{}", file);
            }
        }
    }
    Ok(())
}

/// Print file information
pub fn print_file_info(filename: &str, size: u64, format: OutputFormat) -> Result<(), io::Error> {
    match format {
        OutputFormat::Text => {
            println!("{}", "File Information".bold());
            println!("{}", "=".repeat(50));
            println!("Filename: {}", filename);
            println!("Size:     {} bytes", size);
        }
        OutputFormat::Json => {
            let info = serde_json::json!({
                "filename": filename,
                "size": size
            });
            print_json(&info)?;
        }
        OutputFormat::Csv => {
            println!("filename,size");
            println!("{},{}", filename, size);
        }
    }
    Ok(())
}

/// Print table data
#[allow(dead_code)]
pub fn print_table_data<T: Serialize>(data: &T, _format: OutputFormat) -> Result<(), io::Error> {
    print_output(data)
}
