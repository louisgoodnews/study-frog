#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The stack browser: pure update logic, and the whole flow against a real vault.

mod common;

use common::{run_to_idle, temp_dir};
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_ui::browse::filter_items;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{DraftField, Msg};
use studyfrog_ui::state::{ItemRow, NoticeKind, Selection};
use studyfrog_ui::update::update;

fn row(n: u128, title: &str) -> ItemRow {
    ItemRow {
        uid: Uid(n),
        title: title.into(),
        kind: ItemKind::Card,
        path: format!("s/{n}.md"),
    }
}

#[test]
fn title_filter_ignores_case_and_surrounding_spaces_and_handles_unicode() {
    let items = [
        row(1, "Bonjour"),
        row(2, "Au revoir"),
        row(3, "ÄPFEL und Birnen"),
    ];
    let titles = |q: &str| {
        filter_items(&items, q)
            .iter()
            .map(|i| i.uid.0)
            .collect::<Vec<_>>()
    };
    assert_eq!(titles(""), [1, 2, 3]);
    assert_eq!(titles("  BON "), [1]);
    assert_eq!(titles("voir"), [2]);
    assert_eq!(titles("äpfel"), [3]);
    assert!(titles("zzz").is_empty());
}

#[test]
fn selecting_loads_items_and_resets_the_filter() {
    let mut m = Model::default();
    apply(&mut m, Msg::SearchEdited("old".into()));
    let fx = apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    assert_eq!(fx, [Effect::LoadItems(Selection::Stack("french".into()))]);
    assert!(m.state.browser.search.is_empty() && m.state.browser.items.is_empty());
    assert!(
        apply(&mut m, Msg::Select(Selection::Nothing)).is_empty(),
        "the overview needs no items"
    );
    apply(&mut m, Msg::ItemsLoaded(Ok(vec![row(1, "A")])));
    assert_eq!(m.state.browser.items.len(), 1);
    apply(&mut m, Msg::ItemsLoaded(Err("disk".into())));
    assert_eq!(m.state.notice.as_ref().unwrap().kind, NoticeKind::Error);
}

#[test]
fn the_forms_validate_before_asking_for_anything() {
    let mut m = Model::default();
    assert!(apply(&mut m, Msg::CreateStackClicked).is_empty());
    assert!(m.state.notice.as_ref().unwrap().text.contains("title"));
    apply(&mut m, Msg::NewStackEdited("  Maths ".into()));
    assert_eq!(
        apply(&mut m, Msg::CreateStackClicked),
        [Effect::CreateStack("Maths".into())]
    );

    assert!(
        apply(&mut m, Msg::AddCardClicked).is_empty(),
        "no stack open"
    );
    assert!(m.state.notice.as_ref().unwrap().text.contains("stack"));
    apply(&mut m, Msg::Select(Selection::Stack("maths".into())));
    apply(&mut m, Msg::DraftEdited(DraftField::Front, "2+2?".into()));
    assert!(apply(&mut m, Msg::AddCardClicked).is_empty(), "no back yet");
    apply(&mut m, Msg::DraftEdited(DraftField::Back, "4".into()));
    apply(&mut m, Msg::DraftEdited(DraftField::Title, "Sum".into()));
    let fx = apply(&mut m, Msg::AddCardClicked);
    assert_eq!(
        fx,
        [Effect::CreateCard {
            folder: "maths".into(),
            title: "Sum".into(),
            front: "2+2?".into(),
            back: "4".into()
        }]
    );
}

#[test]
fn creating_succeeds_clears_the_forms_and_refreshes_the_open_stack() {
    let mut m = Model::default();
    apply(&mut m, Msg::Select(Selection::Stack("maths".into())));
    apply(&mut m, Msg::NewStackEdited("X".into()));
    apply(&mut m, Msg::DraftEdited(DraftField::Front, "f".into()));
    let info = studyfrog_service_info();
    let (state, fx) = update(m.state.clone(), Msg::Created(Ok(info)));
    assert!(state.browser.new_stack.is_empty() && state.browser.draft.front.is_empty());
    assert_eq!(fx, [Effect::LoadItems(Selection::Stack("maths".into()))]);
    let (state, fx) = update(m.state, Msg::Created(Err("exists".into())));
    assert!(
        fx.is_empty() && state.browser.new_stack == "X",
        "a failure keeps what the user typed"
    );
}

fn studyfrog_service_info() -> studyfrog_ui::state::VaultInfo {
    studyfrog_ui::state::VaultInfo {
        root: "/v".into(),
        name: "v".into(),
        overview: Default::default(),
        settings: studyfrog_core::settings::vault_settings_default(),
        settings_error: None,
    }
}

#[test]
fn a_user_can_build_a_stack_with_cards_from_an_empty_vault() {
    let vault = temp_dir("build");
    let mut worker = worker_new(None);
    let mut m = Model::default();
    let fx = apply(&mut m, Msg::OpenRecent(vault.path.display().to_string()));
    run_to_idle(&mut m, &mut worker, fx);
    assert!(m.state.vault.as_ref().unwrap().overview.stacks.is_empty());

    apply(&mut m, Msg::NewStackEdited("Spanish".into()));
    let fx = apply(&mut m, Msg::CreateStackClicked);
    run_to_idle(&mut m, &mut worker, fx);
    let stacks = &m.state.vault.as_ref().unwrap().overview.stacks;
    assert_eq!((stacks.len(), stacks[0].folder.as_str()), (1, "spanish"));
    assert!(m.state.browser.new_stack.is_empty());

    let fx = apply(&mut m, Msg::Select(Selection::Stack("spanish".into())));
    run_to_idle(&mut m, &mut worker, fx);
    for (front, back) in [("hola?", "hello"), ("adios?", "goodbye")] {
        apply(&mut m, Msg::DraftEdited(DraftField::Front, front.into()));
        apply(&mut m, Msg::DraftEdited(DraftField::Back, back.into()));
        let fx = apply(&mut m, Msg::AddCardClicked);
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
    let fx = apply(&mut m, Msg::OpenRecent(vault.path.display().to_string()));
    run_to_idle(&mut m, &mut worker, fx);
    apply(&mut m, Msg::NewStackEdited("line one\nline two".into()));
    let fx = apply(&mut m, Msg::CreateStackClicked);
    run_to_idle(&mut m, &mut worker, fx);
    assert!(
        m.state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("single line")
    );
    assert!(m.state.browser.new_stack.contains("line one"));
}
