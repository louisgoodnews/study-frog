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
    let doc = parse_doc(CARD_TEXT).expect("valid card doc");
    LoadedDoc {
        path: "cards/card1.md".to_owned(),
        doc,
        hash: 42,
    }
}

#[test]
fn rehearsal_edit_flow_opens_saves_and_reloads() {
    let state = State::default();
    let uid = Uid(1);

    // 1. Open edit request emits Load effect
    let (state, effects) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::Open(uid)),
    );
    assert_eq!(
        effects,
        vec![Effect::RehearsalEdit(RehearsalEditEffect::Load(uid))]
    );

    // 2. Document loaded opens modal
    let loaded = sample_card();
    let (state, effects) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::Loaded(uid, Ok(loaded))),
    );
    assert!(effects.is_empty());
    let Some(Modal::EditItem(modal)) = &state.modal else {
        panic!("expected Modal::EditItem");
    };
    assert_eq!(modal.title, "Original Title");
    assert_eq!(modal.text, "Front question");
    assert_eq!(modal.answer, "Back answer\n");
    assert_eq!(modal.kind, NewKind::Card);

    // 3. Edit fields
    let (state, _) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::EditTitle("Updated Title".to_owned())),
    );
    let (state, _) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::EditText("Updated Front".to_owned())),
    );
    let (state, _) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::EditAnswer("Updated Back".to_owned())),
    );

    // 4. Trigger save
    let (state, effects) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::Save),
    );
    assert_eq!(effects.len(), 1);
    let Effect::RehearsalEdit(RehearsalEditEffect::Save(save_uid, path, doc, hash)) = &effects[0] else {
        panic!("expected RehearsalEditEffect::Save");
    };
    assert_eq!(*save_uid, uid);
    assert_eq!(path, "cards/card1.md");
    assert_eq!(*hash, 42);
    if let Doc::Card(c) = doc {
        assert_eq!(c.meta.title, "Updated Title");
        assert_eq!(c.front, "Updated Front");
        assert_eq!(c.back, "Updated Back");
    } else {
        panic!("expected Doc::Card");
    }

    // 5. Save succeeds: closes modal and emits LoadItem to refresh rehearsal card
    let (state, effects) = studyfrog_ui::update::update(
        state,
        Msg::RehearsalEdit(RehearsalEditMsg::Saved(Ok(uid))),
    );
    assert!(state.modal.is_none());
    assert_eq!(effects, vec![Effect::LoadItem(uid)]);
    assert!(state.notice.as_ref().is_some_and(|n| n.text.contains("updated")));
}
