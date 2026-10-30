#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Reading speed: what the time between reveal and rating suggests.

use proptest::prelude::*;
use studyfrog_core::event::{
    Difficulty, EventKind, RunEvent, event_from_parts, event_name, event_payload, is_item_event,
};
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::reading::{
    ItemRecord, Reason, expected_ms, item_records, reading_params_default, suggest, word_count,
};

fn ev(seq: u32, t_ms: u32, item: Option<u128>, kind: EventKind) -> RunEvent {
    RunEvent {
        run: Uid(1),
        seq,
        t_ms,
        item: item.map(Uid),
        kind,
    }
}

#[test]
fn words_are_counted_and_the_baseline_gives_the_expected_time() {
    let p = reading_params_default();
    assert_eq!(word_count("  one two\nthree\tfour "), 4);
    assert_eq!(word_count(""), 0);
    assert_eq!(expected_ms(238, &p), 60_000);
    assert_eq!(
        expected_ms(20, &p),
        5042,
        "twenty words take about five seconds"
    );
}

#[test]
fn a_fast_read_of_a_known_item_rated_good_suggests_easy_and_a_new_one_suggests_hard() {
    let p = reading_params_default();
    let known = suggest(20, 3000, Some(Rating::Good), Rating::Good, &p).unwrap();
    assert_eq!(
        (known.rating, known.reason),
        (Rating::Easy, Reason::KnownAndFast)
    );
    let new = suggest(20, 3000, None, Rating::Good, &p).unwrap();
    assert_eq!((new.rating, new.reason), (Rating::Hard, Reason::NewAndFast));
    assert_eq!(
        suggest(20, 3000, None, Rating::Easy, &p).unwrap().rating,
        Rating::Hard
    );
}

#[test]
fn nothing_is_suggested_for_normal_speed_short_texts_or_ratings_that_already_agree() {
    let p = reading_params_default();
    assert_eq!(
        suggest(20, 5000, Some(Rating::Good), Rating::Good, &p),
        None,
        "normal pace"
    );
    assert_eq!(
        suggest(7, 100, Some(Rating::Good), Rating::Good, &p),
        None,
        "too short"
    );
    assert_eq!(
        suggest(20, 3000, Some(Rating::Good), Rating::Easy, &p),
        None,
        "already easy"
    );
    assert_eq!(
        suggest(20, 3000, Some(Rating::Good), Rating::Hard, &p),
        None,
        "they struggled"
    );
    assert_eq!(
        suggest(20, 3000, None, Rating::Again, &p),
        None,
        "they failed it"
    );
    assert_eq!(
        suggest(20, 3000, None, Rating::Hard, &p),
        None,
        "already hard"
    );
}

#[test]
fn the_log_gives_reading_time_without_paused_time_and_the_last_mark() {
    let events = [
        ev(0, 0, None, EventKind::RunStarted),
        ev(1, 0, Some(1), EventKind::ItemShown),
        ev(2, 500, Some(1), EventKind::ItemMarked(Difficulty::Easy)),
        ev(3, 1000, Some(1), EventKind::ItemFlipped),
        ev(4, 1500, None, EventKind::RunPaused),
        ev(5, 2500, None, EventKind::RunResumed),
        ev(
            6,
            1000 + 4000,
            Some(1),
            EventKind::ItemMarked(Difficulty::Difficult),
        ),
        ev(7, 5000, Some(1), EventKind::ItemRated(Rating::Good)),
        ev(8, 5000, Some(2), EventKind::ItemShown),
        ev(9, 5200, Some(2), EventKind::ItemSkipped),
    ];
    let records = item_records(&events);
    assert_eq!(
        records,
        [
            ItemRecord {
                item: Uid(1),
                rated: Some(Rating::Good),
                marked: Some(Difficulty::Difficult),
                read_ms: Some(3000)
            },
            ItemRecord {
                item: Uid(2),
                rated: None,
                marked: None,
                read_ms: None
            },
        ]
    );
}

#[test]
fn an_item_that_comes_back_is_judged_by_its_last_showing() {
    let events = [
        ev(0, 0, Some(1), EventKind::ItemShown),
        ev(1, 100, Some(1), EventKind::ItemFlipped),
        ev(2, 900, Some(1), EventKind::ItemRated(Rating::Again)),
        ev(3, 1000, Some(1), EventKind::ItemShown),
        ev(4, 1100, Some(1), EventKind::ItemFlipped),
        ev(5, 3100, Some(1), EventKind::ItemRated(Rating::Good)),
    ];
    let r = item_records(&events);
    assert_eq!(
        (r.len(), r[0].rated, r[0].read_ms),
        (1, Some(Rating::Good), Some(2000))
    );
}

proptest! {
    #[test]
    fn a_suggestion_never_repeats_or_softens_the_given_rating(
        words in 0_usize..200, ms in 0_u32..30_000, prev in prop::option::of(1_u8..5), rated in 1_u8..5,
    ) {
        let to = |v: u8| studyfrog_core::review::rating_from_value(v).unwrap();
        let p = reading_params_default();
        let (previous, rated) = (prev.map(to), to(rated));
        if let Some(s) = suggest(words, ms, previous, rated, &p) {
            prop_assert!(s.rating != rated);
            prop_assert!(words >= p.min_words);
            prop_assert!(rated >= Rating::Good, "a low rating is never overridden");
            prop_assert_eq!(s.rating == Rating::Easy, previous.is_some());
        }
    }
}

#[test]
fn a_mark_survives_being_stored_and_loaded() {
    for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Difficult] {
        let kind = EventKind::ItemMarked(d);
        assert!(is_item_event(&kind));
        let back = event_from_parts(event_name(&kind), event_payload(&kind).as_deref());
        assert_eq!(back, Some(kind));
    }
    assert_eq!(event_from_parts("item_marked", Some("tough")), None);
    assert_eq!(event_from_parts("item_marked", None), None);
}
