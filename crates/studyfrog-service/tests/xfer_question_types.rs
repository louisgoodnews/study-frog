#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! End-to-end verification of Question and answer_type (open, single, multiple) imports across formats (TASK-0312).

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_core::uid::Uid;
use studyfrog_service::content::load_item_content;
use studyfrog_service::create::IdSource;
use studyfrog_service::session::open_vault;
use studyfrog_service::xfer_csv::{from_csv, to_csv};
use studyfrog_service::xfer_export::export_bundle;
use studyfrog_service::xfer_import::{Policy, apply_import};
use studyfrog_service::xfer_json::{from_json, to_json};
use studyfrog_service::xfer_ron::{from_ron, to_ron};
use studyfrog_service::xfer_toml::{from_toml, to_toml};

fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "sf_xfer_q_{label}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn seed_source_vault(root: &Path) {
    fs::create_dir_all(root.join("quiz")).unwrap();
    fs::write(
        root.join("quiz/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Quiz\n---\n",
    )
    .unwrap();
    fs::write(
        root.join("quiz/open.md"),
        "---\nid: 00000000000000000000000002\nkind: question\ntitle: Open Q\nanswer_type: open\n---\nWhat is FSRS?\n???\nFree Spaced Repetition Scheduler\n",
    )
    .unwrap();
    fs::write(
        root.join("quiz/single.md"),
        "---\nid: 00000000000000000000000003\nkind: question\ntitle: Single Q\nanswer_type: single\n---\nCapital of France?\n- [x] Paris\n- [ ] Lyon\n???\n",
    )
    .unwrap();
    fs::write(
        root.join("quiz/multi.md"),
        "---\nid: 00000000000000000000000004\nkind: question\ntitle: Multi Q\nanswer_type: multiple\n---\nPrime numbers:\n- [x] 2\n- [x] 3\n- [ ] 4\n???\n2 and 3\n",
    )
    .unwrap();
}

fn assert_imported_questions(bundle: &studyfrog_service::xfer_model::Bundle, label: &str) {
    let dst = temp_dir(label);
    let (mut dst_session, _) = open_vault(dst.to_str().unwrap()).unwrap();
    let ids = IdSource {
        now_ms: 1_700_000_000_000,
        random: [42; 10],
    };
    let done = apply_import(&mut dst_session, bundle, None, (Policy::Skip, false), ids).unwrap();
    assert_eq!(done.created, 3, "{label}: notes={:?}", done.notes);
    assert_eq!(done.skipped, 0, "{label}: notes={:?}", done.notes);

    let mut types = Vec::new();
    for (uid, _) in studyfrog_store::item_meta::load_item_meta(&dst_session.conn).unwrap() {
        let content = load_item_content(&dst_session, uid).unwrap();
        types.push((content.title, content.answer_type));
    }
    types.sort();
    assert_eq!(
        types,
        vec![
            ("Multi Q".into(), Some("multiple".into())),
            ("Open Q".into(), Some("open".into())),
            ("Single Q".into(), Some("single".into())),
        ],
        "{label}: answer_type must survive round-trip"
    );
    let _ = fs::remove_dir_all(&dst);
}

#[test]
fn question_answer_types_survive_json_toml_ron_and_csv_round_trips() {
    let src = temp_dir("src");
    seed_source_vault(&src);
    let (session, _) = open_vault(src.to_str().unwrap()).unwrap();

    // Sanity-check source vault loads all three answer types
    assert_eq!(
        load_item_content(&session, Uid(2))
            .unwrap()
            .answer_type
            .as_deref(),
        Some("open")
    );
    assert_eq!(
        load_item_content(&session, Uid(3))
            .unwrap()
            .answer_type
            .as_deref(),
        Some("single")
    );
    assert_eq!(
        load_item_content(&session, Uid(4))
            .unwrap()
            .answer_type
            .as_deref(),
        Some("multiple")
    );

    let bundle = export_bundle(&session, None, false).unwrap();
    assert_eq!(bundle.items.len(), 3);

    let from_j = from_json(&to_json(&bundle, "2026-11-26T12:00:00Z")).unwrap();
    assert_imported_questions(&from_j, "json");

    let from_t = from_toml(&to_toml(&bundle, "2026-11-26T12:00:00Z")).unwrap();
    assert_imported_questions(&from_t, "toml");

    let from_r = from_ron(&to_ron(&bundle, "2026-11-26T12:00:00Z")).unwrap();
    assert_imported_questions(&from_r, "ron");

    let from_c = from_csv(&to_csv(&bundle)).unwrap();
    assert_imported_questions(&from_c, "csv");

    let _ = fs::remove_dir_all(&src);
}
