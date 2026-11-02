#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Bulk edits from the library: every kind of edit, the reasons for skipping, undo, and saved filters.

use std::fs;
use std::path::PathBuf;

use studyfrog_config::filters::SavedFilter;
use studyfrog_core::priority::Priority;
use studyfrog_core::status::Status;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::bulk::{BulkEdit, bulk_apply, bulk_undo};
use studyfrog_service::filters::{delete_filter, load_filters, save_filter};
use studyfrog_service::session::{ServiceError, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn ulid(n: usize) -> String {
    let digit = "0123456789ABCDEFGHJKMNPQRSTVWXYZ".chars().nth(n).unwrap();
    format!("{digit:0>26}")
}

fn stack(title: &str, id: usize) -> String {
    format!("---\nid: {}\nkind: stack\ntitle: {title}\n---\n", ulid(id))
}

fn note(id: usize, title: &str) -> String {
    format!(
        "---\nid: {}\nkind: note\ntitle: {title}\ntags: [x]\nowner: me   # keep\n---\nBody of {title}\n",
        ulid(id)
    )
}

fn vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-bulk-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (folder, title, id) in [("a", "A", 1), ("b", "B", 2)] {
        fs::create_dir_all(dir.join(folder)).unwrap();
        fs::write(dir.join(folder).join("_stack.md"), stack(title, id)).unwrap();
    }
    fs::write(dir.join("a/one.md"), note(10, "One")).unwrap();
    fs::write(dir.join("a/two.md"), note(11, "Two")).unwrap();
    fs::write(dir.join("b/one.md"), note(12, "Other one")).unwrap();
    Dir(dir)
}

const NOW: UnixMs = UnixMs(1_800_000_000_000);

fn read(d: &Dir, p: &str) -> String {
    fs::read_to_string(d.0.join(p)).unwrap()
}

#[test]
fn field_edits_change_every_selected_file_and_keep_the_rest() {
    let d = vault("fields");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let ids = [Uid(10), Uid(11)];
    let done = bulk_apply(&mut s, &ids, &BulkEdit::Status(Status::Done), NOW).unwrap();
    assert_eq!((done.changed, done.skipped.len()), (2, 0));
    for p in ["a/one.md", "a/two.md"] {
        let t = read(&d, p);
        assert!(
            t.contains("status: done") && t.contains("owner: me   # keep"),
            "{t}"
        );
        assert!(t.contains("updated:"), "{t}");
    }
    assert!(
        !read(&d, "b/one.md").contains("status:"),
        "an unselected file is untouched"
    );

    bulk_apply(&mut s, &ids, &BulkEdit::Priority(Priority::High), NOW).unwrap();
    assert!(read(&d, "a/one.md").contains("priority: high"));
    bulk_apply(&mut s, &ids, &BulkEdit::AddTag("exam".into()), NOW).unwrap();
    bulk_apply(&mut s, &ids, &BulkEdit::AddTag("exam".into()), NOW).unwrap();
    let t = read(&d, "a/one.md");
    assert_eq!(t.matches("exam").count(), 1, "a tag is added once: {t}");
    bulk_apply(&mut s, &ids, &BulkEdit::RemoveTag("x".into()), NOW).unwrap();
    assert!(
        !read(&d, "a/one.md").contains("tags: [x"),
        "{}",
        read(&d, "a/one.md")
    );
}

#[test]
fn moving_picks_a_free_name_and_delete_removes_and_both_can_be_undone() {
    let d = vault("move");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let before = read(&d, "a/one.md");
    let moved = bulk_apply(
        &mut s,
        &[Uid(10), Uid(11)],
        &BulkEdit::MoveTo("b".into()),
        NOW,
    )
    .unwrap();
    assert_eq!(moved.changed, 2);
    assert!(
        !d.0.join("a/one.md").exists() && d.0.join("b/one-2.md").exists(),
        "name taken: one-2"
    );
    assert!(d.0.join("b/two.md").exists());
    bulk_undo(&mut s, &moved.undo).unwrap();
    assert_eq!(read(&d, "a/one.md"), before);
    assert!(!d.0.join("b/one-2.md").exists() && !d.0.join("b/two.md").exists());

    let gone = bulk_apply(&mut s, &[Uid(11)], &BulkEdit::Delete, NOW).unwrap();
    assert!(!d.0.join("a/two.md").exists());
    assert!(gone.info.overview.problems.is_empty());
    bulk_undo(&mut s, &gone.undo).unwrap();
    assert!(read(&d, "a/two.md").contains("Body of Two"));
}

#[test]
fn what_cannot_be_changed_is_skipped_with_a_reason_and_a_bad_target_is_refused() {
    let d = vault("skip");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let done = bulk_apply(
        &mut s,
        &[Uid(1), Uid(31), Uid(10)],
        &BulkEdit::Status(Status::Paused),
        NOW,
    )
    .unwrap();
    assert_eq!(
        (done.changed, done.skipped.len()),
        (1, 2),
        "{:?}",
        done.skipped
    );
    let err = bulk_apply(&mut s, &[Uid(10)], &BulkEdit::MoveTo("../etc".into()), NOW);
    assert!(matches!(err, Err(ServiceError::Invalid(_))));
}

#[test]
fn saved_filters_are_kept_in_the_vault_and_replaced_by_name() {
    let d = vault("filters");
    let (s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    assert!(load_filters(&s).0.is_empty());
    let f = |n: &str, q: &str| SavedFilter {
        name: n.into(),
        query: q.into(),
    };
    save_filter(&s, f("Due", "due <= 7d")).unwrap();
    save_filter(&s, f("Cards", "type = card")).unwrap();
    let list = save_filter(&s, f("due", "due <= 3d")).unwrap();
    assert_eq!(list, [f("due", "due <= 3d"), f("Cards", "type = card")]);
    assert_eq!(load_filters(&s).0, list);
    assert!(read(&d, "filters.toml").contains("[[filter]]"));
    assert_eq!(delete_filter(&s, "CARDS").unwrap(), [f("due", "due <= 3d")]);
    assert!(save_filter(&s, f(" ", "x")).is_err());
    fs::write(d.0.join("filters.toml"), "nonsense = [").unwrap();
    let (list, why) = load_filters(&s);
    assert!(list.is_empty() && why.unwrap().contains("filters.toml is not used"));
    assert!(
        save_filter(&s, f("A", "b")).is_err(),
        "a broken file is not overwritten"
    );
}
