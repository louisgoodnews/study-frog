#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The items that need attention: the least familiar, with their reasons.

use std::fs;

use studyfrog_core::memory::MemState;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::attention::needs_attention;
use studyfrog_service::session::open_vault;
use studyfrog_store::overview::load_indexed_items;
use studyfrog_store::schedule::{StoredMem, insert_review_timed, save_mem_state};

const DAY: i64 = 86_400_000;
const NOW: i64 = 1_800_000_000_000;

#[test]
fn the_least_familiar_items_are_listed_with_why_and_the_well_known_are_not() {
    let dir = std::env::temp_dir().join(format!("studyfrog-attn-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    for (id, name) in [('A', "weak"), ('B', "strong"), ('C', "unseen")] {
        let text = format!(
            "---\nid: 0000000000000000000000000{id}\nkind: card\ntitle: {name}\n---\nq\n???\na\n"
        );
        fs::write(dir.join(format!("s/{name}.md")), text).unwrap();
    }
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let review = |item: u128, rating, days_ago: i64| Review {
        item: Uid(item),
        rating,
        at: UnixMs(NOW - days_ago * DAY),
        elapsed_days: 1.0,
    };
    insert_review_timed(&s.conn, &review(0xA, Rating::Again, 5), None).unwrap();
    for (i, d) in [(1, 30), (2, 20), (3, 10), (4, 2)] {
        let _ = i;
        insert_review_timed(&s.conn, &review(0xB, Rating::Easy, d), None).unwrap();
    }
    let mem = |due: i64| StoredMem {
        state: MemState {
            stability: 10.0,
            difficulty: 5.0,
        },
        due: UnixMs(due),
        last_review: UnixMs(NOW - 2 * DAY),
    };
    save_mem_state(&s.conn, Uid(0xA), &mem(NOW - 3 * DAY)).unwrap();
    save_mem_state(&s.conn, Uid(0xB), &mem(NOW + 9 * DAY)).unwrap();
    let items = load_indexed_items(&s.conn).unwrap();
    let got = needs_attention(&s.conn, &items, UnixMs(NOW)).unwrap();
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].title, "weak");
    assert!(got[0].reasons[0].starts_with("Familiarity"));
    assert!(
        got[0]
            .reasons
            .iter()
            .any(|r| r.contains("3 day(s) overdue")),
        "{:?}",
        got[0].reasons
    );
    let _ = fs::remove_dir_all(&dir);
}
