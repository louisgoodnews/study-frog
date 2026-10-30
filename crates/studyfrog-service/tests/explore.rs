#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The report explorer on a real vault with a small history.

use std::fs;

use studyfrog_config::subjects::SubjectBook;
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::explore::{Period, RowKind, Scope, explore};
use studyfrog_service::schedule::apply_rating;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_srs::params::params_default;
use studyfrog_store::schedule::{insert_review, save_mem_state};

const DAY: i64 = 86_400_000;
const NOW: i64 = 1_800_000_000_000 - (1_800_000_000_000 % DAY) + 12 * 3_600_000;

fn write(dir: &std::path::Path, rel: &str, n: u128, kind: &str, extra: &str) {
    let body = if kind == "card" { "q\n???\na\n" } else { "" };
    let text = format!(
        "---\nid: {}\nkind: {kind}\ntitle: T{n}\n{extra}---\n{body}",
        format_ulid(Uid(n))
    );
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn review(s: &VaultSession, item: u128, rating: Rating, at: i64) {
    let (r, mem) = apply_rating(None, Uid(item), rating, UnixMs(at), &params_default());
    insert_review(&s.conn, &r).unwrap();
    save_mem_state(&s.conn, Uid(item), &mem).unwrap();
}

fn vault(label: &str) -> (std::path::PathBuf, VaultSession) {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-explore-{label}-{}", std::process::id()));
    write(&dir, "a/_stack.md", 1, "stack", "subjects: Maths\n");
    write(&dir, "a/b/_stack.md", 2, "stack", "");
    write(&dir, "a/c10.md", 10, "card", "");
    write(&dir, "a/c11.md", 11, "card", "");
    write(&dir, "a/b/c12.md", 12, "card", "");
    write(&dir, "z/_stack.md", 3, "stack", "");
    write(&dir, "z/c13.md", 13, "card", "");
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    review(&s, 10, Rating::Good, NOW - 5 * DAY);
    review(&s, 11, Rating::Again, NOW - 40 * DAY);
    review(&s, 11, Rating::Good, NOW - 1000);
    review(&s, 12, Rating::Easy, NOW - 2 * DAY);
    (dir, s)
}

#[test]
fn the_vault_level_adds_up_and_the_period_filters() {
    let (dir, s) = vault("vault");
    let now = UnixMs(NOW);
    let all = explore(&s, (Scope::Vault, Period::All), now, 0).unwrap();
    assert_eq!(
        (all.perf.reviews, all.perf.items, all.perf.ratings),
        (4, 3, [1, 0, 2, 1])
    );
    assert_eq!(all.crumbs.len(), 1);
    let rows: Vec<(&str, u32)> = all
        .rows
        .iter()
        .map(|r| (r.label.as_str(), r.perf.reviews))
        .collect();
    assert_eq!(
        rows,
        [("T1", 4), ("T3", 0)],
        "top-level stacks, children counted inside their parent"
    );
    assert_eq!(
        all.subjects
            .iter()
            .map(|(n, p)| (n.as_str(), p.reviews))
            .collect::<Vec<_>>(),
        [("Maths", 4)]
    );
    assert_eq!(all.chart.len(), 30);
    assert_eq!(
        all.chart.iter().map(|d| d.reviews).sum::<u32>(),
        3,
        "the review 40 days ago is outside the chart"
    );

    let month = explore(&s, (Scope::Vault, Period::Days(30)), now, 0).unwrap();
    assert_eq!(month.perf.reviews, 3, "the 40-day-old Again is left out");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_stack_and_item_levels_list_what_is_inside_and_the_way_down() {
    let (dir, s) = vault("levels");
    let now = UnixMs(NOW);
    let stack = explore(&s, (Scope::Stack("a".into()), Period::All), now, 0).unwrap();
    assert_eq!(stack.perf.reviews, 4);
    let kinds: Vec<(&str, RowKind)> = stack
        .rows
        .iter()
        .map(|r| (r.label.as_str(), r.kind))
        .collect();
    assert_eq!(kinds[0], ("T2", RowKind::Stack));
    assert_eq!(
        kinds
            .iter()
            .filter(|(_, k)| matches!(k, RowKind::Item(_)))
            .count(),
        2,
        "its own two cards"
    );
    assert_eq!(
        stack
            .crumbs
            .iter()
            .map(|c| c.0.as_str())
            .collect::<Vec<_>>(),
        ["All", "T1"]
    );

    let inner = explore(&s, (Scope::Stack("a/b".into()), Period::All), now, 0).unwrap();
    assert_eq!(
        inner
            .crumbs
            .iter()
            .map(|c| c.0.as_str())
            .collect::<Vec<_>>(),
        ["All", "T1", "T2"]
    );

    let item = explore(&s, (Scope::Item(Uid(11)), Period::All), now, 0).unwrap();
    assert_eq!(item.perf.reviews, 2);
    assert_eq!(
        item.history.iter().map(|h| h.rating).collect::<Vec<_>>(),
        [3, 1],
        "newest first"
    );
    assert_eq!(
        item.crumbs.iter().map(|c| c.0.as_str()).collect::<Vec<_>>(),
        ["All", "T1", "T11"]
    );
    let _ = SubjectBook::default();
    let _ = fs::remove_dir_all(&dir);
}
