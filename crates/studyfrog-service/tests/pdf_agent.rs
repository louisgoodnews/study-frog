#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! PDF pages as context for the assistant.

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::PathBuf;

use studyfrog_service::create::IdSource;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::pdf_notes::items_made_from;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn session(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-pagent-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("bio")).unwrap();
    fs::write(
        dir.join("bio/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Bio\n---\n",
    )
    .unwrap();
    fs::write(dir.join("bio/three.pdf"), pdfmake::three()).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

#[test]
fn pdf_pages_are_context_for_the_assistant_and_a_proposed_card_names_its_page() {
    use studyfrog_agent::wiki::{Change, parse_proposal};
    use studyfrog_core::time::UnixMs;
    use studyfrog_service::agent_pdf::{page_id, wiki_context_with_pdf};
    use studyfrog_service::agent_vault::apply_changes;
    let (dir, mut s) = session("agent");
    studyfrog_service::pdfs::index_pdfs_fully(&mut s, i64::MAX / 2).unwrap();
    let ctx =
        wiki_context_with_pdf(&s, &["bio".to_owned()], Some(("bio/three.pdf", 1, 2))).unwrap();
    let pages: Vec<&str> = ctx
        .items
        .iter()
        .filter(|i| i.kind == "pdf")
        .map(|i| i.uid.as_str())
        .collect();
    assert_eq!(
        pages,
        ["pdf:bio/three.pdf#page=2", "pdf:bio/three.pdf#page=3"]
    );
    assert_eq!(page_id("bio/three.pdf", 1), pages[0]);
    let uids: Vec<String> = ctx.items.iter().map(|i| i.uid.clone()).collect();
    let reply = r#"{"summary":"s","changes":[
        {"action":"create_card","stack":"bio","front":"Third?","back":"Gamma","source":"pdf:bio/three.pdf#page=3"},
        {"action":"create_card","stack":"bio","front":"Made up","back":"x","source":"pdf:other.pdf#page=1"}]}"#;
    let p = parse_proposal(reply, &uids, &["bio".to_owned()]).unwrap();
    assert!(
        matches!(&p.changes[0], Change::CreateCard { source: Some(s), .. } if s == "pdf:bio/three.pdf#page=3")
    );
    assert!(
        matches!(&p.changes[1], Change::CreateCard { source: None, .. }),
        "an id it was not shown is dropped"
    );
    let ids = IdSource {
        now_ms: 1_800_000_000_000,
        random: [4; 10],
    };
    let done = apply_changes(&mut s, &p.changes, ids, UnixMs(1_800_000_000_000));
    assert_eq!(done.created, 2, "{:?}", done.failed);
    let made = items_made_from(&s.conn, "bio/three.pdf", Some(2)).unwrap();
    assert_eq!(made.len(), 1);
    assert!(
        fs::read_to_string(dir.0.join(&made[0].path))
            .unwrap()
            .contains("Gamma")
    );
    assert!(wiki_context_with_pdf(&s, &["bio".to_owned()], Some(("bio/none.pdf", 0, 0))).is_err());
}
