#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Achievements from a real vault's history.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::achievements::check_achievements;
use studyfrog_service::schedule::apply_rating;
use studyfrog_service::session::open_vault;
use studyfrog_srs::params::params_default;
use studyfrog_store::schedule::{insert_review, save_mem_state};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const DAY: i64 = 86_400_000;
const T0: i64 = 1_800_000_000_000;

fn vault(label: &str, extra: Option<&str>) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-ach-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("deck")).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    fs::write(
        dir.join("deck/_stack.md"),
        format!("---\nid: {}\nkind: stack\ntitle: Deck\n---\n", id(1)),
    )
    .unwrap();
    for n in [10, 11] {
        let text = format!(
            "---\nid: {}\nkind: card\ntitle: C{n}\n---\nq\n???\na\n",
            id(n)
        );
        fs::write(dir.join(format!("deck/c{n}.md")), text).unwrap();
    }
    if let Some(text) = extra {
        fs::write(dir.join("achievements.toml"), text).unwrap();
    }
    Dir(dir)
}

fn review(s: &studyfrog_service::session::VaultSession, item: u128, rating: Rating, day: i64) {
    let (r, mem) = apply_rating(
        None,
        Uid(item),
        rating,
        UnixMs(T0 + day * DAY),
        &params_default(),
    );
    insert_review(&s.conn, &r).unwrap();
    save_mem_state(&s.conn, Uid(item), &mem).unwrap();
}

#[test]
fn history_unlocks_achievements_once_with_global_and_stack_progress() {
    let d = vault(
        "unlock",
        Some(
            "[[achievement]]\nid = \"easy-2\"\ntitle = \"Two easy\"\nscope = \"global\"\nmetric = \"easy_ratings\"\nthreshold = 2\n\n[[achievement]]\nid = \"deck-2\"\ntitle = \"Both cards\"\nscope = \"stack\"\nmetric = \"items_learned\"\nthreshold = 2\n\n[[achievement]]\nid = \"run\"\ntitle = \"A run\"\nscope = \"global\"\nmetric = \"runs_finished\"\nthreshold = 1\n",
        ),
    );
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let now = UnixMs(T0 + 2 * DAY);

    let before = check_achievements(&mut s, now, 0).unwrap();
    assert!(
        before.new.is_empty() && before.entries.len() == 3,
        "{before:?}"
    );
    assert_eq!(before.rules_error, None);

    review(&s, 10, Rating::Easy, 0);
    review(&s, 11, Rating::Easy, 1);
    s.conn
        .execute(
            "INSERT INTO run (uid, mode, state) VALUES ('R1', 'flip', 'finished')",
            [],
        )
        .unwrap();

    let report = check_achievements(&mut s, now, 0).unwrap();
    let titles: Vec<&str> = report.new.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["Two easy", "Both cards", "A run"]);
    let deck = report.entries.iter().find(|e| e.rule == "deck-2").unwrap();
    assert_eq!(
        (deck.stack.as_deref(), deck.value, deck.unlocked),
        (Some("Deck"), 2, Some(now))
    );

    let again = check_achievements(&mut s, UnixMs(T0 + 5 * DAY), 0).unwrap();
    assert!(again.new.is_empty(), "an unlock happens once");
    assert_eq!(
        again
            .entries
            .iter()
            .find(|e| e.rule == "run")
            .unwrap()
            .unlocked,
        Some(now),
        "and keeps its time"
    );
}

#[test]
fn without_a_file_the_built_in_set_is_used_and_a_broken_file_is_reported() {
    let d = vault("default", None);
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let r = check_achievements(&mut s, UnixMs(T0), 0).unwrap();
    assert!(r.entries.iter().any(|e| e.rule == "first-run") && r.rules_error.is_none());

    let d = vault(
        "broken",
        Some(
            "[[achievement]]\nid = \"x\"\ntitle = \"X\"\nscope = \"galaxy\"\nmetric = \"streak_days\"\nthreshold = 1\n",
        ),
    );
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let r = check_achievements(&mut s, UnixMs(T0), 0).unwrap();
    assert!(
        r.rules_error
            .unwrap()
            .contains("achievements.toml is not used")
    );
    assert!(
        r.entries.iter().any(|e| e.rule == "first-run"),
        "falls back to the built-in set"
    );
}

#[test]
fn the_streak_follows_the_local_day() {
    let d = vault(
        "streak",
        Some(
            "[[achievement]]\nid = \"streak-2\"\ntitle = \"Two days\"\nscope = \"global\"\nmetric = \"streak_days\"\nthreshold = 2\n",
        ),
    );
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    // 23:00 UTC on day 0 and 00:30 UTC on day 1 are the same local day at UTC-2 ... and two days at UTC.
    let late = T0 - T0.rem_euclid(DAY) + 23 * 3_600_000;
    for (item, at) in [(10, late), (11, late + 5_400_000)] {
        let (r, mem) = apply_rating(None, Uid(item), Rating::Good, UnixMs(at), &params_default());
        insert_review(&s.conn, &r).unwrap();
        save_mem_state(&s.conn, Uid(item), &mem).unwrap();
    }
    let now = UnixMs(late + 5_400_000);
    assert_eq!(
        check_achievements(&mut s, now, 0).unwrap().entries[0].value,
        2,
        "UTC: two calendar days"
    );
    assert_eq!(
        check_achievements(&mut s, now, -120).unwrap().entries[0].value,
        1,
        "UTC-2: one local day"
    );
}
