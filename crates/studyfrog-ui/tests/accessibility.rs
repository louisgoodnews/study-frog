#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Accessibility audit of every screen through the AccessKit tree.

mod common;

use common::views::{assert_all_named, harness, open_model};
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen};

#[test]
fn every_control_on_every_screen_has_an_accessible_name() {
    let mut welcome = Model::default();
    welcome.state.app.recent = vec!["/v/old".into()];
    let mut h = harness(welcome);
    h.run();
    assert_all_named(&h, "welcome");

    // The vault dialog is where the path field lives now.
    let mut dialog = Model::default();
    dialog.state.app.recent = vec!["/v/old".into()];
    dialog.state.modal = Some(studyfrog_ui::modal::Modal::Vault);
    let mut h = harness(dialog);
    h.run();
    assert_all_named(&h, "vault dialog");
    let field = h.get_by_role(Role::TextInput);
    assert_eq!(
        field.accesskit_node().label().as_deref(),
        Some("Vault folder"),
        "the path field is labelled"
    );

    for screen in [Screen::Library, Screen::Rehearse, Screen::Settings] {
        let mut m = open_model();
        apply(&mut m, Msg::Navigate(screen));
        m.state.notice = Some(studyfrog_ui::state::Notice {
            kind: NoticeKind::Error,
            text: "x".into(),
        });
        let mut h = harness(m);
        h.run();
        assert_all_named(&h, &format!("{screen:?}"));
        assert!(
            h.query_all_by_role(Role::Button).count() >= 4,
            "{screen:?}: nav, close and dismiss are buttons"
        );
    }
}
