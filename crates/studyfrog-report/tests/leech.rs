#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Leeches and confusion pairs (STORY-0104).

use proptest::prelude::*;
use studyfrog_core::review::Rating::{self, Again, Easy, Good, Hard};
use studyfrog_report::confusion::{
    PairInput, co_lapses, confusion_pairs, jaccard, pair_rules_default, tag_set, token_set,
};
use studyfrog_report::leech::{LeechInput, LeechReason, find_leeches, leech_rules_default};

fn item(item: u64, ratings: &[Rating], total_ms: u64, stability: f32) -> LeechInput {
    LeechInput {
        item,
        group: 1,
        ratings: ratings.to_vec(),
        total_ms,
        stability: Some(stability),
    }
}

#[test]
fn each_signal_flags_an_item_and_the_strongest_leech_comes_first() {
    let items = [
        item(1, &[Good, Good, Good], 10_000, 30.0), // fine
        item(
            2,
            &[Good, Again, Good, Again, Again, Good, Again],
            12_000,
            3.0,
        ), // 4 lapses, 7 reviews, S 3
        item(3, &[Good, Good, Hard, Again, Hard], 10_000, 5.0), // a streak of 3
        item(4, &[Good, Easy], 40_000, 20.0),       // 4 x the median time
        item(
            5,
            &[Again, Again, Again, Good, Again, Good, Again, Again],
            9_000,
            2.0,
        ),
    ];
    let leeches = find_leeches(&items, &leech_rules_default());
    let ids: Vec<u64> = leeches.iter().map(|l| l.item).collect();
    assert!(!ids.contains(&1));
    let two = leeches.iter().find(|l| l.item == 2).unwrap();
    assert_eq!((two.lapses, two.streak, two.reviews), (4, 1, 7));
    assert_eq!(
        two.reasons,
        [LeechReason::RepeatedLapses, LeechReason::StagnantStability]
    );
    assert!((two.efficiency.unwrap() - 3.0 / 7.0).abs() < 1e-6);
    let three = leeches.iter().find(|l| l.item == 3).unwrap();
    assert_eq!(
        (three.streak, &three.reasons[..]),
        (3, &[LeechReason::StruggleStreak][..])
    );
    let four = leeches.iter().find(|l| l.item == 4).unwrap();
    // median of 9 000, 10 000, 10 000, 12 000, 40 000 is 10 000
    assert!((four.time_burden.unwrap() - 4.0).abs() < 1e-6);
    assert_eq!(four.reasons, [LeechReason::HighLatencyBurden]);
    assert_eq!(
        ids[0], 5,
        "five lapses (the first Again is not a lapse) and stagnant: {leeches:?}"
    );
}

#[test]
fn time_burden_is_measured_within_the_group() {
    let mut a = item(1, &[Good], 40_000, 10.0);
    a.group = 2;
    let items = [
        a,
        item(2, &[Good], 40_000, 10.0),
        item(3, &[Good], 40_000, 10.0),
    ];
    assert!(
        find_leeches(&items, &leech_rules_default()).is_empty(),
        "everyone is as slow as their stack"
    );
}

fn pair(item: u64, text: &str, tags: &[&str], links: &[u64], lapses: &[i64]) -> PairInput {
    PairInput {
        item,
        group: 1,
        tokens: token_set(text),
        tags: tag_set(tags.iter().copied()),
        links: links.to_vec(),
        lapse_days: lapses.to_vec(),
    }
}

#[test]
fn words_are_hashed_without_case_and_short_words() {
    assert_eq!(
        token_set("The Sodium, sodium of NaCl"),
        token_set("sodium nacl the")
    );
    assert!(
        (jaccard(
            &token_set("sodium chloride salt"),
            &token_set("sodium hydroxide base")
        ) - 0.2)
            .abs()
            < 1e-6
    );
    assert_eq!(jaccard(&[], &[]), 0.0);
}

#[test]
fn close_lapses_count_in_both_directions() {
    assert_eq!(co_lapses(&[1, 10, 20], &[2, 11, 40], 1), 2);
    assert_eq!(
        co_lapses(&[1, 2, 3], &[2], 1),
        1,
        "one lapse of b cannot match three of a"
    );
    assert_eq!(co_lapses(&[1], &[5], 1), 0);
}

#[test]
fn look_alike_or_linked_items_that_lapse_together_are_paired() {
    let items = [
        pair(1, "ionic bond electrons transfer", &[], &[], &[3, 9, 15]),
        pair(2, "covalent bond electrons shared", &[], &[], &[3, 10, 30]), // alike, 2 close lapses
        pair(3, "molar mass of water", &["units"], &[], &[3, 9]),          // close, no likeness
        pair(4, "avogadro number", &["units"], &[], &[4, 9]),              // shares a tag with 3
        pair(5, "unrelated words entirely", &[], &[6], &[50, 60]),
        pair(6, "different again", &[], &[], &[50, 61]), // linked from 5
        pair(7, "ionic bond electrons transfer", &[], &[], &[3]), // one lapse only
    ];
    let pairs = confusion_pairs(&items, &pair_rules_default());
    let keys: Vec<(u64, u64)> = pairs.iter().map(|p| (p.a, p.b)).collect();
    assert!(keys.contains(&(1, 2)), "{pairs:?}");
    assert!(keys.contains(&(3, 4)));
    assert!(keys.contains(&(5, 6)));
    assert!(
        !keys.contains(&(1, 3)),
        "close lapses but nothing in common"
    );
    assert!(!keys.iter().any(|k| k.0 == 7 || k.1 == 7));
    let one_two = pairs.iter().find(|p| (p.a, p.b) == (1, 2)).unwrap();
    assert!(
        (one_two.similarity - 2.0 / 6.0).abs() < 1e-6,
        "bond and electrons of six words"
    );
    assert_eq!(
        (one_two.co_lapses, one_two.linked, one_two.same_group),
        (2, false, true)
    );
    assert!(pairs.iter().find(|p| (p.a, p.b) == (5, 6)).unwrap().linked);
}

proptest! {
    #[test]
    fn pairs_are_ordered_and_meet_the_rules(
        raw in proptest::collection::vec((0_u64..30, proptest::collection::vec(0_i64..20, 0..6), 0_u8..3), 0..25)
    ) {
        let items: Vec<PairInput> = raw.iter().enumerate().map(|(i, (_, days, word))| {
            let mut days = days.clone();
            days.sort_unstable();
            days.dedup();
            pair(i as u64, ["alpha beta gamma", "alpha beta delta", "omega psi"][*word as usize], &[], &[], &days)
        }).collect();
        let rules = pair_rules_default();
        for p in confusion_pairs(&items, &rules) {
            prop_assert!(p.a < p.b);
            prop_assert!(p.co_lapses >= rules.min_co_lapses);
            prop_assert!(p.similarity >= rules.min_similarity || p.shared_tags > 0 || p.linked);
        }
    }
}
