//! MPQ table structures (hash, block, HET, BET)

use crate::Result;

/// Hash table entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HashEntry {
    pub name_a: u32,
    pub name_b: u32,
    pub locale: u16,
    pub platform: u16,
    pub block_index: u32,
}

/// Block table entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BlockEntry {
    pub file_pos: u32,
    pub compressed_size: u32,
    pub file_size: u32,
    pub flags: u32,
}

/// Hash table
#[derive(Debug)]
pub struct HashTable {
    entries: Vec<HashEntry>,
}

impl HashTable {
    /// Create a new hash table
    pub fn new(_size: usize) -> Result<Self> {
        todo!("Implement hash table creation")
    }

    /// Get entries
    pub fn entries(&self) -> &[HashEntry] {
        &self.entries
    }
}

/// Block table
#[derive(Debug)]
pub struct BlockTable {
    entries: Vec<BlockEntry>,
}

impl BlockTable {
    /// Create a new block table
    pub fn new(_size: usize) -> Result<Self> {
        todo!("Implement block table creation")
    }

    /// Get entries
    pub fn entries(&self) -> &[BlockEntry] {
        &self.entries
    }
}
