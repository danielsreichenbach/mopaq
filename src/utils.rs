//! Utility functions for working with MPQ archives

/// Calculate the size of a table in bytes
pub fn calculate_table_size(entry_count: u32, entry_size: u32) -> u64 {
    entry_count as u64 * entry_size as u64
}

/// Calculate a suitable hash table size (power of 2)
pub fn calculate_hash_table_size(file_count: u32) -> u32 {
    // Get the next power of 2 that's at least twice the file count
    let mut size = 1;
    while size < file_count * 2 {
        size *= 2;
    }
    size
}

/// Calculate the sector count for a file
pub fn calculate_sector_count(file_size: u64, sector_size: u32) -> u32 {
    // Calculate how many sectors are needed for this file
    let full_sectors = file_size / sector_size as u64;
    let has_partial = (file_size % sector_size as u64) != 0;

    if has_partial {
        (full_sectors + 1) as u32
    } else {
        full_sectors as u32
    }
}

/// Get the sector size from the sector size shift
pub fn get_sector_size(sector_size_shift: u16) -> u32 {
    1 << sector_size_shift
}
