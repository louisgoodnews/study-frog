#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Folder and file operations in the service: create, rename, move, delete, rollback, hostile paths (TASK-0203).

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_core::memory::MemState;
use studyfrog_core::time::UnixMs;
use studyfrog_service::folder_ops::{
    create_folder, delete_path, execute_move_with_fault, move_path, plan_move, rename_path,
    undo_folder_op,
};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::schedule::{StoredMem, load_mem_state_in, save_mem_state_in};

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-fops-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn write(dir: &Path, rel: &str, content: &str) {
    let file = dir.join(rel);
    if let Some(p) = file.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&file, content).unwrap();
}

#[test]
fn test_create_and_delete_folder() {
    let (d, mut s) = vault("create_del");
    let (_, snap, plan) = create_folder(&mut s, "notes/science").unwrap();
    assert_eq!(plan.path, "notes/science");
    assert!(d.0.join("notes/science").is_dir());

    undo_folder_op(&mut s, &snap).unwrap();
    assert!(!d.0.join("notes/science").exists());

    create_folder(&mut s, "notes/science").unwrap();
    write(
        &d.0,
        "notes/science/a.md",
        "---\nid: 01000000000000000000000001\nkind: note\ntitle: A\n---\nBody\n",
    );
    let (_, del_snap, del_plan) = delete_path(&mut s, "notes/science").unwrap();
    assert_eq!(del_plan.files_count, 1);
    assert!(!d.0.join("notes/science").exists());

    undo_folder_op(&mut s, &del_snap).unwrap();
    assert!(d.0.join("notes/science/a.md").is_file());
}

#[test]
fn test_rename_with_link_rewriting() {
    let (d, mut s) = vault("rename");
    write(&d.0, "assets/pic.png", "img_data");
    write(
        &d.0,
        "notes/test.md",
        "---\nid: 01000000000000000000000002\nkind: note\ntitle: Test\n---\nSee ![diag](../assets/pic.png) and [[Test]].\n",
    );

    let (_, snap, plan) = rename_path(&mut s, "assets/pic.png", "diagram.png").unwrap();
    assert_eq!(plan.dst, "assets/diagram.png");
    assert!(d.0.join("assets/diagram.png").is_file());
    assert!(!d.0.join("assets/pic.png").exists());

    let content = fs::read_to_string(d.0.join("notes/test.md")).unwrap();
    assert!(content.contains("![diag](assets/diagram.png)"));
    assert!(content.contains("[[Test]]"));

    undo_folder_op(&mut s, &snap).unwrap();
    assert!(d.0.join("assets/pic.png").is_file());
    assert!(!d.0.join("assets/diagram.png").exists());
}

#[test]
fn test_move_cross_stack_and_references() {
    let (d, mut s) = vault("move");
    let stack_a = "01000000000000000000000010";
    let stack_b = "01000000000000000000000020";
    write(
        &d.0,
        "stack_a/_stack.md",
        &format!("---\nid: {stack_a}\nkind: stack\ntitle: Stack A\n---\n"),
    );
    write(
        &d.0,
        "stack_b/_stack.md",
        &format!("---\nid: {stack_b}\nkind: stack\ntitle: Stack B\n---\n"),
    );
    write(
        &d.0,
        "stack_a/card.md",
        &format!(
            "---\nid: 00000000000000000000000030\nkind: card\ntitle: My Card\nreferenced_by: [{stack_b}]\n---\nFront\n???\nBack\n"
        ),
    );
    write(
        &d.0,
        "doc.md",
        "---\nid: 01000000000000000000000040\nkind: note\ntitle: Doc\n---\n[card](stack_a/card.md)\n",
    );

    studyfrog_store::reindex::reindex(&mut s.conn, &s.root).unwrap();

    let mem = StoredMem {
        state: MemState {
            stability: 2.5,
            difficulty: 4.0,
        },
        due: UnixMs(2_000_000_000_000),
        last_review: UnixMs(1_900_000_000_000),
    };
    let stack_a_uid = studyfrog_md::ulid::parse_ulid(stack_a).unwrap();
    let card_uid = studyfrog_md::ulid::parse_ulid("00000000000000000000000030").unwrap();
    save_mem_state_in(&s.conn, (card_uid, stack_a_uid), &mem).unwrap();

    let plan = plan_move(&s, "stack_a/card.md", "stack_b").unwrap();
    assert!(plan.is_cross_stack);
    assert_eq!(plan.items_changing_primary_stack, 1);
    assert_eq!(plan.schedules_dropped, 1);

    let (_, snap, _) = move_path(&mut s, "stack_a/card.md", "stack_b").unwrap();
    assert!(d.0.join("stack_b/card.md").is_file());
    assert!(!d.0.join("stack_a/card.md").exists());

    let moved_text = fs::read_to_string(d.0.join("stack_b/card.md")).unwrap();
    assert!(!moved_text.contains(stack_b));

    let doc_text = fs::read_to_string(d.0.join("doc.md")).unwrap();
    assert!(doc_text.contains("[card](stack_b/card.md)"));

    let dropped = load_mem_state_in(&s.conn, (card_uid, stack_a_uid)).unwrap();
    assert!(dropped.is_none());

    undo_folder_op(&mut s, &snap).unwrap();
    assert!(d.0.join("stack_a/card.md").is_file());
    let restored_mem = load_mem_state_in(&s.conn, (card_uid, stack_a_uid)).unwrap();
    assert!(restored_mem.is_some());
}

#[test]
fn test_rollback_on_simulated_failure() {
    let (d, mut s) = vault("rollback");
    write(&d.0, "src_dir/note.md", "some text");
    let plan = plan_move(&s, "src_dir/note.md", "").unwrap();

    let err = execute_move_with_fault(&mut s, &plan, Some(1)).unwrap_err();
    assert!(format!("{err}").contains("simulated fault"));

    assert!(d.0.join("src_dir/note.md").is_file());
    assert!(!d.0.join("note.md").exists());
}

#[test]
fn test_rejection_of_traversal_and_root() {
    let (_, mut s) = vault("hostile_traversal");
    assert!(create_folder(&mut s, "").is_err());
    assert!(create_folder(&mut s, "..").is_err());
    assert!(create_folder(&mut s, "../escape").is_err());
    assert!(create_folder(&mut s, "/absolute").is_err());
    assert!(create_folder(&mut s, "C:/windows").is_err());
    assert!(create_folder(&mut s, ".studyfrog").is_err());
    assert!(create_folder(&mut s, ".studyfrog/evil").is_err());
    assert!(delete_path(&mut s, "").is_err());
    assert!(delete_path(&mut s, ".studyfrog").is_err());
}

#[test]
fn test_rejection_of_device_and_special_chars() {
    let (_, mut s) = vault("hostile_chars");
    assert!(create_folder(&mut s, "bad/con/dir").is_err());
    assert!(create_folder(&mut s, "bad/aux.txt").is_err());
    assert!(create_folder(&mut s, "bad/nul").is_err());
    assert!(create_folder(&mut s, "bad/com1").is_err());
    assert!(create_folder(&mut s, "bad/foo:bar").is_err());
    assert!(create_folder(&mut s, "bad/foo*bar").is_err());
    assert!(create_folder(&mut s, "bad/foo?bar").is_err());
}
