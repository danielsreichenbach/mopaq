//! Special MPQ files handling: (listfile), (attributes), (signature), etc.

use crate::{Error, Result};

/// Parse a (listfile) into individual filenames
///
/// The (listfile) format supports:
/// - One filename per line
/// - Comments starting with ';' or '#'
/// - Optional file metadata after ';' on each line
/// - Empty lines are ignored
pub fn parse_listfile(data: &[u8]) -> Result<Vec<String>> {
    let content = match std::str::from_utf8(data) {
        Ok(s) => s.to_string(),
        Err(_) => {
            // Try lossy conversion for files with invalid UTF-8
            log::warn!("(listfile) contains invalid UTF-8, using lossy conversion");
            String::from_utf8_lossy(data).into_owned()
        }
    };

    let files: Vec<String> = content
        .lines()
        .filter_map(|line| {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                return None;
            }

            // Handle semicolon-separated format (filename;metadata)
            let filename = if let Some(pos) = line.find(';') {
                line[..pos].trim()
            } else {
                line
            };

            // Skip if the result is empty
            if filename.is_empty() {
                None
            } else {
                Some(filename.to_string())
            }
        })
        .collect();

    log::debug!("Parsed {} files from (listfile)", files.len());
    Ok(files)
}

/// Information about a special file
#[derive(Debug, Clone)]
pub struct SpecialFileInfo {
    /// The filename (e.g., "(listfile)")
    pub name: &'static str,
    /// Whether this file is encrypted by default
    pub encrypted: bool,
    /// Whether this file is compressed by default
    pub compressed: bool,
}

/// Get information about known special files
pub fn get_special_file_info(filename: &str) -> Option<SpecialFileInfo> {
    match filename {
        "(listfile)" => Some(SpecialFileInfo {
            name: "(listfile)",
            encrypted: false,
            compressed: false,
        }),
        "(attributes)" => Some(SpecialFileInfo {
            name: "(attributes)",
            encrypted: false,
            compressed: true,
        }),
        "(signature)" => Some(SpecialFileInfo {
            name: "(signature)",
            encrypted: false,
            compressed: false,
        }),
        "(user data)" => Some(SpecialFileInfo {
            name: "(user data)",
            encrypted: false,
            compressed: false,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_listfile() {
        let content = b"file1.txt\nfile2.dat\nfile3.bin";
        let files = parse_listfile(content).unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], "file1.txt");
        assert_eq!(files[1], "file2.dat");
        assert_eq!(files[2], "file3.bin");
    }

    #[test]
    fn test_parse_listfile_with_comments() {
        let content = b"; This is a comment\n\
                       file1.txt\n\
                       # Another comment\n\
                       file2.dat\n\
                       ; file3.txt - commented out\n\
                       file4.bin";

        let files = parse_listfile(content).unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], "file1.txt");
        assert_eq!(files[1], "file2.dat");
        assert_eq!(files[2], "file4.bin");
    }

    #[test]
    fn test_parse_listfile_with_metadata() {
        let content = b"file1.txt;12345\n\
                       file2.dat;67890;extra data\n\
                       file3.bin";

        let files = parse_listfile(content).unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], "file1.txt");
        assert_eq!(files[1], "file2.dat");
        assert_eq!(files[2], "file3.bin");
    }

    #[test]
    fn test_parse_empty_listfile() {
        let content = b"";
        let files = parse_listfile(content).unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_parse_listfile_only_comments() {
        let content = b"; Comment 1\n# Comment 2\n; Comment 3";
        let files = parse_listfile(content).unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_parse_listfile_with_empty_lines() {
        let content = b"file1.txt\n\n\nfile2.dat\n\n";
        let files = parse_listfile(content).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], "file1.txt");
        assert_eq!(files[1], "file2.dat");
    }

    #[test]
    fn test_parse_listfile_with_whitespace() {
        let content = b"  file1.txt  \n\tfile2.dat\t\n   file3.bin   ;   metadata   ";
        let files = parse_listfile(content).unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], "file1.txt");
        assert_eq!(files[1], "file2.dat");
        assert_eq!(files[2], "file3.bin");
    }

    #[test]
    fn test_special_file_info() {
        assert!(get_special_file_info("(listfile)").is_some());
        assert!(get_special_file_info("(attributes)").is_some());
        assert!(get_special_file_info("(signature)").is_some());
        assert!(get_special_file_info("(user data)").is_some());
        assert!(get_special_file_info("regular_file.txt").is_none());

        let info = get_special_file_info("(attributes)").unwrap();
        assert!(!info.encrypted);
        assert!(info.compressed);
    }
}
