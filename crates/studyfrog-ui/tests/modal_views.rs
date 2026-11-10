#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The create dialog, the vault dialog, the toast and the welcome page, drawn headless.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::settings::AppSettings;
use studyfrog_service::create_more::NewKind;
use studyfrog_store::overview::VaultOverview;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::{Modal, ModalMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn with_dialog() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
    m
}

#[test]
fn the_create_dialog_offers_the_four_kinds_with_their_own_fields() {
    let mut h = harness(with_dialog());
    h.run_steps(8);
    assert_all_named(&h, "create dialog");
    h.get_by_label("Type of item").click();
    h.run_steps(8);
    for kind in ["Stack", "Card", "Note", "Question"] {
        assert!(h.query_all_by_label(kind).count() >= 1, "missing {kind}");
    }
    h.get_by_label("Card").click();
    h.run_steps(8);
    assert!(h.query_by_label("More fields").is_some());
    h.get_by_label("More fields").click();
    h.run_steps(4);
    for field in [
        "Title",
        "Front",
        "Back",
        "Parent stack",
        "Priority",
        "Difficulty",
        "Tags",
        "Create another",
        "Create",
        "Cancel",
        "Close",
    ] {
        assert!(
            h.query_all_by_label(field).count() >= 1,
            "a card needs {field}"
        );
    }
    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::SetKind(NewKind::Question)),
    );
    h.run_steps(8);
    assert!(h.query_by_label("Prompt").is_some() && h.query_by_label("Reference answer").is_some());
    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::SetKind(NewKind::Note)),
    );
    h.run_steps(8);
    assert!(h.query_by_label("Body").is_some() && h.query_by_label("Back").is_none());
    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::SetKind(NewKind::Stack)),
    );
    h.run_steps(8);
    assert!(h.query_by_label("Description").is_some());
    assert!(
        h.query_all_by_label("Inside stack (optional)").count() >= 1,
        "a stack may be nested, a note must be in one"
    );
}

#[test]
fn typing_and_pressing_create_sends_the_request() {
    let mut h = harness(with_dialog());
    h.run();
    h.get_by_label("Front").focus();
    h.get_by_label("Front").type_text("Bonjour?");
    h.get_by_label("Back").focus();
    h.get_by_label("Back").type_text("Hello");
    h.run();
    h.get_by_label("Create").click();
    h.run();
    let sent = h.state().1.iter().any(|e| {
        matches!(e, Effect::CreateItem { kind: NewKind::Card, text, answer, parent, .. }
        if text == "Bonjour?" && answer == "Hello" && parent.as_deref() == Some("french"))
    });
    assert!(sent, "{:?}", h.state().1);
}

#[test]
fn an_incomplete_form_shows_why_and_the_dialog_can_be_closed_three_ways() {
    let mut h = harness(with_dialog());
    h.run();
    h.get_by_label("Create").click();
    h.run();
    assert!(
        h.query_by_label("A card needs a front and a back.")
            .is_some()
    );
    h.get_by_label("Cancel").click();
    h.run();
    assert!(h.state().0.state.modal.is_none(), "Cancel closes");

    apply(&mut h.state_mut().0, Msg::Modal(ModalMsg::OpenCreate));
    h.run();
    h.get_by_label("Close").click();
    h.run();
    assert!(h.state().0.state.modal.is_none(), "the cross closes");
}

#[test]
fn without_any_stack_the_dialog_explains_that_cards_need_one() {
    let mut m = open_model();
    m.state.vault.as_mut().unwrap().overview = VaultOverview::default();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
    apply(&mut m, Msg::Modal(ModalMsg::SetKind(NewKind::Card)));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label_contains("create a stack first").is_some());
}

#[test]
fn the_welcome_page_greets_instead_of_asking_for_a_path_and_the_toast_opens_the_vault_dialog() {
    let mut m = Model::default();
    apply(
        &mut m,
        Msg::Started {
            app: AppSettings::default(),
            error: None,
        },
    );
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("Welcome to StudyFrog").is_some());
    assert!(
        h.query_by_label("Vault folder").is_none(),
        "no path field up front"
    );
    assert!(
        h.query_by_label_contains("Set up your vault").is_some(),
        "the toast is on screen"
    );
    assert_all_named(&h, "first launch");
    h.get_by_label_contains("Set up your vault").click();
    h.run();
    assert_eq!(h.state().0.state.modal, Some(Modal::Vault));
    assert!(h.query_by_label("Vault folder").is_some() && h.query_by_label("Open vault").is_some());
    assert!(
        h.query_by_label_contains("Set up your vault").is_none(),
        "the toast went away"
    );
}

#[test]
fn a_toast_can_be_dismissed_without_opening_anything() {
    let mut m = Model::default();
    apply(
        &mut m,
        Msg::Started {
            app: AppSettings::default(),
            error: None,
        },
    );
    let mut h = harness(m);
    h.run();
    h.get_by_label("Dismiss notification").click();
    h.run();
    assert!(h.state().0.state.toasts.is_empty() && h.state().0.state.modal.is_none());
}

#[test]
fn the_library_shows_nested_stacks_as_a_tree_with_the_inner_ones_under_their_parent() {
    let mut m = open_model();
    let v = m.state.vault.as_mut().unwrap();
    let mut verbs = common::views::stack("Verbs", "french/verbs", 0, 1);
    verbs.parent = Some("french".into());
    v.overview.stacks.insert(1, verbs);
    m.state.app.stack_tabs = vec!["french:list".into()];
    apply(
        &mut m,
        Msg::Select(studyfrog_ui::state::Selection::Stack("french".into())),
    );
    let mut h = harness(m);
    h.run();
    assert!(h.query_all_by_label("Verbs").count() >= 1);
    assert!(
        h.query_by_label("Stacks inside:").is_some(),
        "the open stack lists its inner stacks"
    );
    h.get_all_by_label("Verbs").last().unwrap().click();
    h.run();
    assert_eq!(
        h.state().0.state.browser.selection,
        studyfrog_ui::state::Selection::Stack("french/verbs".into())
    );
}
