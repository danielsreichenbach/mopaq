//! Listfile handling for MPQ archives
//! Handles the (listfile) special file that contains filenames

use std::io::{self, BufRead, Cursor};

use crate::error::{Error, Result};

/// Reads a listfile and returns the list of filenames
pub fn read_listfile(data: &[u8]) -> Result<Vec<String>> {
    let mut filenames = Vec::new();

    // Try to detect the format (text vs. binary)
    if is_binary_listfile(data) {
        read_binary_listfile(data, &mut filenames)?;
    } else {
        read_text_listfile(data, &mut filenames)?;
    }

    Ok(filenames)
}

/// Checks if the listfile is in binary format
fn is_binary_listfile(data: &[u8]) -> bool {
    // Binary listfiles typically start with a count or header
    // This is a simplified check - real implementation would need more analysis
    if data.len() < 4 {
        return false;
    }

    // Check for a valid count - assume binary if first 4 bytes could be a valid count
    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    // If count is reasonable and data size makes sense, assume binary
    count > 0 && count < 1_000_000 && data.len() >= (4 + count as usize)
}

/// Reads a text-format listfile
fn read_text_listfile(data: &[u8], filenames: &mut Vec<String>) -> Result<()> {
    let cursor = Cursor::new(data);
    let reader = io::BufReader::new(cursor);

    for line in reader.lines() {
        match line {
            Ok(filename) => {
                // Skip empty lines and comments
                if !filename.is_empty() && !filename.starts_with('#') {
                    filenames.push(filename);
                }
            }
            Err(e) => {
                return Err(Error::IoError(e));
            }
        }
    }

    Ok(())
}

/// Reads a binary-format listfile
fn read_binary_listfile(data: &[u8], filenames: &mut Vec<String>) -> Result<()> {
    if data.len() < 4 {
        return Err(Error::Other("Binary listfile too small".to_string()));
    }

    // Read file count
    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    // Validate count
    if count > 1_000_000 {
        return Err(Error::Other(format!("Unreasonable file count: {}", count)));
    }

    // Read each filename
    let mut offset = 4;

    for _ in 0..count {
        if offset >= data.len() {
            break;
        }

        // Read string length
        if offset + 1 > data.len() {
            break;
        }

        let str_len = data[offset] as usize;
        offset += 1;

        // Read string
        if offset + str_len > data.len() {
            break;
        }

        if let Ok(filename) = std::str::from_utf8(&data[offset..offset + str_len]) {
            filenames.push(filename.to_string());
        }

        offset += str_len;
    }

    Ok(())
}

/// Writes a list of filenames to a text listfile
pub fn write_listfile(filenames: &[String]) -> Vec<u8> {
    let mut result = Vec::new();

    for filename in filenames {
        result.extend_from_slice(filename.as_bytes());
        result.push(b'\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_text_listfile() {
        let data = b"file1.txt\nfile2.txt\n\n# Comment\ndir\\file3.txt\n";
        let filenames = read_listfile(data).unwrap();

        assert_eq!(filenames.len(), 3);
        assert_eq!(filenames[0], "file1.txt");
        assert_eq!(filenames[1], "file2.txt");
        assert_eq!(filenames[2], "dir\\file3.txt");
    }

    #[test]
    fn test_write_listfile() {
        let filenames = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "dir\\file3.txt".to_string(),
        ];

        let data = write_listfile(&filenames);
        let expected = b"file1.txt\nfile2.txt\ndir\\file3.txt\n";

        assert_eq!(&data[..], &expected[..]);
    }
}
