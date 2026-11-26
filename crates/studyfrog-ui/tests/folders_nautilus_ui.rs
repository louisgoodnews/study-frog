#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless UI and logic tests for the Nautilus/Thunar-style Folders redesign (EPIC-074).

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::folders_browse::{BrowseMsg, FoldersViewMode, SortColumn, visible_dir_entries};
use studyfrog_ui::folders_types::{EntryKind, FolderEntry, FoldersMsg, SortOrder};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::{Msg, MsgVault};
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
    size_bytes: u64,
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
        size_bytes,
    }
}

fn sample_entries() -> Vec<FolderEntry> {
    vec![
        entry(
            "diagnostik",
            "diagnostik",
            true,
            true,
            "",
            0,
            EntryKind::Folder,
            200,
            0,
        ),
        entry(
            "diary",
            "diary",
            true,
            false,
            "",
            0,
            EntryKind::Folder,
            100,
            0,
        ),
        entry(
            "lecture.pdf",
            "lecture.pdf",
            false,
            false,
            "",
            0,
            EntryKind::Pdf,
            300,
            2_500_000,
        ),
        entry(
            "diagnostik/ptt.md",
            "ptt.md",
            false,
            false,
            "diagnostik",
            1,
            EntryKind::Markdown,
            400,
            4096,
        ),
        entry(
            "diagnostik/chart.png",
            "chart.png",
            false,
            false,
            "diagnostik",
            1,
            EntryKind::Image,
            350,
            120_000,
        ),
    ]
}

#[test]
fn grid_and_list_views_switch_cleanly_and_keep_accessible_names() {
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Folders)));
    m.state.folders.entries = sample_entries();

    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run();

    // Default is Grid view showing root items and status pill
    assert_eq!(h.state().0.state.folders.view_mode, FoldersViewMode::Grid);
    assert!(h.query_all_by_label("diagnostik").count() >= 1);
    assert!(h.query_all_by_label("lecture.pdf").count() >= 1);
    assert!(h.query_by_label_contains("3 items in view").is_some());
    assert_all_named(&h, "folders grid view");

    // Switch to List view via View Switcher
    h.get_by_label("List view").click();
    h.run();
    assert_eq!(h.state().0.state.folders.view_mode, FoldersViewMode::List);
    assert!(h.query_by_label("Sort by Name").is_some());
    assert!(h.query_by_label("Sort by Type").is_some());
    assert!(h.query_by_label("Sort by Size").is_some());
    assert!(h.query_by_label("Sort by Modified").is_some());
    assert_all_named(&h, "folders list view");

    // Click Sort by Size column header
    h.get_by_label("Sort by Size").click();
    h.run();
    assert_eq!(h.state().0.state.folders.sort, SortOrder::SizeDesc);

    // Click again reverses to SizeAsc
    h.get_by_label("Sort by Size").click();
    h.run();
    assert_eq!(h.state().0.state.folders.sort, SortOrder::SizeAsc);
}

#[test]
fn directory_navigation_breadcrumbs_and_selection_status_pill() {
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Folders)));
    m.state.folders.entries = sample_entries();

    // Enter diagnostik folder
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::Browse(BrowseMsg::OpenDir("diagnostik".into()))),
    );
    assert_eq!(m.state.folders.current_dir, "diagnostik");

    // Select ptt.md and verify status pill shows name and size
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::Select("diagnostik/ptt.md".into())),
    );
    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run();
    assert!(h.query_by_label("Path Vault").is_some());
    assert!(h.query_by_label("Path diagnostik").is_some());
    assert!(h.query_all_by_label("ptt").count() >= 1);
    assert!(
        h.query_by_label_contains("\"ptt\" selected (4.0 KB)")
            .is_some()
    );

    // Back returns to vault root; Forward returns to diagnostik; Up returns to root
    h.get_by_label("Folder back").click();
    h.run();
    assert_eq!(h.state().0.state.folders.current_dir, "");
    h.get_by_label("Folder forward").click();
    h.run();
    assert_eq!(h.state().0.state.folders.current_dir, "diagnostik");
    h.get_by_label("Folder up").click();
    h.run();
    assert_eq!(h.state().0.state.folders.current_dir, "");
}

#[test]
fn filtering_starring_and_sorting_work_consistently() {
    let mut m = open_model();
    m.state.folders.entries = sample_entries();

    // Filter at root
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::Browse(BrowseMsg::SetFilter("lect".into()))),
    );
    let filtered = visible_dir_entries(&m.state.folders.entries, &m.state.folders);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].rel_path, "lecture.pdf");

    // Clear filter and test starring + starred-only view
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::Browse(BrowseMsg::SetFilter(String::new()))),
    );
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::Browse(BrowseMsg::ToggleStar(
            "diagnostik/ptt.md".into(),
        ))),
    );
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::Browse(BrowseMsg::ToggleStarredOnly)),
    );
    let starred = visible_dir_entries(&m.state.folders.entries, &m.state.folders);
    assert_eq!(starred.len(), 1);
    assert_eq!(starred[0].rel_path, "diagnostik/ptt.md");

    // Sort by Kind toggling
    apply(
        &mut m,
        Msg::Folders(FoldersMsg::Browse(BrowseMsg::ToggleSortColumn(
            SortColumn::Kind,
        ))),
    );
    assert_eq!(m.state.folders.sort, SortOrder::KindAsc);
}
