//! Sparse/RLE compression and decompression

use crate::{Error, Result};

/// Decompress sparse/RLE compressed data
pub(crate) fn decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    // Sparse compression is a simple RLE format
    let mut output = Vec::with_capacity(expected_size);
    let mut pos = 0;

    while pos < data.len() && output.len() < expected_size {
        // Read control byte
        let control = data[pos];
        pos += 1;

        if control == 0xFF {
            // End of stream marker
            break;
        }

        if control & 0x80 != 0 {
            // Run of zeros
            let count = (control & 0x7F) as usize;
            output.resize(output.len() + count, 0);
        } else {
            // Copy bytes
            let count = control as usize;
            if pos + count > data.len() {
                return Err(Error::compression(
                    "Sparse decompression: unexpected end of data",
                ));
            }
            output.extend_from_slice(&data[pos..pos + count]);
            pos += count;
        }
    }

    // Pad with zeros if needed
    if output.len() < expected_size {
        output.resize(expected_size, 0);
    }

    Ok(output)
}

/// Compress using sparse/RLE compression
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        // Look for runs of zeros
        let zero_start = pos;
        while pos < data.len() && data[pos] == 0 {
            pos += 1;
        }

        let mut zero_count = pos - zero_start;
        if zero_count > 0 {
            // Encode runs of zeros
            while zero_count > 0 {
                let chunk = zero_count.min(0x7F);
                output.push(0x80 | (chunk as u8));
                zero_count -= chunk;
            }
        }

        // Look for non-zero bytes
        let data_start = pos;
        while pos < data.len() && data[pos] != 0 && (pos - data_start) < 0x7F {
            pos += 1;
        }

        let data_count = pos - data_start;
        if data_count > 0 {
            // Encode literal bytes
            output.push(data_count as u8);
            output.extend_from_slice(&data[data_start..pos]);
        }
    }

    // Add end marker
    output.push(0xFF);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress() {
        // Test sparse format: [count | data] or [0x80 | zero_count]
        let compressed = vec![
            5, b'H', b'e', b'l', b'l', b'o', // 5 bytes of data
            0x85, // 5 zeros (0x80 | 5)
            5, b'W', b'o', b'r', b'l', b'd', // 5 bytes of data
            0xFF, // End marker
        ];

        let decompressed = decompress(&compressed, 15).expect("Decompression failed");
        let expected = b"Hello\0\0\0\0\0World";

        assert_eq!(decompressed, expected);
    }

    #[test]
    fn test_round_trip() {
        let original = b"Hello\0\0\0\0\0World\0\0\0!!!";

        let compressed = compress(original).expect("Compression failed");
        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_all_zeros() {
        let original = vec![0u8; 100];

        let compressed = compress(&original).expect("Compression failed");
        assert!(compressed.len() < original.len()); // Should compress well

        let decompressed = decompress(&compressed, original.len()).expect("Decompression failed");
        assert_eq!(decompressed, original);
    }
}
