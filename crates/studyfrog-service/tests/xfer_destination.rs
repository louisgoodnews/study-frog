#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Where an import puts items: as the file says, into a folder, or at the vault root.

use std::fs;
use std::path::PathBuf;

use studyfrog_service::session::open_vault;
use studyfrog_service::xfer_import::{Verdict, plan_import};
use studyfrog_service::xfer_json::from_json;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const BUNDLE: &str = r#"{"format":"studyfrog-export","version":1,"exported":"2026-01-01T00:00:00Z","scheduling":false,"stacks":[],"items":[{"kind":"card","title":"Hola","stack":"","text":"hello","answer":"hola","tags":[]},{"kind":"card","title":"Adios","stack":"verbs","text":"bye","answer":"adios","tags":[]}]}"#;

#[test]
fn the_destination_decides_the_folder_of_each_item() {
    let dir = std::env::temp_dir().join(format!("studyfrog-dest-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let _guard = Dir(dir.clone());
    let (session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let bundle = from_json(BUNDLE).unwrap();

    let folders = |target: Option<&str>| -> Vec<String> {
        plan_import(&session, &bundle, target)
            .unwrap()
            .lines
            .iter()
            .map(|l| l.folder.clone())
            .collect()
    };
    assert_eq!(
        folders(Some("")),
        ["", "verbs"],
        "the vault root, stacks kept at the root"
    );
    assert_eq!(folders(Some("Spanish")), ["Spanish", "Spanish/verbs"]);
    assert_eq!(folders(None)[1], "verbs", "as the file says");
    assert!(
        plan_import(&session, &bundle, None)
            .unwrap()
            .lines
            .iter()
            .any(|l| matches!(l.verdict, Verdict::Error(_))),
        "an item with no stack needs a target when the file says none"
    );
}

const SAME_STACK: &str = r#"{"format":"studyfrog-export","version":1,"exported":"2026-01-01T00:00:00Z","scheduling":false,"stacks":[],"items":[{"kind":"card","title":"One","stack":"statistik","text":"q1","answer":"a1","tags":[]},{"kind":"card","title":"Two","stack":"statistik","text":"q2","answer":"a2","tags":[]}]}"#;

#[test]
fn a_file_naming_the_chosen_stack_lands_in_that_stack_not_a_copy_of_it() {
    let dir = std::env::temp_dir().join(format!("studyfrog-same-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let _guard = Dir(dir.clone());
    let (session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let bundle = from_json(SAME_STACK).unwrap();
    let folders = |target: Option<&str>| -> Vec<String> {
        plan_import(&session, &bundle, target)
            .unwrap()
            .lines
            .iter()
            .map(|l| l.folder.clone())
            .collect()
    };
    assert_eq!(folders(Some("statistik")), ["statistik", "statistik"]);
    assert_eq!(
        folders(Some("Languages/statistik")),
        ["Languages/statistik", "Languages/statistik"],
        "a nested chosen stack is named by its last part"
    );
    assert_eq!(
        folders(None),
        ["statistik", "statistik"],
        "as the file says"
    );
    assert_eq!(
        folders(Some("Spanish")),
        ["Spanish/statistik", "Spanish/statistik"],
        "a different chosen stack still nests the file's stack"
    );
}
