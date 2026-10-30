#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The real egui views, run headlessly with the kittest harness: clicks, typing, navigation.

mod common;

use common::views::{App, harness, open_model};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use studyfrog_core::settings::AppSettings;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen};

#[test]
fn the_vault_dialog_opens_a_typed_path_and_the_welcome_page_lists_recent_vaults() {
    let mut model = Model::default();
    model.state.app = AppSettings {
        vault: None,
        recent: vec!["/v/old".into()],
        reading_wpm: None,
        answer_engine: Default::default(),
        reading: Default::default(),
    };
    let mut h = harness(model);
    h.run();
    assert!(
        h.query_by_label("Vault folder").is_none(),
        "the path is not the first thing asked"
    );
    h.get_by_label("Open or create a vault…").click();
    h.run();
    h.get_by_label("Vault folder").focus();
    h.get_by_label("Vault folder").type_text("/v/new");
    h.run();
    assert_eq!(h.state().0.state.path_input, "/v/new");

    h.get_by_label("Open vault").click();
    h.run_steps(3); // the spinner keeps repainting, so the loop never settles
    assert_eq!(h.state().1, [Effect::OpenVault("/v/new".into())]);
    assert!(h.state().0.state.opening);
    assert!(
        h.query_by_label("Opening and indexing the vault…")
            .is_some()
    );

    // A recent vault is one click away (new harness: this one is busy opening).
    let mut model = Model::default();
    model.state.app.recent = vec!["/v/old".into()];
    let mut h = harness(model);
    h.run();
    h.get_by_label("/v/old").click();
    h.run_steps(3);
    assert_eq!(h.state().1, [Effect::OpenVault("/v/old".into())]);
}

#[test]
fn library_lists_stacks_counts_inbox_and_problems() {
    let mut h = harness(open_model());
    h.run();
    for text in [
        "notes",
        "French",
        "Maths",
        "2 stacks, 11 items",
        "Inbox: 3 items outside any stack",
        "Problems (1)",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text:?}");
    }
    for text in [
        "sm2 · 20 new / – reviews a day",
        "fsrs (default) · 20 new / – reviews a day",
    ] {
        assert!(
            h.query_by_label_contains(text).is_some(),
            "missing {text:?}"
        );
    }
}

#[test]
fn navigation_switches_screens_and_close_vault_returns_to_welcome() {
    let mut h = harness(open_model());
    h.run();
    h.get_by_label("Settings").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Settings);
    assert!(
        h.query_by_label("Default scheduler").is_some()
            && h.query_by_label("published defaults").is_some()
    );

    h.get_by_label("Rehearse").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Rehearse);
    h.get_by_label("Library").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Library);

    h.get_by_label("Profile and vault").click();
    h.run();
    h.get_by_label("Close vault").click();
    h.run();
    assert_eq!(
        (h.state().0.state.screen, h.state().0.state.vault.is_none()),
        (Screen::Welcome, true)
    );
    assert!(
        h.query_by_label("Library").is_none(),
        "no navigation without a vault"
    );
}

#[test]
fn invalid_vault_settings_are_shown_on_the_settings_screen() {
    let mut m = open_model();
    m.state.vault.as_mut().unwrap().settings_error =
        Some("line 2: scheduler must be \"fsrs\" or \"sm2\"".into());
    apply(&mut m, Msg::Navigate(Screen::Settings));
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label_contains("studyfrog.toml is not used")
            .is_some()
    );
}

#[test]
fn notices_show_in_the_status_bar_and_can_be_dismissed() {
    let mut m = open_model();
    m.state.notice = Some(studyfrog_ui::state::Notice {
        kind: NoticeKind::Error,
        text: "Checking the vault failed: disk gone".into(),
    });
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("Checking the vault failed: disk gone")
            .is_some()
    );
    h.get_by_label("Dismiss").click();
    h.run();
    assert!(h.state().0.state.notice.is_none());
    assert!(
        h.query_by_label("/v/notes").is_some(),
        "the vault path is shown when there is no notice"
    );
}

#[test]
fn ticks_never_stack_polls_and_resume_after_a_reply() {
    let mut h = harness(open_model());
    h.run_steps(12); // 3 seconds of egui time at the harness step of 0.25 s
    let polls = |h: &Harness<'_, App>| {
        h.state()
            .1
            .iter()
            .filter(|e| **e == Effect::PollVault)
            .count()
    };
    assert_eq!(
        polls(&h),
        1,
        "one poll stays outstanding however many ticks pass"
    );

    apply(&mut h.state_mut().0, Msg::VaultPolled(Ok(None)));
    h.run_steps(8);
    assert_eq!(polls(&h), 2, "after the reply, the next tick polls again");
}
