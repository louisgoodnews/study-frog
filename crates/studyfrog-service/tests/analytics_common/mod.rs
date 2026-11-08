#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Fixture vaults with a review history and run events, for the analytics of EPIC-034.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::schedule::apply_rating;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_srs::params::params_default;
use studyfrog_store::schedule::StoredMem;
use studyfrog_store::schedule::save_mem_state;

pub const DAY: i64 = 86_400_000;
/// Midnight UTC of a Monday (2027-01-11).
pub const T0: i64 = 1_799_625_600_000;

/// One run event: (t_ms, item, kind, payload).
pub type Event<'a> = (u32, Option<u128>, &'a str, Option<&'a str>);

pub struct Dir(pub PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A card: id, stack folder ("" for the vault root), title, body.
pub type Card<'a> = (u128, &'a str, &'a str, &'a str);

/// A vault with stacks (folder, id, title) and cards, opened.
pub fn vault(label: &str, stacks: &[(&str, u128, &str)], cards: &[Card]) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-an-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (folder, id, title) in stacks {
        fs::create_dir_all(dir.join(folder)).unwrap();
        let text = format!(
            "---\nid: {}\nkind: stack\ntitle: {title}\n---\n",
            format_ulid(Uid(*id))
        );
        fs::write(dir.join(folder).join("_stack.md"), text).unwrap();
    }
    for (id, folder, title, body) in cards {
        let text = format!(
            "---\nid: {}\nkind: card\ntitle: {title}\n---\n{body}\n",
            format_ulid(Uid(*id))
        );
        fs::write(dir.join(folder).join(format!("c{id}.md")), text).unwrap();
    }
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

/// Rate items at the given instants (in order), keeping the FSRS state of each in its primary stack.
pub fn rate_all(s: &VaultSession, ratings: &[(u128, i64, Rating)]) {
    let p = params_default();
    let mut prev: HashMap<u128, StoredMem> = HashMap::new();
    for (item, at, rating) in ratings {
        let (r, mem) = apply_rating(prev.get(item), Uid(*item), *rating, UnixMs(*at), &p);
        studyfrog_store::reviews::insert_review(&s.conn, &r).unwrap();
        save_mem_state(&s.conn, Uid(*item), &mem).unwrap();
        prev.insert(*item, mem);
    }
}

/// Record a run started at `start_ms` with events (seq follows the order): (t_ms, item, kind, payload).
pub fn run_events(s: &VaultSession, run: u128, start_ms: i64, events: &[Event]) {
    s.conn
        .execute(
            "INSERT INTO run (uid, mode, state, started_at_ms) VALUES (?1, 'flip', 'finished', ?2)",
            rusqlite::params![format_ulid(Uid(run)), start_ms],
        )
        .unwrap();
    for (seq, (t, item, kind, payload)) in events.iter().enumerate() {
        s.conn
            .execute(
                "INSERT INTO run_event (run_uid, seq, t_ms, item_uid, kind, payload) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![format_ulid(Uid(run)), seq as u32, t, item.map(|i| format_ulid(Uid(i))), kind, payload],
            )
            .unwrap();
    }
}

/// Show, flip after `flip_ms` and rate one item at `t`: three events.
pub fn shown_flipped_rated(
    t: u32,
    item: u128,
    flip_ms: u32,
    rating: &'static str,
) -> Vec<Event<'static>> {
    vec![
        (t, Some(item), "item_shown", None),
        (t + flip_ms, Some(item), "item_flipped", None),
        (t + flip_ms + 1_000, Some(item), "item_rated", Some(rating)),
    ]
}
