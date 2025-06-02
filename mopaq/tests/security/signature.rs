//! Tests for digital signature verification

use mopaq::{Archive, SignatureStatus};
use std::path::Path;

#[test]
#[ignore = "Requires test archive with valid weak signature"]
fn test_weak_signature_verification() {
    // This test requires a real MPQ archive with a valid weak signature
    // Such archives can be found in older Blizzard games
    let test_archive = "test-data/signed/weak_signature.mpq";

    if !Path::new(test_archive).exists() {
        eprintln!("Skipping test: {} not found", test_archive);
        return;
    }

    let mut archive = Archive::open(test_archive).expect("Failed to open test archive");

    let info = archive.get_info().expect("Failed to get archive info");

    assert!(info.has_signature, "Archive should have a signature");
    assert_eq!(
        info.signature_status,
        SignatureStatus::WeakValid,
        "Weak signature should be valid"
    );
}

#[test]
fn test_no_signature() {
    // Create a simple archive without signature
    use mopaq::{ArchiveBuilder, FormatVersion};
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let archive_path = dir.path().join("unsigned.mpq");

    ArchiveBuilder::new()
        .version(FormatVersion::V1)
        .add_file_data(b"test data".to_vec(), "test.txt")
        .build(&archive_path)
        .expect("Failed to create archive");

    let mut archive = Archive::open(&archive_path).expect("Failed to open archive");

    let info = archive.get_info().expect("Failed to get archive info");

    assert!(!info.has_signature, "Archive should not have a signature");
    assert_eq!(
        info.signature_status,
        SignatureStatus::None,
        "Signature status should be None"
    );
}

#[test]
fn test_signature_public_key_parsing() {
    use mopaq::crypto::public_keys;

    // Test that we can load the weak public key
    let weak_key = public_keys::weak_public_key().expect("Failed to load weak public key");

    // Verify key properties
    use rsa::traits::PublicKeyParts;
    let n_bytes = weak_key.n().to_bytes_be();
    assert_eq!(n_bytes.len(), 64); // 512 bits / 8

    // Test that we can load the strong public key
    let strong_key = public_keys::strong_public_key().expect("Failed to load strong public key");

    // Verify key properties (may be 255 or 256 bytes depending on leading zeros)
    let n_bytes = strong_key.n().to_bytes_be();
    assert!(n_bytes.len() >= 255 && n_bytes.len() <= 256); // 2048 bits / 8
}

#[test]
fn test_weak_signature_parsing() {
    use mopaq::crypto::parse_weak_signature;

    // Create a mock weak signature (64 bytes)
    let signature_data = vec![0xAB; 70]; // Extra bytes to test parsing

    let parsed = parse_weak_signature(&signature_data).expect("Failed to parse weak signature");

    assert_eq!(parsed.len(), 64);
    assert_eq!(parsed, &signature_data[..64]);
}

#[test]
fn test_signature_status_enum() {
    // Ensure all SignatureStatus variants are accessible
    let statuses = vec![
        SignatureStatus::None,
        SignatureStatus::WeakValid,
        SignatureStatus::WeakInvalid,
        SignatureStatus::StrongValid,
        SignatureStatus::StrongInvalid,
        SignatureStatus::StrongNoKey,
    ];

    for status in statuses {
        // Just ensure they can be created and compared
        assert_eq!(status, status);
    }
}
