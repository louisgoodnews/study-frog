//! Seeded pseudo-random numbers (SplitMix64) for reproducible shuffles and generated exercises.

/// The next 64-bit value; advances `state`.
pub fn splitmix_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A number in `[0, 1)`; advances `state`.
pub fn unit_interval(state: &mut u64) -> f64 {
    (splitmix_next(state) >> 11) as f64 / (1u64 << 53) as f64
}

/// A number in `[lo, hi]`; advances `state`.
pub fn uniform(state: &mut u64, lo: f64, hi: f64) -> f64 {
    lo + (hi - lo) * unit_interval(state)
}
