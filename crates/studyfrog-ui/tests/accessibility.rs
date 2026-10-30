#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Accessibility audit of every screen through the AccessKit tree.

mod common;

use common::views::{App, harness, open_model};
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen};

/// Every control a screen offers must be reachable by a screen reader, so it needs a name.
fn assert_all_named(h: &Harness<'_, App>, screen: &str) {
    for role in [Role::Button, Role::TextInput, Role::Label] {
        for node in h.query_all_by_role(role) {
            // Static text carries its content as `value`; controls must have a real `label`.
            let n = node.accesskit_node();
            let name = if role == Role::Label {
                n.label().or_else(|| n.value())
            } else {
                n.label()
            };
            let name = name.unwrap_or_default();
            assert!(
                !name.trim().is_empty(),
                "{screen}: a {role:?} has no accessible name"
            );
        }
    }
}

#[test]
fn every_control_on_every_screen_has_an_accessible_name() {
    let mut welcome = Model::default();
    welcome.state.app.recent = vec!["/v/old".into()];
    let mut h = harness(welcome);
    h.run();
    assert_all_named(&h, "welcome");
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
