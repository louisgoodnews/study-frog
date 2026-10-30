#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Which items link to an item.

use std::fs;

use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::session::open_vault;
use studyfrog_store::overview::linking_to;

#[test]
fn notes_cards_and_questions_that_link_to_a_title_are_found_but_not_the_item_itself() {
    let dir = std::env::temp_dir().join(format!("studyfrog-backlinks-{}", std::process::id()));
    fs::create_dir_all(dir.join("s")).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    fs::write(
        dir.join("s/_stack.md"),
        format!("---\nid: {}\nkind: stack\ntitle: S\n---\n", id(1)),
    )
    .unwrap();
    fs::write(
        dir.join("s/alpha.md"),
        format!(
            "---\nid: {}\nkind: note\ntitle: Alpha\n---\nSee [[Alpha]] itself.\n",
            id(2)
        ),
    )
    .unwrap();
    fs::write(
        dir.join("s/beta.md"),
        format!(
            "---\nid: {}\nkind: note\ntitle: Beta\n---\nAbout [[alpha]].\n",
            id(3)
        ),
    )
    .unwrap();
    fs::write(
        dir.join("s/c.md"),
        format!(
            "---\nid: {}\nkind: card\ntitle: C\n---\nWhat is [[Alpha]]?\n???\nA note.\n",
            id(4)
        ),
    )
    .unwrap();
    let (session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let found = linking_to(&session.conn, "Alpha", Uid(2)).unwrap();
    let mut titles: Vec<&str> = found.iter().map(|r| r.title.as_str()).collect();
    titles.sort_unstable();
    assert_eq!(titles, ["Beta", "C"], "case-insensitive, and not itself");
    assert!(
        linking_to(&session.conn, "Nobody", Uid(0))
            .unwrap()
            .is_empty()
    );
    let _ = fs::remove_dir_all(&dir);
}
