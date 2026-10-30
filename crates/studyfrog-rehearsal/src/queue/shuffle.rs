//! Seeded shuffling: SplitMix64 and Fisher-Yates, so runs are reproducible without a `rand`
//! dependency (P10).

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Uniform-enough value in `0..n` (multiply-shift; the bias is below 2^-32 for any realistic `n`).
fn below(state: &mut u64, n: usize) -> usize {
    ((u128::from(splitmix64(state)) * n as u128) >> 64) as usize
}

/// Shuffle in place; the same seed and input length always give the same permutation.
pub fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed;
    for i in (1..items.len()).rev() {
        items.swap(i, below(&mut state, i + 1));
    }
}
