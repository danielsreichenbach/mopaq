//! Hash generation and comparison utilities

use anyhow::Result;
use colored::Colorize;
use mopaq::crypto::{hash_string, hash_type};

use crate::HashType;
use crate::GLOBAL_OPTS;

/// Generate hash values for a filename
pub fn generate(filename: &str, hash_type: Option<HashType>, all: bool) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    if all || hash_type.is_none() {
        // Generate all hash types
        println!("{}", "Hash values:".bold());
        println!(
            "  Table offset: {:#010x}",
            hash_string(filename, hash_type::TABLE_OFFSET)
        );
        println!(
            "  Name A:       {:#010x}",
            hash_string(filename, hash_type::NAME_A)
        );
        println!(
            "  Name B:       {:#010x}",
            hash_string(filename, hash_type::NAME_B)
        );
        println!(
            "  File key:     {:#010x}",
            hash_string(filename, hash_type::FILE_KEY)
        );
        println!(
            "  Key2 mix:     {:#010x}",
            hash_string(filename, hash_type::KEY2_MIX)
        );
    } else if let Some(ht) = hash_type {
        let hash_type_id = match ht {
            HashType::TableOffset => hash_type::TABLE_OFFSET,
            HashType::NameA => hash_type::NAME_A,
            HashType::NameB => hash_type::NAME_B,
            HashType::FileKey => hash_type::FILE_KEY,
            HashType::Key2Mix => hash_type::KEY2_MIX,
        };

        let value = hash_string(filename, hash_type_id);

        if global_opts.output == crate::OutputFormat::Json {
            println!(
                r#"{{"filename":"{}","type":"{:?}","value":"0x{:08x}"}}"#,
                filename, ht, value
            );
        } else {
            println!("{:#010x}", value);
        }
    }

    Ok(())
}

/// Compare hash values for two filenames
pub fn compare(filename1: &str, filename2: &str) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    println!("{}", "Hash comparison:".bold());
    println!("\nFilenames:");
    println!("  1: {}", filename1);
    println!("  2: {}", filename2);
    println!();

    let types = [
        ("Table offset", 0),
        ("Name A", 1),
        ("Name B", 2),
        ("File key", 3),
        ("Key2 mix", 4),
    ];

    for (name, hash_type) in types {
        let hash1 = hash_string(filename1, hash_type);
        let hash2 = hash_string(filename2, hash_type);
        let matches = hash1 == hash2;

        if global_opts.verbose > 0 || matches {
            let status = if matches {
                "MATCH".green()
            } else {
                "DIFFER".red()
            };

            println!(
                "{:13} {:#010x} vs {:#010x} [{}]",
                format!("{}:", name),
                hash1,
                hash2,
                status
            );
        }
    }

    Ok(())
}

/// Generate Jenkins hash (for HET tables)
pub fn jenkins(_filename: &str) -> Result<()> {
    let global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    // TODO: Implement Jenkins hash when HET support is added
    if !global_opts.quiet {
        println!("Jenkins hash generation not yet implemented");
        println!("This will be available when HET table support is added");
    }

    Ok(())
}
