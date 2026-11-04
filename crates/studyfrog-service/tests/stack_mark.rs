#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Marking a normal folder as a stack, and unmarking a stack into a normal folder (TASK-0217).

use std::fs;
use std::path::PathBuf;

use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::create::IdSource;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_service::stack_mark::{
    list_normal_folders, mark_as_stack, plan_unmark, undo_mark_change, unmark_stack,
};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-mark-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn ids(n: u8) -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000 + u64::from(n),
        random: [n; 10],
    }
}

fn write(dir: &std::path::Path, rel: &str, content: &str) {
    let file = dir.join(rel);
    if let Some(p) = file.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&file, content).unwrap();
}

#[test]
fn plan_unmark_reports_everything_that_changes() {
    let (d, mut s) = vault("plan");
    let stack_b_id = "00000000000000000000000002";
    write(
        &d.0,
        "french/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\n---\n",
    );
    write(
        &d.0,
        "french/verbs/_stack.md",
        &format!(
            "---\nid: {stack_b_id}\nkind: stack\ntitle: Verbs\nstatus: learning\nsubjects: [Grammar]\n---\nDescription\n\n## Goals\n- Master irregular verbs\n- Learn conjugations\n"
        ),
    );
    write(
        &d.0,
        "french/verbs/_stack.toml",
        "[scheduler]\nalgorithm = \"sm-2\"\n",
    );
    write(
        &d.0,
        "french/verbs/irregular/_stack.md",
        "---\nid: 00000000000000000000000003\nkind: stack\ntitle: Irregular\n---\n",
    );
    write(
        &d.0,
        "french/verbs/avoir.md",
        "---\nid: 00000000000000000000000010\nkind: card\ntitle: Avoir\n---\nPrompt\n???\nAnswer\n",
    );
    write(
        &d.0,
        "german/shared.md",
        &format!(
            "---\nid: 00000000000000000000000020\nkind: card\ntitle: Shared\nreferenced_by: [{stack_b_id}]\n---\nQ\n???\nA\n"
        ),
    );
    write(
        &d.0,
        "german/_stack.md",
        "---\nid: 00000000000000000000000004\nkind: stack\ntitle: German\n---\n",
    );

    studyfrog_store::reindex::reindex(&mut s.conn, &s.root).unwrap();

    let plan = plan_unmark(&s, "french/verbs").unwrap();
    assert_eq!(plan.folder, "french/verbs");
    assert_eq!(plan.title, "Verbs");
    assert_eq!(plan.uid, Uid(2));
    assert!(plan.has_settings);
    assert_eq!(plan.status, Some(Status::Learning));
    assert_eq!(plan.goals_count, 2);
    assert_eq!(plan.subjects, vec!["Grammar"]);
    assert_eq!(plan.sub_stacks, vec!["french/verbs/irregular"]);
    assert_eq!(plan.items_count, 1);
    assert_eq!(plan.target_folder.as_deref(), Some("french"));
    assert_eq!(plan.target_title, "French");
    assert_eq!(plan.referencing_items.len(), 1);
    assert_eq!(plan.referencing_items[0].1, "german/shared.md");
}

#[test]
fn unmarking_a_stack_removes_mark_drops_references_and_can_be_undone() {
    let (d, mut s) = vault("unmark");
    let stack_b_id = format_ulid(Uid(2));
    write(
        &d.0,
        "french/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\n---\n",
    );
    write(
        &d.0,
        "french/verbs/_stack.md",
        &format!("---\nid: {stack_b_id}\nkind: stack\ntitle: Verbs\n---\n"),
    );
    write(
        &d.0,
        "french/verbs/_stack.toml",
        "[scheduler]\nalgorithm = \"sm-2\"\n",
    );
    write(
        &d.0,
        "french/verbs/avoir.md",
        "---\nid: 00000000000000000000000010\nkind: card\ntitle: Avoir\n---\nPrompt\n???\nAnswer\n",
    );
    write(
        &d.0,
        "german/shared.md",
        &format!(
            "---\nid: 00000000000000000000000020\nkind: card\ntitle: Shared\nreferenced_by: [{stack_b_id}]\n---\nQ\n???\nA\n"
        ),
    );
    write(
        &d.0,
        "german/_stack.md",
        "---\nid: 00000000000000000000000004\nkind: stack\ntitle: German\n---\n",
    );

    studyfrog_store::reindex::reindex(&mut s.conn, &s.root).unwrap();

    let (info, snap, plan) = unmark_stack(&mut s, "french/verbs").unwrap();
    assert_eq!(plan.title, "Verbs");
    assert!(!d.0.join("french/verbs/_stack.md").exists());
    assert!(!d.0.join("french/verbs/_stack.toml").exists());

    // German shared item no longer references French/Verbs
    let german_text = fs::read_to_string(d.0.join("german/shared.md")).unwrap();
    assert!(!german_text.contains(&stack_b_id), "{german_text}");

    // Overview now shows French/Verbs is gone, and French has the avoir item
    let verb_stack = info
        .overview
        .stacks
        .iter()
        .find(|st| st.folder == "french/verbs");
    assert!(verb_stack.is_none());

    // Undo restores the stack
    let restored = undo_mark_change(&mut s, &snap).unwrap();
    assert!(d.0.join("french/verbs/_stack.md").is_file());
    assert!(d.0.join("french/verbs/_stack.toml").is_file());
    let german_restored = fs::read_to_string(d.0.join("german/shared.md")).unwrap();
    assert!(german_restored.contains(&stack_b_id));
    assert!(
        restored
            .overview
            .stacks
            .iter()
            .any(|st| st.folder == "french/verbs")
    );
}

#[test]
fn marking_a_normal_folder_as_a_stack_adopts_its_items_and_can_be_undone() {
    let (d, mut s) = vault("mark");
    write(
        &d.0,
        "scratch/note.md",
        "---\nid: 00000000000000000000000010\nkind: note\ntitle: Scratch note\n---\nHello\n",
    );
    studyfrog_store::reindex::reindex(&mut s.conn, &s.root).unwrap();

    let normals = list_normal_folders(&s).unwrap();
    assert_eq!(normals, vec!["scratch"]);

    let (info, snap, created) = mark_as_stack(&mut s, "scratch", ids(5)).unwrap();
    assert_eq!(created.title, "scratch");
    assert_eq!(created.path, "scratch/_stack.md");
    assert!(d.0.join("scratch/_stack.md").is_file());

    let stack = info
        .overview
        .stacks
        .iter()
        .find(|st| st.folder == "scratch");
    assert!(stack.is_some());
    assert_eq!(stack.unwrap().notes, 1);

    // Undo removes the mark
    let restored = undo_mark_change(&mut s, &snap).unwrap();
    assert!(!d.0.join("scratch/_stack.md").exists());
    assert!(
        restored
            .overview
            .stacks
            .iter()
            .all(|st| st.folder != "scratch")
    );
}
