#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0097, STORY-0123: the rehearsal and the library read an item's colour from its file.

use std::fs;

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;
use studyfrog_service::content::load_item_content;
use studyfrog_service::library_rows::load_library_rows;
use studyfrog_service::session::open_vault;

#[test]
fn the_colour_of_an_item_reaches_the_rehearsal_and_the_library() {
    let dir = std::env::temp_dir().join(format!("studyfrog-color-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        "---\nid: 01JBYQ00W9Y7A19B1X00000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    let card = "---\nid: 01JBYQ00W9Y7A19B1X00000002\nkind: card\ntitle: C\ncolor: \"#4a90e2\"\n---\nq\n???\na\n";
    fs::write(dir.join("s/c.md"), card).unwrap();
    let plain = "---\nid: 01JBYQ00W9Y7A19B1X00000003\nkind: card\ntitle: D\ncolor: nonsense\n---\nq\n???\na\n";
    fs::write(dir.join("s/d.md"), plain).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let c: Uid = parse_ulid("01JBYQ00W9Y7A19B1X00000002").unwrap();
    let d: Uid = parse_ulid("01JBYQ00W9Y7A19B1X00000003").unwrap();
    assert_eq!(
        load_item_content(&s, c).unwrap().color.as_deref(),
        Some("#4A90E2")
    );
    assert_eq!(
        load_item_content(&s, d).unwrap().color,
        None,
        "a value that is no colour is ignored"
    );
    let rows = load_library_rows(&s, UnixMs(0)).unwrap();
    let row = rows.iter().find(|r| r.uid == c).unwrap();
    assert_eq!(row.color.as_deref(), Some("#4A90E2"));
    assert_eq!(rows.iter().find(|r| r.uid == d).unwrap().color, None);
    let _ = fs::remove_dir_all(&dir);
}
