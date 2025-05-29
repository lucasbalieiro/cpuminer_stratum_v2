pub fn sha256d(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let first_hash = Sha256::digest(data);
    let second_hash = Sha256::digest(first_hash);

    let mut result: [u8; 32] = [0u8; 32];
    result.copy_from_slice(&second_hash);
    result
}

pub fn compute_merkle_root(transactions: &[&[u8]]) -> [u8; 32] {
    if transactions.is_empty() {
        return [0u8; 32];
    }

    let mut hashes: Vec<[u8; 32]> = transactions.iter().map(|tx| sha256d(tx)).collect();

    while hashes.len() > 1 {
        let mut new_hashes = Vec::new();
        let mut i = 0;
        while i < hashes.len() {
            let left = hashes[i];
            let right = if i + 1 < hashes.len() {
                hashes[i + 1]
            } else {
                // Duplicate the last hash if odd number of elements
                hashes[i]
            };
            let combined = [left, right].concat();
            new_hashes.push(sha256d(&combined));
            i += 2;
        }
        hashes = new_hashes;
    }

    hashes[0]
}

pub fn compute_target(bits: u32) -> [u8; 32] {
    // Bitcoin's compact encoding: https://en.bitcoin.it/wiki/Difficulty
    let exponent = (bits >> 24) as usize;
    let mut coefficient = bits & 0x007fffff;
    let mut target = [0u8; 32];

    if exponent <= 3 {
        coefficient >>= 8 * (3 - exponent);
        target[31] = (coefficient & 0xff) as u8;
        if exponent > 1 {
            target[30] = ((coefficient >> 8) & 0xff) as u8;
        }
        if exponent > 2 {
            target[29] = ((coefficient >> 16) & 0xff) as u8;
        }
    } else {
        let idx = 32 - exponent;
        if idx < 32 {
            target[idx] = ((coefficient >> 16) & 0xff) as u8;
        }
        if idx + 1 < 32 {
            target[idx + 1] = ((coefficient >> 8) & 0xff) as u8;
        }
        if idx + 2 < 32 {
            target[idx + 2] = (coefficient & 0xff) as u8;
        }
    }
    target
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sha256d() {
        let input = b"hello world";
        let expected_output = [
            0xbc, 0x62, 0xd4, 0xb8, 0x0d, 0x9e, 0x36, 0xda, 0x29, 0xc1, 0x6c, 0x5d, 0x4d, 0x9f,
            0x11, 0x73, 0x1f, 0x36, 0x05, 0x2c, 0x72, 0x40, 0x1a, 0x76, 0xc2, 0x3c, 0x0f, 0xb5,
            0xa9, 0xb7, 0x44, 0x23,
        ];
        assert_eq!(sha256d(input), expected_output);
    }

    #[test]
    fn test_merkle_root_empty() {
        let transactions: [&[u8]; 0] = [];
        assert_eq!(compute_merkle_root(&transactions), [0u8; 32]);
    }

    #[test]
    fn test_merkle_root_single() {
        let transactions = [b"tx1".as_ref()];
        assert_eq!(compute_merkle_root(&transactions), sha256d(b"tx1"));
    }

    #[test]
    fn test_merkle_root_even() {
        let transactions = [b"tx1".as_ref(), b"tx2".as_ref()];
        let h1 = sha256d(b"tx1");
        let h2 = sha256d(b"tx2");
        let expected = sha256d(&[h1, h2].concat());
        assert_eq!(compute_merkle_root(&transactions), expected);
    }

    #[test]
    fn test_merkle_root_odd() {
        let transactions = [b"tx1".as_ref(), b"tx2".as_ref(), b"tx3".as_ref()];
        let h1 = sha256d(b"tx1");
        let h2 = sha256d(b"tx2");
        let h3 = sha256d(b"tx3");
        // Level 1: [h1, h2, h3] -> [sha256d(h1+h2), sha256d(h3+h3)]
        let l1_0 = sha256d(&[h1, h2].concat());
        let l1_1 = sha256d(&[h3, h3].concat());
        // Level 2: [l1_0, l1_1] -> sha256d(l1_0+l1_1)
        let expected = sha256d(&[l1_0, l1_1].concat());
        assert_eq!(compute_merkle_root(&transactions), expected);
    }

    #[test]
    fn test_compute_target() {
        // Test with Bitcoin's genesis block bits: 0x1d00ffff
        let bits = 0x1d00ffff;
        let target = compute_target(bits);
        // The expected target for 0x1d00ffff is:
        // 0x00000000FFFF0000000000000000000000000000000000000000000000000000
        let mut expected = [0u8; 32];
        expected[0] = 0x00;
        expected[1] = 0x00;
        expected[2] = 0x00;
        expected[3] = 0x00;
        expected[4] = 0xff;
        expected[5] = 0xff;
        // The rest remain zero
        assert_eq!(target, expected);

        // Test with a lower difficulty (higher target)
        let bits = 0x1e00ffff;
        let target = compute_target(bits);
        // The expected target for 0x1e00ffff is:
        // 0x000000FFFF000000000000000000000000000000000000000000000000000000
        let mut expected = [0u8; 32];
        expected[0] = 0x00;
        expected[1] = 0x00;
        expected[2] = 0x00;
        expected[3] = 0xff;
        expected[4] = 0xff;
        // The rest remain zero
        assert_eq!(target, expected);
    }
}
