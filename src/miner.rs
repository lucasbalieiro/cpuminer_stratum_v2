use crate::utils::{compute_target, sha256d};

pub struct BlockHeader {
    pub version: u8,
    pub previous_block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
}

impl BlockHeader {
    pub fn hash(&self) -> [u8; 32] {
        let mut header_bytes = Vec::new();
        header_bytes.extend_from_slice(&(self.version as u32).to_le_bytes());
        header_bytes.extend_from_slice(&self.previous_block_hash);
        header_bytes.extend_from_slice(&self.merkle_root);
        header_bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        header_bytes.extend_from_slice(&self.bits.to_le_bytes());
        header_bytes.extend_from_slice(&self.nonce.to_le_bytes());
        sha256d(&header_bytes)
    }
}

pub struct Miner;

impl Miner {
    pub fn mine(mut block: BlockHeader) -> (u32, [u8; 32]) {
        let target = compute_target(block.bits);
        loop {
            let block_hash = block.hash();
            // Bitcoin hashes and targets are compared as 256-bit little-endian integers
            // To be explicit, compare from least significant byte to most
            if block_hash.iter().rev().cmp(target.iter()) != std::cmp::Ordering::Greater {
                return (block.nonce, block_hash);
            }
            block.nonce = block.nonce.wrapping_add(1);
        }
    }
}
