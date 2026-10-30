use proptest::prelude::*;

use crate::review::{Rating, rating_from_value, rating_value};

const ALL: [Rating; 4] = [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy];

#[test]
fn value_round_trips_for_every_rating() {
    for rating in ALL {
        assert_eq!(rating_from_value(rating_value(rating)), Some(rating));
    }
}

#[test]
fn ratings_are_ordered_worst_to_best() {
    assert!(ALL.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(ALL.map(rating_value), [1, 2, 3, 4]);
}

proptest! {
    #[test]
    fn values_outside_one_to_four_are_rejected(v in any::<u8>()) {
        prop_assert_eq!(rating_from_value(v).is_some(), (1..=4).contains(&v));
    }
}
