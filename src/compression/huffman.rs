//! Huffman compression implementation for MPQ archives

use super::{CompressionError, CompressionResult, CompressionType, Compressor, Decompressor};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Compresses data using Huffman encoding
pub fn compress_huffman(data: &[u8]) -> CompressionResult<Vec<u8>> {
    // Implement Huffman compression

    // Step 1: Build frequency table
    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] += 1;
    }

    // Step 2: Build Huffman tree
    let tree = build_huffman_tree(&freq)?;

    // Step 3: Generate code table
    let code_table = generate_code_table(&tree)?;

    // Step 4: Encode the data
    let mut result = Vec::new();

    // Add the frequency table to the result (for decompression)
    for &f in &freq {
        result.extend_from_slice(&f.to_le_bytes());
    }

    // Add the compressed data length
    let mut bit_count = 0;
    for &b in data {
        bit_count += code_table[b as usize].len();
    }
    let byte_count = (bit_count + 7) / 8;
    result.extend_from_slice(&(byte_count as u32).to_le_bytes());

    // Add the original data length
    result.extend_from_slice(&(data.len() as u32).to_le_bytes());

    // Add the compressed data
    let mut current_byte = 0u8;
    let mut bits_used = 0;

    for &b in data {
        let code = &code_table[b as usize];
        for &bit in code {
            if bit {
                current_byte |= 1 << bits_used;
            }
            bits_used += 1;

            if bits_used == 8 {
                result.push(current_byte);
                current_byte = 0;
                bits_used = 0;
            }
        }
    }

    // Add the last byte if there are bits remaining
    if bits_used > 0 {
        result.push(current_byte);
    }

    Ok(result)
}

/// Decompresses data using Huffman encoding
pub fn decompress_huffman(data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
    if data.len() < 1024 + 8 {
        // 256*4 for frequency table + 8 for size fields
        return Err(CompressionError::InvalidData(format!(
            "Data too small for Huffman header: {} bytes",
            data.len()
        )));
    }

    // Step 1: Read frequency table
    let mut freq = [0u32; 256];
    for i in 0..256 {
        let offset = i * 4;
        freq[i] = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
    }

    // Step 2: Read compressed and original sizes
    let compressed_size =
        u32::from_le_bytes([data[1024], data[1025], data[1026], data[1027]]) as usize;

    let original_size =
        u32::from_le_bytes([data[1028], data[1029], data[1030], data[1031]]) as usize;

    if original_size != expected_size {
        return Err(CompressionError::InvalidData(format!(
            "Size mismatch: header says {}, expected {}",
            original_size, expected_size
        )));
    }

    // Step 3: Rebuild Huffman tree
    let tree = build_huffman_tree(&freq)?;

    // Step 4: Decode the data
    let mut result = Vec::with_capacity(original_size);
    let mut node = &tree;

    for i in 1032..data.len() {
        let byte = data[i];

        for bit in 0..8 {
            // If we've decoded enough bytes, we're done
            if result.len() >= original_size {
                break;
            }

            // Follow the tree based on the current bit
            let bit_set = (byte & (1 << bit)) != 0;

            node = if bit_set {
                match node {
                    HuffmanNode::Leaf(_) => &tree, // Shouldn't happen
                    HuffmanNode::Internal { left: _, right } => right,
                }
            } else {
                match node {
                    HuffmanNode::Leaf(_) => &tree, // Shouldn't happen
                    HuffmanNode::Internal { left, right: _ } => left,
                }
            };

            // If we've reached a leaf, output the byte
            if let HuffmanNode::Leaf(b) = node {
                result.push(*b);
                node = &tree; // Start from the root again
            }
        }
    }

    if result.len() != original_size {
        return Err(CompressionError::DecompressionFailed(format!(
            "Expected {} bytes, got {}",
            original_size,
            result.len()
        )));
    }

    Ok(result)
}

// Huffman tree node
#[derive(Debug)]
enum HuffmanNode {
    Leaf(u8),
    Internal {
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
    },
}

// For the priority queue
#[derive(Debug, Eq)]
struct WeightedNode {
    weight: u32,
    node: Box<HuffmanNode>,
}

impl Ord for WeightedNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other.weight.cmp(&self.weight)
    }
}

impl PartialOrd for WeightedNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WeightedNode {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

// Build a Huffman tree from frequency table
fn build_huffman_tree(freq: &[u32; 256]) -> CompressionResult<HuffmanNode> {
    let mut pq = BinaryHeap::new();

    // Add all symbols with non-zero frequency to the queue
    for (i, &f) in freq.iter().enumerate() {
        if f > 0 {
            pq.push(WeightedNode {
                weight: f,
                node: Box::new(HuffmanNode::Leaf(i as u8)),
            });
        }
    }

    // Special case: no symbols
    if pq.is_empty() {
        return Err(CompressionError::InvalidData(
            "No non-zero frequencies".to_string(),
        ));
    }

    // Special case: only one symbol
    if pq.len() == 1 {
        let node = pq.pop().unwrap();
        return Ok(*node.node);
    }

    // Build the tree
    while pq.len() > 1 {
        let left = pq.pop().unwrap();
        let right = pq.pop().unwrap();

        pq.push(WeightedNode {
            weight: left.weight + right.weight,
            node: Box::new(HuffmanNode::Internal {
                left: left.node,
                right: right.node,
            }),
        });
    }

    // Return the root node
    Ok(*pq.pop().unwrap().node)
}

// Generate a code table from a Huffman tree
fn generate_code_table(tree: &HuffmanNode) -> CompressionResult<Vec<Vec<bool>>> {
    let mut table = vec![Vec::new(); 256];
    let mut code = Vec::new();

    fn traverse(node: &HuffmanNode, code: &mut Vec<bool>, table: &mut Vec<Vec<bool>>) {
        match node {
            HuffmanNode::Leaf(b) => {
                table[*b as usize] = code.clone();
            }
            HuffmanNode::Internal { left, right } => {
                // Traverse left (0)
                code.push(false);
                traverse(left, code, table);
                code.pop();

                // Traverse right (1)
                code.push(true);
                traverse(right, code, table);
                code.pop();
            }
        }
    }

    traverse(tree, &mut code, &mut table);
    Ok(table)
}

/// Huffman compressor implementation
pub struct HuffmanCompressor;

impl Compressor for HuffmanCompressor {
    fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        compress_huffman(data)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Huffman
    }
}

/// Huffman decompressor implementation
pub struct HuffmanDecompressor;

impl Decompressor for HuffmanDecompressor {
    fn decompress(&self, data: &[u8], expected_size: usize) -> CompressionResult<Vec<u8>> {
        decompress_huffman(data, expected_size)
    }

    fn compression_type(&self) -> CompressionType {
        CompressionType::Huffman
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_roundtrip() {
        // Test with various data
        let test_cases = [
            b"This is a test of Huffman compression. It should compress well and decompress back to the original.",
            b"aaaaaaaaaaaaaaabbbbbbbbbbccccccddddeeeeeffffffffffffffffffffgggggggggghhhhhhhhhhiiiiiiiiiijjjjjjjjj", // Highly compressible
            &[0u8; 99], // All zeros
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], // Small range
        ];

        for &original in &test_cases {
            // Compress
            let compressed = compress_huffman(original).expect("Compression failed");

            // Decompress
            let decompressed =
                decompress_huffman(&compressed, original.len()).expect("Decompression failed");

            // Check that we got the original data back
            assert_eq!(decompressed, original);
        }
    }
}
