//! Create command implementation

use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use mopaq::{compression::flags, ArchiveBuilder, FormatVersion, ListfileOption};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Options for creating an archive
pub struct CreateOptions {
    pub version: FormatVersion,
    pub compression: u8,
    pub block_size: u16,
    pub listfile: ListfileOption,
    pub recursive: bool,
    pub follow_symlinks: bool,
    pub ignore_patterns: Vec<String>,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            version: FormatVersion::V1,
            compression: flags::ZLIB,
            block_size: 3, // 4KB sectors
            listfile: ListfileOption::Generate,
            recursive: true,
            follow_symlinks: false,
            ignore_patterns: vec![
                ".git".to_string(),
                ".svn".to_string(),
                "Thumbs.db".to_string(),
                ".DS_Store".to_string(),
            ],
        }
    }
}

/// Create a new MPQ archive
pub fn create(archive_path: &str, source: &str, options: CreateOptions) -> Result<()> {
    // Validate inputs
    let source_path = Path::new(source);
    if !source_path.exists() {
        return Err(anyhow::anyhow!("Source path '{}' does not exist", source));
    }

    // Check if archive already exists
    if Path::new(archive_path).exists() {
        println!(
            "{} Archive '{}' already exists",
            "Warning:".yellow().bold(),
            archive_path
        );
        print!("Overwrite? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    println!(
        "{} MPQ archive: {}",
        "Creating".green().bold(),
        archive_path.cyan()
    );
    println!("  {}: {}", "Source".bold(), source_path.display());
    println!(
        "  {}: v{} ({})",
        "Format".bold(),
        options.version as u16 + 1,
        format_version_name(options.version)
    );
    println!(
        "  {}: {}",
        "Compression".bold(),
        compression_name(options.compression)
    );
    println!(
        "  {}: {} bytes",
        "Sector size".bold(),
        512 << options.block_size
    );
    println!();

    // Collect files
    let files = if source_path.is_file() {
        // Single file
        vec![FileEntry {
            path: source_path.to_path_buf(),
            archive_name: source_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }]
    } else {
        // Directory
        collect_files(source_path, &options)?
    };

    if files.is_empty() {
        return Err(anyhow::anyhow!("No files found to add to archive"));
    }

    println!(
        "{} {} files",
        "Found".green(),
        files.len().to_string().bright_blue()
    );

    // Create progress bar
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
            )?
            .progress_chars("#>-"),
    );

    // Build archive
    let mut builder = ArchiveBuilder::new()
        .version(options.version)
        .block_size(options.block_size)
        .default_compression(options.compression)
        .listfile_option(options.listfile);

    // Add files
    let start_time = std::time::Instant::now();
    let mut total_size = 0u64;

    for file_entry in &files {
        pb.set_message(format!("Adding {}", file_entry.archive_name));

        // Get file size for statistics
        if let Ok(metadata) = file_entry.path.metadata() {
            total_size += metadata.len();
        }

        // Add file to builder
        builder = builder.add_file(&file_entry.path, &file_entry.archive_name);

        pb.inc(1);
    }

    pb.finish_and_clear();
    println!("{} Building archive...", "📦".blue());

    // Build the archive
    builder
        .build(archive_path)
        .context("Failed to build archive")?;

    // Get final archive size
    let archive_metadata = fs::metadata(archive_path)?;
    let compressed_size = archive_metadata.len();

    let elapsed = start_time.elapsed();

    // Print summary
    println!();
    println!("{}", "Archive created successfully!".green().bold());
    println!();
    println!("{}", "Summary:".bold().underline());
    println!(
        "  {}: {}",
        "Files added".bold(),
        files.len().to_string().green()
    );
    println!("  {}: {}", "Total size".bold(), format_size(total_size));
    println!(
        "  {}: {} ({:.1}% ratio)",
        "Archive size".bold(),
        format_size(compressed_size),
        if total_size > 0 {
            100.0 * compressed_size as f64 / total_size as f64
        } else {
            100.0
        }
    );
    println!("  {}: {:.2}s", "Time elapsed".bold(), elapsed.as_secs_f64());

    Ok(())
}

/// Collect files from a directory
fn collect_files(source_dir: &Path, options: &CreateOptions) -> Result<Vec<FileEntry>> {
    let mut files = Vec::new();

    let walker = if options.recursive {
        WalkDir::new(source_dir).follow_links(options.follow_symlinks)
    } else {
        WalkDir::new(source_dir)
            .max_depth(1)
            .follow_links(options.follow_symlinks)
    };

    for entry in walker {
        let entry = entry?;

        // Skip directories
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // Check ignore patterns
        if should_ignore(path, &options.ignore_patterns) {
            continue;
        }

        // Calculate archive name (relative path with backslashes)
        let relative_path = path
            .strip_prefix(source_dir)?
            .to_string_lossy()
            .replace('/', "\\");

        files.push(FileEntry {
            path: path.to_path_buf(),
            archive_name: relative_path,
        });
    }

    // Sort files for consistent ordering
    files.sort_by(|a, b| a.archive_name.cmp(&b.archive_name));

    Ok(files)
}

/// Check if a path should be ignored
fn should_ignore(path: &Path, patterns: &[String]) -> bool {
    for pattern in patterns {
        if path.to_string_lossy().contains(pattern) {
            return true;
        }
    }
    false
}

#[derive(Debug)]
struct FileEntry {
    path: PathBuf,
    archive_name: String,
}

/// Format file size
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Get human-readable format version name
fn format_version_name(version: FormatVersion) -> &'static str {
    match version {
        FormatVersion::V1 => "Original/Vanilla",
        FormatVersion::V2 => "Burning Crusade",
        FormatVersion::V3 => "Cataclysm Beta",
        FormatVersion::V4 => "Cataclysm+",
    }
}

/// Get human-readable compression name
fn compression_name(compression: u8) -> &'static str {
    match compression {
        0 => "None",
        flags::HUFFMAN => "Huffman",
        flags::ZLIB => "Zlib/Deflate",
        flags::PKWARE => "PKWare DCL",
        flags::BZIP2 => "BZip2",
        flags::SPARSE => "Sparse/RLE",
        flags::LZMA => "LZMA",
        _ => "Multiple/Unknown",
    }
}
