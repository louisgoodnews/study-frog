//! Content hash for change detection. In-house (P10): 8 bytes per step, strong final mix.
//! Not cryptographic; it only decides whether a file with a changed mtime really changed.

const K: u64 = 0x517c_c1b7_2722_0a95;

fn mix(h: u64, word: u64) -> u64 {
    (h.rotate_left(5) ^ word).wrapping_mul(K)
}

/// 64-bit hash of `data`.
pub fn hash_bytes(data: &[u8]) -> u64 {
    let (words, rest) = data.as_chunks::<8>();
    let h = words
        .iter()
        .fold(data.len() as u64, |h, w| mix(h, u64::from_le_bytes(*w)));
    let tail = rest.iter().fold(0_u64, |acc, b| acc << 8 | u64::from(*b));
    let h = mix(h, tail);
    let h = (h ^ (h >> 32)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    h ^ (h >> 29)
}

/// Fold another value into a running hash (order-sensitive).
pub fn combine(h: u64, x: u64) -> u64 {
    mix(h, x).rotate_left(23) ^ x
}
