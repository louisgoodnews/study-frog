#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The import dialog suggests what it found on the computer, and previews the one the user picks.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_service::xfer_discover::{Found, FoundKind};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::transfer_ui::{Tab, TransferJob, TransferMsg};

fn t(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

fn discover_jobs(fx: &[Effect]) -> usize {
    fx.iter()
        .filter(|e| {
            matches!(
                e,
                Effect::Library(LibraryEffect::Transfer(TransferJob::Discover))
            )
        })
        .count()
}

fn vault() -> Found {
    Found {
        kind: FoundKind::ObsidianVault,
        name: "Psych".into(),
        path: "/home/u/Notes/Psych".into(),
        detail: "12 Markdown files".into(),
    }
}

#[test]
fn the_search_runs_once_when_the_import_half_shows_and_not_for_export() {
    let mut m = open_model();
    let fx = apply(&mut m, t(TransferMsg::Open(Tab::Export, None)));
    assert_eq!(discover_jobs(&fx), 0);
    let fx = apply(&mut m, t(TransferMsg::Tab(Tab::Import)));
    assert_eq!(discover_jobs(&fx), 1);
    assert!(m.state.transfer.looking);
    let fx = apply(&mut m, t(TransferMsg::Tab(Tab::Export)));
    assert_eq!(discover_jobs(&fx), 0);
    let fx = apply(&mut m, t(TransferMsg::Tab(Tab::Import)));
    assert_eq!(discover_jobs(&fx), 0, "already asked");
    apply(&mut m, t(TransferMsg::Found(vec![vault()])));
    assert!(!m.state.transfer.looking && m.state.transfer.found.as_ref().unwrap().len() == 1);
    let fx = apply(&mut m, t(TransferMsg::Open(Tab::Import, None)));
    assert_eq!(discover_jobs(&fx), 1, "a new opening looks again");
}

#[test]
fn picking_a_suggestion_sets_the_path_and_asks_for_the_preview() {
    let mut m = open_model();
    apply(&mut m, t(TransferMsg::Open(Tab::Import, None)));
    apply(&mut m, t(TransferMsg::Found(vec![vault()])));
    let fx = apply(&mut m, t(TransferMsg::Pick("/home/u/Notes/Psych".into())));
    assert_eq!(m.state.transfer.path, "/home/u/Notes/Psych");
    assert!(m.state.transfer.busy);
    assert_eq!(
        fx,
        [Effect::Library(LibraryEffect::Transfer(
            TransferJob::Preview("/home/u/Notes/Psych".into(), None, Default::default())
        ))]
    );
}

#[test]
fn the_dialog_lists_what_was_found_or_says_nothing_was() {
    let mut m = open_model();
    apply(&mut m, t(TransferMsg::Open(Tab::Import, None)));
    let mut h = harness(m);
    h.run_steps(6);
    assert!(
        h.query_by_label_contains("Looking for Obsidian vaults")
            .is_some()
    );
    apply_to(&mut h, TransferMsg::Found(Vec::new()));
    assert!(
        h.query_by_label_contains("No Obsidian vault or Anki package found")
            .is_some()
    );
    let anki = Found {
        kind: FoundKind::AnkiProfile,
        name: "User 1".into(),
        path: "/x".into(),
        detail: "export a deck".into(),
    };
    apply_to(&mut h, TransferMsg::Found(vec![vault(), anki]));
    assert!(
        h.query_by_label_contains("Psych · Obsidian vault · 12 Markdown files")
            .is_some()
    );
    assert!(h.query_by_label_contains("User 1 (Anki profile)").is_some());
    h.get_by_role_and_label(
        eframe::egui::accesskit::Role::Button,
        "Psych · Obsidian vault · 12 Markdown files",
    )
    .click();
    h.run_steps(4);
    assert_eq!(h.state().0.state.transfer.path, "/home/u/Notes/Psych");
}

fn apply_to(h: &mut egui_kittest::Harness<'_, common::views::App>, m: TransferMsg) {
    apply(&mut h.state_mut().0, t(m));
    h.run_steps(6);
}
