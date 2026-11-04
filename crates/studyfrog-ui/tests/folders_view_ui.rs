#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless UI and interaction tests for the Folders file explorer view (TASK-0204).

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::folders_types::{EntryKind, FolderEntry, FoldersKey, FoldersMsg, SortOrder};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

#[allow(clippy::too_many_arguments)]
fn entry(
    rel: &str,
    name: &str,
    is_folder: bool,
    is_stack: bool,
    parent: &str,
    depth: usize,
    kind: EntryKind,
    mtime: i64,
) -> FolderEntry {
    FolderEntry {
        rel_path: rel.into(),
        name: name.into(),
        is_folder,
        is_stack,
        parent: parent.into(),
        depth,
        kind,
        mtime_ns: mtime,
    }
}

fn sample_entries() -> Vec<FolderEntry> {
    vec![
        entry(
            "french",
            "french",
            true,
            true,
            "",
            0,
            EntryKind::Folder,
            100,
        ),
        entry(
            "french/verbs.md",
            "verbs.md",
            false,
            false,
            "french",
            1,
            EntryKind::Markdown,
            200,
        ),
        entry(
            "french/guide.pdf",
            "guide.pdf",
            false,
            false,
            "french",
            1,
            EntryKind::Pdf,
            300,
        ),
        entry(
            "assets",
            "assets",
            true,
            false,
            "",
            0,
            EntryKind::Folder,
            50,
        ),
        entry(
            "assets/logo.png",
            "logo.png",
            false,
            false,
            "assets",
            1,
            EntryKind::Image,
            60,
        ),
    ]
}

#[test]
fn folders_navigation_from_sidebar_and_accessible_names() {
    let mut m = open_model();
    m.state.folders.entries = sample_entries();
    m.state.folders.expanded.insert("french".into());
    m.state.folders.expanded.insert("assets".into());

    let mut h = harness(m);
    h.run();
    h.get_by_label("Folders").click();
    h.run();

    assert_eq!(h.state().0.state.screen, Screen::Folders);
    assert_all_named(&h, "folders screen");
    assert!(h.query_by_label("New note").is_some());
    assert!(h.query_by_label("New folder").is_some());
    assert!(h.query_by_label("Expand all").is_some());
    assert!(h.query_by_label("Collapse all").is_some());
}

#[test]
fn folders_tree_renders_badges_and_strips_markdown_extension() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Folders));
    m.state.folders.entries = sample_entries();
    m.state.folders.expanded.insert("french".into());
    m.state.folders.expanded.insert("assets".into());

    let mut h = harness(m);
    h.run();

    // Stack folder has [S] badge, normal folder has [F]
    assert!(h.query_by_label("[S]").is_some());
    assert!(h.query_by_label("[F]").is_some());

    // Markdown extension is stripped in display: "verbs", not "verbs.md"
    assert!(h.query_by_label("verbs").is_some());
    assert!(h.query_by_label("verbs.md").is_none());

    // Attachments show kind badges
    assert!(h.query_by_label("[PDF]").is_some());
    assert!(h.query_by_label("guide.pdf").is_some());
    assert!(h.query_by_label("[IMG]").is_some());
    assert!(h.query_by_label("logo.png").is_some());
}

#[test]
fn folders_toolbar_expand_collapse_and_sort() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Folders));
    m.state.folders.entries = sample_entries();

    let mut h = harness(m);
    h.run();

    // Expand all adds folders to expanded set
    h.get_by_label("Expand all").click();
    h.run();
    assert!(h.state().0.state.folders.expanded.contains("french"));
    assert!(h.state().0.state.folders.expanded.contains("assets"));

    // Collapse all empties expanded set
    h.get_by_label("Collapse all").click();
    h.run();
    assert!(h.state().0.state.folders.expanded.is_empty());

    // Sort order message
    apply(
        &mut h.state_mut().0,
        Msg::Folders(FoldersMsg::SetSort(SortOrder::NameDesc)),
    );
    assert_eq!(h.state().0.state.folders.sort, SortOrder::NameDesc);
}

#[test]
fn folders_keyboard_navigation_and_rename() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Folders));
    m.state.folders.entries = sample_entries();
    m.state.folders.expanded.insert("french".into());

    // Select first entry
    apply(&mut m, Msg::Folders(FoldersMsg::Select("french".into())));
    assert_eq!(m.state.folders.selected.as_deref(), Some("french"));

    // Arrow Down moves to guide.pdf
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Down)),
    );
    assert_eq!(
        m.state.folders.selected.as_deref(),
        Some("french/guide.pdf")
    );

    // Arrow Down moves to verbs.md
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Down)),
    );
    assert_eq!(m.state.folders.selected.as_deref(), Some("french/verbs.md"));

    // Arrow Up moves back to guide.pdf
    apply(&mut m, Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Up)));
    assert_eq!(
        m.state.folders.selected.as_deref(),
        Some("french/guide.pdf")
    );

    // Arrow Up moves back to french
    apply(&mut m, Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Up)));
    assert_eq!(m.state.folders.selected.as_deref(), Some("french"));

    // Arrow Left collapses french
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Left)),
    );
    assert!(!m.state.folders.expanded.contains("french"));

    // Arrow Right expands french
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::KeyAction(FoldersKey::Right)),
    );
    assert!(m.state.folders.expanded.contains("french"));

    // F2 starts inline rename
    apply(&mut m, Msg::Folders(FoldersMsg::KeyAction(FoldersKey::F2)));
    assert_eq!(
        m.state.folders.renaming,
        Some(("french".into(), "french".into()))
    );
}
