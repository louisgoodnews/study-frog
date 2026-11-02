#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The rows of the library list, built from a real vault, and queries run over them.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::memory::MemState;
use studyfrog_core::query_eval::run;
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::status::Status;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_core::views::{ViewEvent, ViewKind};
use studyfrog_service::library_rows::{LibraryItem, load_library_rows};
use studyfrog_service::session::open_vault;
use studyfrog_store::schedule::{StoredMem, insert_review_timed, save_mem_state};
use studyfrog_store::views::insert_view;

const DAY: i64 = 86_400_000;
const NOW: i64 = 1_800_000_000_000;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> Dir {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-librows-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("french/verbs")).unwrap();
    fs::create_dir_all(dir.join("maths")).unwrap();
    let w = |p: &str, t: &str| fs::write(dir.join(p), t).unwrap();
    w(
        "french/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\nsubjects: [\"Languages, modern\"]\npriority: high\n---\n",
    );
    w(
        "french/verbs/_stack.md",
        "---\nid: 00000000000000000000000002\nkind: stack\ntitle: Verbs\n---\n",
    );
    w(
        "maths/_stack.md",
        "---\nid: 00000000000000000000000003\nkind: stack\ntitle: Mathematics\n---\n",
    );
    w(
        "french/a.md",
        "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Bonjour\ntags: [exam, greeting]\n---\nHello\n???\nGood morning, said the baker\n",
    );
    w(
        "french/verbs/b.md",
        "---\nid: 0000000000000000000000000B\nkind: card\ntitle: Etre\nstatus: done\ndifficulty: difficult\n---\nTo be\n???\nEtre\n",
    );
    w(
        "maths/c.md",
        "---\nid: 0000000000000000000000000C\nkind: note\ntitle: Primes\n---\nA prime has exactly two divisors.\n",
    );
    w(
        "inbox.md",
        "---\nid: 0000000000000000000000000D\nkind: question\ntitle: Loose\n---\nWhy?\n???\nBecause.\n",
    );
    w(
        "subjects.toml",
        "[[teacher]]\nname = \"Prof. Dr. Künkelmann, Harry\"\n[[subject]]\nname = \"Languages, modern\"\nteacher = \"Prof. Dr. Künkelmann, Harry\"\n",
    );
    Dir(dir)
}

fn rows(label: &str) -> (Dir, Vec<LibraryItem>) {
    let dir = vault(label);
    let (session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let a = Uid(0xA);
    for (n, at) in [
        (1, NOW - 30 * DAY),
        (2, NOW - 20 * DAY),
        (3, NOW - 10 * DAY),
    ] {
        let review = Review {
            item: a,
            rating: Rating::Good,
            at: UnixMs(at),
            elapsed_days: n as f32,
        };
        insert_review_timed(&session.conn, &review, Some((1_000, 2_000))).unwrap();
    }
    let mem = StoredMem {
        state: MemState {
            stability: 12.0,
            difficulty: 5.0,
        },
        due: UnixMs(NOW + 3 * DAY),
        last_review: UnixMs(NOW - 10 * DAY),
    };
    save_mem_state(&session.conn, a, &mem).unwrap();
    let seen = ViewEvent {
        item: a,
        kind: ViewKind::View,
        at: UnixMs(NOW - DAY),
        active_ms: 5_000,
        expected_ms: None,
    };
    insert_view(&session.conn, &seen).unwrap();
    let list = load_library_rows(&session, UnixMs(NOW)).unwrap();
    (dir, list)
}

fn titles(list: &[LibraryItem], q: &str) -> Vec<String> {
    let rows: Vec<_> = list.iter().map(|i| i.row.clone()).collect();
    run(&parse_query(q).unwrap(), &rows, NOW / DAY)
        .into_iter()
        .map(|i| rows[i].title.clone())
        .collect()
}

fn row<'a>(list: &'a [LibraryItem], title: &str) -> &'a studyfrog_core::query_eval::Row {
    &list.iter().find(|i| i.row.title == title).unwrap().row
}

#[test]
fn every_item_has_a_row_with_what_a_query_can_ask() {
    let (_dir, list) = rows("fields");
    assert_eq!(list.len(), 4, "stacks are not items");
    let a = row(&list, "Bonjour");
    assert_eq!(
        (a.status, a.reviews),
        (Status::Learning, 3),
        "derived from the reviews"
    );
    assert!(
        a.familiarity.unwrap() > 0.3,
        "ratings and a view count: {:?}",
        a.familiarity
    );
    assert_eq!(a.stacks, ["french", "French"]);
    assert_eq!(a.subjects, ["Languages, modern"]);
    assert_eq!(a.teachers, ["Prof. Dr. Künkelmann, Harry"]);
    assert_eq!(a.tags, ["exam", "greeting"]);
    assert_eq!(
        format!("{:?}", a.priority),
        "High",
        "inherited from the stack"
    );
    assert!(a.body.contains("Good morning"));
    assert_eq!(a.due_day, Some((NOW + 3 * DAY) / DAY), "the next review");
    let b = row(&list, "Etre");
    assert_eq!(b.status, Status::Done, "a stored status wins");
    assert_eq!(
        b.stacks,
        ["french", "French", "french/verbs", "Verbs"],
        "the stack and every stack above it"
    );
    assert!(
        b.familiarity.is_none() && b.due_day.is_none(),
        "never reviewed"
    );
    let loose = row(&list, "Loose");
    assert!(
        loose.stacks.is_empty() && loose.subjects.is_empty(),
        "an item in the inbox"
    );
}

#[test]
fn queries_run_over_the_rows_of_a_real_vault() {
    let (_dir, list) = rows("queries");
    assert_eq!(titles(&list, "stack = french"), ["Bonjour", "Etre"]);
    assert_eq!(titles(&list, "stack = verbs"), ["Etre"]);
    assert_eq!(titles(&list, "status = learning"), ["Bonjour"]);
    assert_eq!(titles(&list, "text ~ baker"), ["Bonjour"]);
    assert_eq!(
        titles(
            &list,
            "teacher = \"Prof. Dr. Künkelmann, Harry\" AND type = card"
        ),
        ["Bonjour", "Etre"]
    );
    assert_eq!(titles(&list, "tag = exam"), ["Bonjour"]);
    assert_eq!(titles(&list, "type = note"), ["Primes"]);
    assert_eq!(
        titles(&list, "familiarity IS EMPTY AND status != done"),
        ["Loose", "Primes"]
    );
    assert_eq!(titles(&list, "difficulty = difficult"), ["Etre"]);
    assert_eq!(
        titles(&list, "ORDER BY title"),
        ["Bonjour", "Etre", "Loose", "Primes"]
    );
    assert_eq!(
        titles(&list, "priority >= high ORDER BY title"),
        ["Bonjour", "Etre"]
    );
}
