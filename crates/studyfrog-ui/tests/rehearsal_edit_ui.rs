//! Tests for in-rehearsal item editing.

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::create_more::NewKind;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::Modal;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::rehearsal_edit::{RehearsalEditEffect, RehearsalEditMsg};
use studyfrog_ui::state::State;

const CARD_TEXT: &str = "---\nid: 00000000000000000000000001\nkind: card\ntitle: Original Title\ntags: [alpha]\n---\nFront question\n???\nBack answer\n";

fn sample_card() -> LoadedDoc {
    let doc = match parse_doc(CARD_TEXT) {
        Ok(d) => d,
        Err(_) => panic!("valid card doc"),
    };
    LoadedDoc {
        path: "cards/card1.md".to_owned(),
        doc,
        hash: 42,
    }
}

fn open_modal(uid: Uid) -> State {
    let state = State::default();
    let (state, _) =
        studyfrog_ui::update::update(state, Msg::RehearsalEdit(RehearsalEditMsg::Open(uid)));
    let loaded = sample_card();
    let (state, _) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::Loaded(uid, Box::new(Ok(loaded)))),
    );
    state
}

#[test]
fn rehearsal_edit_flow_opens_and_modifies() {
    let uid = Uid(1);
    let state = State::default();
    let (_, effects) =
        studyfrog_ui::update::update(state, Msg::RehearsalEdit(RehearsalEditMsg::Open(uid)));
    assert_eq!(
        effects,
        vec![Effect::RehearsalEdit(RehearsalEditEffect::Load(uid))]
    );

    let state = open_modal(uid);
    let Some(Modal::EditItem(modal)) = &state.modal else {
        panic!("expected Modal::EditItem");
    };
    assert_eq!(modal.title, "Original Title");
    assert_eq!(modal.text, "Front question");
    assert_eq!(modal.answer, "Back answer\n");
    assert_eq!(modal.kind, NewKind::Card);

    let (state, _) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::EditTitle("Updated Title".to_owned())),
    );
    let Some(Modal::EditItem(modal)) = &state.modal else {
        panic!("expected Modal::EditItem");
    };
    assert_eq!(modal.title, "Updated Title");
}

#[test]
fn rehearsal_edit_flow_saves_and_reloads() {
    let uid = Uid(1);
    let state = open_modal(uid);
    let (state, _) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::EditTitle("Updated Title".to_owned())),
    );

    let (state, effects) =
        studyfrog_ui::update::update(state, Msg::RehearsalEdit(RehearsalEditMsg::Save));
    assert_eq!(effects.len(), 1);
    let Effect::RehearsalEdit(RehearsalEditEffect::Save(save_uid, path, doc, hash)) = &effects[0]
    else {
        panic!("expected RehearsalEditEffect::Save");
    };
    assert_eq!(*save_uid, uid);
    assert_eq!(path, "cards/card1.md");
    assert_eq!(*hash, 42);
    if let Doc::Card(c) = &**doc {
        assert_eq!(c.meta.title, "Updated Title");
    } else {
        panic!("expected Doc::Card");
    }

    let (state, effects) =
        studyfrog_ui::update::update(state, Msg::RehearsalEdit(RehearsalEditMsg::Saved(Ok(uid))));
    assert!(state.modal.is_none());
    assert_eq!(effects, vec![Effect::LoadItem(uid)]);
    assert!(
        state
            .notice
            .as_ref()
            .is_some_and(|n| n.text.contains("updated"))
    );
}
