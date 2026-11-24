#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The stack browser: pure update logic, and the whole flow against a real vault.
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_ui::browse::filter_items;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectBrowse;
use studyfrog_ui::effect::EffectImport;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::MsgBrowse;
use studyfrog_ui::msg::{DraftField, Msg};
use studyfrog_ui::state::{ItemRow, NoticeKind, Selection};
use studyfrog_ui::update::update;

mod common;

fn row(n: u128, title: &str) -> ItemRow {
    ItemRow {
        uid: Uid(n),
        title: title.into(),
        kind: ItemKind::Card,
        path: format!("s/{n}.md"),
        due: None,
        status: studyfrog_core::status::Status::New,
    }
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
    apply(&mut m, Msg::Browse(MsgBrowse::SearchEdited("old".into())));
    let fx = apply(
        &mut m,
        Msg::Browse(MsgBrowse::Select(Selection::Stack("french".into()))),
    );
    assert_eq!(
        fx,
        [
            Effect::Browse(EffectBrowse::LoadItems(Selection::Stack("french".into()))),
            Effect::Browse(EffectBrowse::LoadStatusCounts(Selection::Stack(
                "french".into()
            )))
        ]
    );
    assert!(m.state.browser.search.is_empty() && m.state.browser.items.is_empty());
    assert!(
        apply(&mut m, Msg::Browse(MsgBrowse::Select(Selection::Nothing))).is_empty(),
        "the overview needs no items"
    );
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::ItemsLoaded(Ok(vec![row(1, "A")]))),
    );
    assert_eq!(m.state.browser.items.len(), 1);
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::ItemsLoaded(Err("disk".into()))),
    );
    assert_eq!(m.state.notice.as_ref().unwrap().kind, NoticeKind::Error);
}

#[test]
fn the_forms_validate_before_asking_for_anything() {
    let mut m = Model::default();
    assert!(
        apply(&mut m, Msg::Browse(MsgBrowse::AddCardClicked)).is_empty(),
        "no stack open"
    );
    assert!(m.state.notice.as_ref().unwrap().text.contains("stack"));
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::Select(Selection::Stack("maths".into()))),
    );
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::DraftEdited(DraftField::Front, "2+2?".into())),
    );
    assert!(
        apply(&mut m, Msg::Browse(MsgBrowse::AddCardClicked)).is_empty(),
        "no back yet"
    );
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::DraftEdited(DraftField::Back, "4".into())),
    );
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::DraftEdited(DraftField::Title, "Sum".into())),
    );
    let fx = apply(&mut m, Msg::Browse(MsgBrowse::AddCardClicked));
    assert_eq!(
        fx,
        [Effect::Import(EffectImport::CreateCard {
            folder: "maths".into(),
            title: "Sum".into(),
            front: "2+2?".into(),
            back: "4".into()
        })]
    );
}

#[test]
fn creating_succeeds_clears_the_forms_and_refreshes_the_open_stack() {
    let mut m = Model::default();
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::Select(Selection::Stack("maths".into()))),
    );
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::DraftEdited(DraftField::Front, "f".into())),
    );
    let info = studyfrog_service_info();
    let (state, fx) = update(m.state.clone(), Msg::Browse(MsgBrowse::Created(Ok(info))));
    let fx: Vec<Effect> = fx
        .into_iter()
        .filter(|e| !matches!(e, Effect::Notifications(_)))
        .collect();
    assert!(state.browser.draft.front.is_empty());
    assert_eq!(
        fx,
        [
            Effect::Browse(EffectBrowse::LoadItems(Selection::Stack("maths".into()))),
            Effect::Browse(EffectBrowse::LoadStatusCounts(Selection::Stack(
                "maths".into()
            )))
        ]
    );
    let (state, fx) = update(
        m.state,
        Msg::Browse(MsgBrowse::Created(Err("exists".into()))),
    );
    assert!(
        fx.is_empty() && state.browser.draft.front == "f",
        "a failure keeps what the user typed"
    );
}
