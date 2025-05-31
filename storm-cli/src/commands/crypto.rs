//! Cryptography test utilities

use anyhow::Result;
use colored::Colorize;
use mopaq::crypto::{decrypt_block, encrypt_block, hash_string, hash_type};

use crate::GLOBAL_OPTS;

/// Test cryptographic functions
pub fn test(test_name: Option<&str>) -> Result<()> {
    let _global_opts = GLOBAL_OPTS.get().expect("Global options not set");

    if let Some(name) = test_name {
        match name {
            "hash" => test_hash_functions()?,
            "encrypt" => test_encryption()?,
            "decrypt" => test_decryption()?,
            _ => anyhow::bail!(
                "Unknown test: {}. Available tests: hash, encrypt, decrypt",
                name
            ),
        }
    } else {
        // Run all tests
        println!("{}", "Running all crypto tests...".bold());
        println!();

        test_hash_functions()?;
        println!();
        test_encryption()?;
        println!();
        test_decryption()?;
    }

    Ok(())
}

fn test_hash_functions() -> Result<()> {
    println!("{}", "Hash Function Tests".cyan());
    println!("{}", "=".repeat(50));

    let test_strings = [
        "(listfile)",
        "(attributes)",
        "(signature)",
        "war3map.j",
        "Scripts\\Common.j",
        "Units\\NightElf\\Wisp\\Wisp.mdx",
    ];

    for test_str in &test_strings {
        println!("\n{}: {}", "Input".bold(), test_str);
        println!(
            "  Table offset: {:#010x}",
            hash_string(test_str, hash_type::TABLE_OFFSET)
        );
        println!(
            "  Name A:       {:#010x}",
            hash_string(test_str, hash_type::NAME_A)
        );
        println!(
            "  Name B:       {:#010x}",
            hash_string(test_str, hash_type::NAME_B)
        );
        println!(
            "  File key:     {:#010x}",
            hash_string(test_str, hash_type::FILE_KEY)
        );
    }

    println!("\n{} Hash tests completed", "✓".green());
    Ok(())
}

fn test_encryption() -> Result<()> {
    println!("{}", "Encryption Tests".cyan());
    println!("{}", "=".repeat(50));

    // Test data - use u32 values
    let test_data: Vec<u32> = vec![0x48656c6c, 0x6f2c2057, 0x6f726c64, 0x21212121];
    let key = 0x12345678;

    println!("Original data: {:08x?}", test_data);
    println!("Key: {:#010x}", key);

    // Encrypt data
    let mut encrypted = test_data.clone();
    encrypt_block(&mut encrypted, key);

    println!("Encrypted: {:08x?}", encrypted);

    // Decrypt to verify
    let mut decrypted = encrypted.clone();
    decrypt_block(&mut decrypted, key);

    let matches = decrypted == test_data;
    if matches {
        println!("{} Encryption/Decryption cycle successful", "✓".green());
    } else {
        println!("{} Encryption/Decryption cycle failed", "✗".red());
        println!("Decrypted: {:08x?}", decrypted);
        anyhow::bail!("Encryption test failed");
    }

    Ok(())
}

fn test_decryption() -> Result<()> {
    println!("{}", "Decryption Tests".cyan());
    println!("{}", "=".repeat(50));

    // Test known encrypted values
    println!("Testing decryption with known values...");

    // Test file key generation
    let filenames = ["war3map.j", "(listfile)", "(attributes)"];
    for filename in &filenames {
        let file_key = hash_string(filename, hash_type::FILE_KEY);
        println!("File key for '{}': {:#010x}", filename, file_key);
    }

    println!("\n{} Decryption tests completed", "✓".green());
    Ok(())
}
