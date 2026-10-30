#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Replacing the rating of the latest review rebuilds the schedule as if it had been given then.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::rerate::{replay_fsrs, replay_sm2, rerate_latest};
use studyfrog_service::schedule::apply_rating;
use studyfrog_service::session::{ServiceError, open_vault};
use studyfrog_srs::params::params_default;
use studyfrog_store::schedule::{insert_review, load_mem_state, load_reviews, save_mem_state};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const DAY: i64 = 86_400_000;
const ITEM: Uid = Uid(10);

fn vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-rerate-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let text = format!(
        "---\nid: {}\nkind: card\ntitle: C\n---\nq\n???\na\n",
        format_ulid(ITEM)
    );
    fs::write(dir.join("c.md"), text).unwrap();
    Dir(dir)
}

#[test]
fn the_rebuilt_schedule_equals_the_one_the_other_rating_would_have_made() {
    let dir = vault("equal");
    let (mut s, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let params = params_default();
    let mut prev = None;
    for (rating, day) in [(Rating::Good, 0), (Rating::Good, 3)] {
        let (review, mem) = apply_rating(
            prev.as_ref(),
            ITEM,
            rating,
            UnixMs(1_800_000_000_000 + day * DAY),
            &params,
        );
        insert_review(&s.conn, &review).unwrap();
        save_mem_state(&s.conn, ITEM, &mem).unwrap();
        prev = Some(mem);
    }
    let good = load_mem_state(&s.conn, ITEM).unwrap().unwrap();

    rerate_latest(&mut s, ITEM, Rating::Good, Rating::Easy).unwrap();

    let reviews = load_reviews(&s.conn, ITEM).unwrap();
    assert_eq!(
        reviews.iter().map(|r| r.rating).collect::<Vec<_>>(),
        [Rating::Good, Rating::Easy]
    );
    let now = load_mem_state(&s.conn, ITEM).unwrap().unwrap();
    assert_eq!(Some(now), replay_fsrs(&reviews, &params));
    assert!(
        now.due > good.due,
        "an easier rating schedules the item later"
    );
}

#[test]
fn a_changed_history_is_refused_and_nothing_is_touched() {
    let dir = vault("refused");
    let (mut s, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let e = rerate_latest(&mut s, ITEM, Rating::Good, Rating::Easy).unwrap_err();
    assert!(
        matches!(e, ServiceError::Invalid(m) if m.contains("changed")),
        "no review at all"
    );

    let (review, mem) = apply_rating(
        None,
        ITEM,
        Rating::Hard,
        UnixMs(1_800_000_000_000),
        &params_default(),
    );
    insert_review(&s.conn, &review).unwrap();
    save_mem_state(&s.conn, ITEM, &mem).unwrap();
    assert!(
        rerate_latest(&mut s, ITEM, Rating::Good, Rating::Easy).is_err(),
        "another rating than expected"
    );
    assert_eq!(load_reviews(&s.conn, ITEM).unwrap()[0].rating, Rating::Hard);
    assert_eq!(load_mem_state(&s.conn, ITEM).unwrap(), Some(mem));
}

#[test]
fn replay_of_no_reviews_is_no_state() {
    assert_eq!(replay_fsrs(&[], &params_default()), None);
    assert_eq!(replay_sm2(&[], 365), None);
}
