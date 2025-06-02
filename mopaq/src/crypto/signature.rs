//! MPQ digital signature support
//!
//! MPQ archives can contain digital signatures to verify their integrity:
//! - Weak signatures (v1+): 512-bit RSA with MD5, stored in (signature) file
//! - Strong signatures (v2+): 2048-bit RSA with SHA-1, appended after archive

use crate::{Error, Result};
use md5::{Digest, Md5};
use num_bigint::BigUint;
use num_traits::Num;
use rsa::traits::PublicKeyParts;
use rsa::{BigUint as RsaBigUint, RsaPublicKey};
use std::io::Read;

/// Weak signature size (512-bit RSA)
pub const WEAK_SIGNATURE_SIZE: usize = 64; // 512 bits / 8

/// Strong signature header
pub const STRONG_SIGNATURE_HEADER: [u8; 4] = *b"NGIS"; // "SIGN" reversed

/// Strong signature size (2048-bit RSA + 4 byte header)
pub const STRONG_SIGNATURE_SIZE: usize = 256 + 4; // 2048 bits / 8 + header

/// Signature type in the archive
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignatureType {
    /// Weak signature (512-bit RSA with MD5)
    Weak,
    /// Strong signature (2048-bit RSA with SHA-1)
    Strong,
}

/// Blizzard public keys for signature verification
pub mod public_keys {
    use super::*;

    /// Blizzard weak signature public key (512-bit)
    /// This is the well-known public key used for weak signatures
    pub const BLIZZARD_WEAK_PUBLIC_KEY_N: &str =
        "C20E0798D2889FBD71F78A37E5BCC4915C4C66EFD16AE9E27CFF68608E40C2875BE6EDC6D36134C0036837657AD78640BD0CF86FAD148B633B8044B5BA0ACC1B";

    /// Weak signature public exponent
    pub const BLIZZARD_WEAK_PUBLIC_KEY_E: u32 = 0x10001; // 65537

    /// Blizzard strong signature public key (2048-bit)
    /// This is the well-known public key used for strong signatures in WoW
    pub const BLIZZARD_STRONG_PUBLIC_KEY_N: &str =
        "9563A70764E4CCCFE006576BC7B96FAE17E996BF7352F2106D84733BFF96CBB92C7AE87823B284F3C17E25159CEF96BE66235E4D59246445B4033C1186172D79E8C2C8D32F2D18D3AD0DDE2C513C5E11643DA631B416264B36A32B8D2ED8DD848374210EE95744047FE036D0154A062ABD099B7008C6BA92C17586629B9EC8BD3E14FA682AAE0151A8FA7831FC8019C07AD5EE94D005A84D6718D3DAD024955F9DC96B6D4A819175F246ED344F0C72F72C9F60CEE5DC9C9266E6C24B0AB545A2D5491CBEEF4BD1769EC325592E7CD4B76FC1423ACB693A968972ECA80FE26FDB6B60EC5BCB5E017A0ED48A58BD77CECAF80A96854A52E064F20A6F1233DF65";

    /// Strong signature public exponent
    pub const BLIZZARD_STRONG_PUBLIC_KEY_E: u32 = 0x10001; // 65537

    /// Get the weak signature public key
    pub fn weak_public_key() -> Result<RsaPublicKey> {
        let n = RsaBigUint::from_str_radix(BLIZZARD_WEAK_PUBLIC_KEY_N, 16)
            .map_err(|e| Error::invalid_format(format!("Invalid weak key modulus: {}", e)))?;
        let e = RsaBigUint::from(BLIZZARD_WEAK_PUBLIC_KEY_E);

        RsaPublicKey::new(n, e)
            .map_err(|e| Error::invalid_format(format!("Invalid weak public key: {}", e)))
    }

    /// Get the strong signature public key
    pub fn strong_public_key() -> Result<RsaPublicKey> {
        let n = RsaBigUint::from_str_radix(BLIZZARD_STRONG_PUBLIC_KEY_N, 16)
            .map_err(|e| Error::invalid_format(format!("Invalid strong key modulus: {}", e)))?;
        let e = RsaBigUint::from(BLIZZARD_STRONG_PUBLIC_KEY_E);

        RsaPublicKey::new(n, e)
            .map_err(|e| Error::invalid_format(format!("Invalid strong public key: {}", e)))
    }
}

/// Parse weak signature from (signature) file data
pub fn parse_weak_signature(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < WEAK_SIGNATURE_SIZE {
        return Err(Error::invalid_format(format!(
            "Weak signature too small: {} bytes, expected {}",
            data.len(),
            WEAK_SIGNATURE_SIZE
        )));
    }

    // Weak signatures are just the raw 512-bit RSA signature
    Ok(data[..WEAK_SIGNATURE_SIZE].to_vec())
}

/// Verify a weak signature (512-bit RSA with MD5)
pub fn verify_weak_signature<R: Read>(
    mut reader: R,
    signature: &[u8],
    archive_size: u64,
) -> Result<bool> {
    // Get the public key
    let public_key = public_keys::weak_public_key()?;

    // Calculate MD5 hash of the archive (excluding the signature)
    let mut hasher = Md5::new();
    let mut buffer = vec![0u8; 65536]; // 64KB buffer
    let mut bytes_read = 0u64;

    // Read up to archive_size (which should exclude the signature)
    while bytes_read < archive_size {
        let to_read = ((archive_size - bytes_read) as usize).min(buffer.len());
        let n = reader.read(&mut buffer[..to_read])?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        bytes_read += n as u64;
    }

    let hash = hasher.finalize();

    // Convert signature from little-endian to big-endian
    let signature_be = reverse_bytes(signature);

    // Decrypt the signature using RSA
    let signature_int = BigUint::from_bytes_be(&signature_be);
    let n = BigUint::from_bytes_be(&public_key.n().to_bytes_be());
    let e = BigUint::from_bytes_be(&public_key.e().to_bytes_be());

    // Perform RSA operation: signature^e mod n
    let decrypted = signature_int.modpow(&e, &n);
    let decrypted_bytes = decrypted.to_bytes_be();

    // Verify PKCS#1 v1.5 padding
    verify_pkcs1_v15_md5(&decrypted_bytes, &hash)
}

/// Verify PKCS#1 v1.5 padding for MD5
fn verify_pkcs1_v15_md5(decrypted: &[u8], expected_hash: &[u8]) -> Result<bool> {
    // PKCS#1 v1.5 structure for MD5:
    // 0x00 || 0x01 || PS || 0x00 || DigestInfo
    // Where PS is padding bytes (0xFF) and DigestInfo contains the hash

    if decrypted.len() < 11 + 16 + 18 {
        // Minimum size for PKCS#1 padding + MD5
        return Ok(false);
    }

    // Check header
    if decrypted[0] != 0x00 || decrypted[1] != 0x01 {
        return Ok(false);
    }

    // Find 0x00 separator after padding
    let mut separator_pos = None;
    for i in 2..decrypted.len() {
        if decrypted[i] == 0x00 {
            separator_pos = Some(i);
            break;
        } else if decrypted[i] != 0xFF {
            return Ok(false); // Invalid padding byte
        }
    }

    let separator_pos = separator_pos
        .ok_or_else(|| Error::invalid_format("No separator found in PKCS#1 padding"))?;

    // MD5 DigestInfo (from PKCS#1)
    let md5_digest_info = [
        0x30, 0x20, // SEQUENCE, length 32
        0x30, 0x0C, // SEQUENCE, length 12
        0x06, 0x08, // OID, length 8
        0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x02, 0x05, // MD5 OID
        0x05, 0x00, // NULL
        0x04, 0x10, // OCTET STRING, length 16
    ];

    // Check DigestInfo
    let digest_start = separator_pos + 1;
    if digest_start + md5_digest_info.len() + 16 != decrypted.len() {
        return Ok(false);
    }

    if &decrypted[digest_start..digest_start + md5_digest_info.len()] != md5_digest_info {
        return Ok(false);
    }

    // Check hash
    let hash_start = digest_start + md5_digest_info.len();
    Ok(&decrypted[hash_start..] == expected_hash)
}

/// Reverse byte order (little-endian to big-endian conversion)
fn reverse_bytes(data: &[u8]) -> Vec<u8> {
    data.iter().rev().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_public_key_loading() {
        let key = public_keys::weak_public_key().unwrap();
        // Check modulus size
        let n_bytes = key.n().to_bytes_be();
        assert_eq!(n_bytes.len(), 64); // 512 bits / 8
        assert_eq!(key.e(), &RsaBigUint::from(65537u32));
    }

    #[test]
    fn test_strong_public_key_loading() {
        let key = public_keys::strong_public_key().unwrap();
        // Check modulus size (may be 255 or 256 bytes depending on leading zeros)
        let n_bytes = key.n().to_bytes_be();
        assert!(n_bytes.len() >= 255 && n_bytes.len() <= 256); // 2048 bits / 8
        assert_eq!(key.e(), &RsaBigUint::from(65537u32));
    }

    #[test]
    fn test_reverse_bytes() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let reversed = reverse_bytes(&data);
        assert_eq!(reversed, vec![0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn test_parse_weak_signature() {
        let data = vec![0xFF; 100];
        let sig = parse_weak_signature(&data).unwrap();
        assert_eq!(sig.len(), WEAK_SIGNATURE_SIZE);
        assert_eq!(sig, &data[..WEAK_SIGNATURE_SIZE]);
    }

    #[test]
    fn test_parse_weak_signature_too_small() {
        let data = vec![0xFF; 32];
        let result = parse_weak_signature(&data);
        assert!(result.is_err());
    }
}
