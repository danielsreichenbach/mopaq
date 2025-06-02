//! Example: Verify digital signatures in MPQ archives

use mopaq::{Archive, SignatureStatus};
use std::env;
use std::process;

fn main() -> mopaq::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <archive.mpq>", args[0]);
        process::exit(1);
    }

    let archive_path = &args[1];

    // Open the archive
    let mut archive = Archive::open(archive_path)?;

    // Get archive info (which includes signature verification)
    let info = archive.get_info()?;

    println!("Archive: {}", archive_path);
    println!("Has signature: {}", info.has_signature);

    match info.signature_status {
        SignatureStatus::None => {
            println!("Signature status: No signature present");
        }
        SignatureStatus::WeakValid => {
            println!("Signature status: ✓ Valid weak signature (512-bit RSA with MD5)");
            println!("The archive has not been modified since signing.");
        }
        SignatureStatus::WeakInvalid => {
            println!("Signature status: ✗ Invalid weak signature");
            println!("WARNING: The archive has been modified after signing!");
        }
        SignatureStatus::StrongValid => {
            println!("Signature status: ✓ Valid strong signature (2048-bit RSA with SHA-1)");
            println!("The archive has not been modified since signing.");
        }
        SignatureStatus::StrongInvalid => {
            println!("Signature status: ✗ Invalid strong signature");
            println!("WARNING: The archive has been modified after signing!");
        }
        SignatureStatus::StrongNoKey => {
            println!("Signature status: Strong signature present but not supported");
            println!("(Strong signature verification is not yet implemented)");
        }
    }

    // If signature exists, show more details
    if info.has_signature {
        if let Some(sig_info) = archive.find_file("(signature)")? {
            println!("\nSignature file details:");
            println!("  Position: 0x{:08X}", sig_info.file_pos);
            println!("  Size: {} bytes", sig_info.file_size);

            // For weak signatures, the size should be 64 bytes
            if sig_info.file_size == 64 {
                println!("  Type: Weak signature (512-bit RSA)");
            } else {
                println!("  Type: Unknown or strong signature");
            }
        }
    }

    Ok(())
}
