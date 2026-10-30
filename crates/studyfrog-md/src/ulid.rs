//! ULID text codec (Crockford base32, 26 characters, 128 bits). In-house: about 30 lines beat a
//! dependency (P10).

use studyfrog_core::uid::Uid;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const LEN: usize = 26;

fn digit(byte: u8) -> Option<u128> {
    let upper = byte.to_ascii_uppercase();
    ALPHABET.iter().position(|&c| c == upper).map(|p| p as u128)
}

/// Parse a ULID string; `None` unless it is exactly 26 valid characters that fit in 128 bits.
pub fn parse_ulid(text: &str) -> Option<Uid> {
    let bytes = text.as_bytes();
    if bytes.len() != LEN || digit(bytes[0])? > 7 {
        return None;
    }
    bytes
        .iter()
        .try_fold(0_u128, |acc, &b| Some(acc << 5 | digit(b)?))
        .map(Uid)
}

/// Format as 26 uppercase characters.
pub fn format_ulid(uid: Uid) -> String {
    (0..LEN)
        .map(|i| {
            let shift = 5 * (LEN - 1 - i);
            char::from(ALPHABET[((uid.0 >> shift) & 31) as usize])
        })
        .collect()
}

/// A ULID from a millisecond timestamp (the low 48 bits are used) and 80 random bits. Ids made
/// later sort later; the caller supplies both inputs so this stays pure.
pub fn compose_ulid(timestamp_ms: u64, random: [u8; 10]) -> Uid {
    let ts = u128::from(timestamp_ms & 0xFFFF_FFFF_FFFF);
    let rnd = random
        .iter()
        .fold(0_u128, |acc, b| acc << 8 | u128::from(*b));
    Uid(ts << 80 | rnd)
}
