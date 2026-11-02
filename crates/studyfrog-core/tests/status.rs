#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Item status: stored words, labels and the derived status.

use studyfrog_core::status::{
    STATUSES, Status, effective_status, skips_schedule, status_from_name, status_label, status_name,
};

#[test]
fn every_status_has_a_stable_word_and_a_label_and_reads_back() {
    let words: Vec<&str> = STATUSES.iter().map(|s| status_name(*s)).collect();
    assert_eq!(words, ["new", "learning", "reviewing", "done", "paused"]);
    for s in STATUSES {
        assert_eq!(status_from_name(status_name(s)), Some(s));
    }
    let labels: Vec<&str> = STATUSES.iter().map(|s| status_label(*s)).collect();
    assert_eq!(
        labels,
        [
            "Not studied yet",
            "Learning",
            "In review",
            "Complete",
            "On hold"
        ]
    );
    assert_eq!(
        status_from_name(" done "),
        Some(Status::Done),
        "blanks are ignored"
    );
    assert_eq!(status_from_name("finished"), None);
    assert_eq!(
        status_from_name("Done"),
        None,
        "the stored words are lowercase"
    );
}

#[test]
fn without_a_stored_status_it_follows_the_history_and_a_stored_one_wins() {
    assert_eq!(effective_status(None, false), Status::New);
    assert_eq!(effective_status(None, true), Status::Learning);
    assert_eq!(effective_status(Some(Status::New), true), Status::New);
    assert_eq!(effective_status(Some(Status::Done), false), Status::Done);
    assert_eq!(
        effective_status(Some(Status::Reviewing), true),
        Status::Reviewing
    );
}

#[test]
fn only_done_and_paused_leave_the_schedule() {
    let skipped: Vec<Status> = STATUSES
        .into_iter()
        .filter(|s| skips_schedule(*s))
        .collect();
    assert_eq!(skipped, [Status::Done, Status::Paused]);
}
