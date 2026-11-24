#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The stack browser: pure update logic, and the whole flow against a real vault.
use common::{run_to_idle, temp_dir};
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::modal::{Field, Modal, ModalMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::MsgBrowse;
use studyfrog_ui::msg::MsgVault;
use studyfrog_ui::msg::{DraftField, Msg};
use studyfrog_ui::state::Selection;

mod common;

#[test]
fn a_user_can_build_a_stack_with_cards_from_an_empty_vault() {
    let vault = temp_dir("build");
    let mut worker = worker_new(None);
    let mut m = Model::default();
    let fx = apply(
        &mut m,
        Msg::Vault(MsgVault::OpenRecent(vault.path.display().to_string())),
    );
    run_to_idle(&mut m, &mut worker, fx);
    assert!(m.state.vault.as_ref().unwrap().overview.stacks.is_empty());

    apply(&mut m, Msg::Modal(ModalMsg::OpenCreateStack));
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(Field::Title, "Spanish".into())),
    );
    let fx = apply(&mut m, Msg::Modal(ModalMsg::Submit));
    run_to_idle(&mut m, &mut worker, fx);
    let stacks = &m.state.vault.as_ref().unwrap().overview.stacks;
    assert_eq!((stacks.len(), stacks[0].folder.as_str()), (1, "spanish"));
    assert!(m.state.modal.is_none(), "the dialog closes when it is done");
    assert_eq!(
        m.state.browser.selection,
        Selection::Stack("spanish".into()),
        "the new stack is opened"
    );

    let fx = apply(
        &mut m,
        Msg::Browse(MsgBrowse::Select(Selection::Stack("spanish".into()))),
    );
    run_to_idle(&mut m, &mut worker, fx);
    for (front, back) in [("hola?", "hello"), ("adios?", "goodbye")] {
        apply(
            &mut m,
            Msg::Browse(MsgBrowse::DraftEdited(DraftField::Front, front.into())),
        );
        apply(
            &mut m,
            Msg::Browse(MsgBrowse::DraftEdited(DraftField::Back, back.into())),
        );
        let fx = apply(&mut m, Msg::Browse(MsgBrowse::AddCardClicked));
        run_to_idle(&mut m, &mut worker, fx);
    }
    let titles: Vec<_> = m
        .state
        .browser
        .items
        .iter()
        .map(|i| i.title.as_str())
        .collect();
    assert_eq!(titles, ["adios?", "hola?"], "listed in file-name order");
    assert_eq!(m.state.vault.as_ref().unwrap().overview.stacks[0].cards, 2);
    assert!(m.state.vault.as_ref().unwrap().overview.problems.is_empty());
    assert!(vault.path.join("spanish/hola.md").is_file());
}

#[test]
fn creating_with_a_bad_title_reports_the_reason_and_keeps_the_form() {
    let vault = temp_dir("bad");
    let mut worker = worker_new(None);
    let mut m = Model::default();
    let fx = apply(
        &mut m,
        Msg::Vault(MsgVault::OpenRecent(vault.path.display().to_string())),
    );
    run_to_idle(&mut m, &mut worker, fx);
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreateStack));
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(Field::Title, "line one\nline two".into())),
    );
    let fx = apply(&mut m, Msg::Modal(ModalMsg::Submit));
    run_to_idle(&mut m, &mut worker, fx);
    let Some(Modal::Create(form)) = &m.state.modal else {
        panic!("the dialog stays open")
    };
    assert!(
        form.error.as_deref().unwrap().contains("single line"),
        "{:?}",
        form.error
    );
    assert!(
        form.title.contains("line one") && !form.busy,
        "what was typed is kept"
    );
}
