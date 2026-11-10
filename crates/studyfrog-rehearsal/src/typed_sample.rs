//! Deterministic pseudo-random sampling of items for typed written recall.

use studyfrog_core::uid::Uid;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Whether an item at `uid` requires typed written recall given `seed` and `sample_pct` (0..=100).
pub fn is_typed_sample(seed: u64, uid: Uid, sample_pct: u8) -> bool {
    if sample_pct >= 100 {
        return true;
    }
    if sample_pct == 0 {
        return false;
    }
    let mixed = seed.wrapping_add((uid.0 as u64) ^ ((uid.0 >> 64) as u64));
    let mut state = mixed ^ 0x517c_c1b7_2722_0a95;
    let r = splitmix64(&mut state);
    ((r % 100) as u8) < sample_pct
}
