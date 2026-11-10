#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for deterministic random typed answer sampling.

use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::typed_sample::is_typed_sample;

#[test]
fn typed_sample_bounds_and_determinism() {
    let seed = 12345_u64;
    let uid1 = Uid(101);
    let uid2 = Uid(102);

    // 100% is always true for any item
    assert!(is_typed_sample(seed, uid1, 100));
    assert!(is_typed_sample(seed, uid2, 100));

    // 0% is always false
    assert!(!is_typed_sample(seed, uid1, 0));
    assert!(!is_typed_sample(seed, uid2, 0));

    // Deterministic for same seed and uid
    let res1 = is_typed_sample(seed, uid1, 50);
    let res2 = is_typed_sample(seed, uid1, 50);
    assert_eq!(res1, res2);

    // Different seeds yield independent selections
    let mut diff_count = 0;
    for i in 1..=50 {
        let u = Uid(i);
        if is_typed_sample(100, u, 50) != is_typed_sample(200, u, 50) {
            diff_count += 1;
        }
    }
    assert!(diff_count > 10, "independent seeds should vary");
}

#[test]
fn typed_sample_distribution_approximate() {
    let seed = 42_u64;
    let total = 200;
    let mut sampled_50 = 0;
    let mut sampled_25 = 0;

    for i in 1..=total {
        let u = Uid(i);
        if is_typed_sample(seed, u, 50) {
            sampled_50 += 1;
        }
        if is_typed_sample(seed, u, 25) {
            sampled_25 += 1;
        }
    }

    // 50% sample should be close to 100 out of 200 (within 35%..65%)
    assert!(sampled_50 > 70 && sampled_50 < 130);
    // 25% sample should be close to 50 out of 200 (within 15%..35%)
    assert!(sampled_25 > 30 && sampled_25 < 70);
}
