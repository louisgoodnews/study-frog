#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Importing cards and making them with a registered card type.
use studyfrog_ui::effect::EffectBrowse;
use studyfrog_ui::effect::EffectImport;
use studyfrog_ui::msg::MsgBrowse;
use studyfrog_ui::msg::MsgImport;
use studyfrog_ui::msg::MsgVault;

mod common;

use common::ext::model_in_stack;
use studyfrog_service::import::ImportReport;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::import_ui::ImportOutcome;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::{DraftField, Msg};
use studyfrog_ui::state::NoticeKind;

#[test]
fn the_card_type_and_the_import_path_drive_the_right_effects() {
    let mut m = model_in_stack();
    apply(
        &mut m,
        Msg::Vault(MsgVault::CardTypeChosen(Some("twice".into()))),
    );
    assert!(
        apply(&mut m, Msg::Browse(MsgBrowse::AddCardClicked)).is_empty(),
        "no source text yet"
    );
    assert!(
        m.state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("text to make cards from")
    );
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::DraftEdited(DraftField::Front, "word".into())),
    );
    let fx = apply(&mut m, Msg::Browse(MsgBrowse::AddCardClicked));
    assert_eq!(
        fx,
        [Effect::Import(EffectImport::CreateFromType {
            folder: "french".into(),
            type_id: "twice".into(),
            source: "word".into()
        })]
    );
    apply(&mut m, Msg::Vault(MsgVault::CardTypeChosen(None)));
    assert!(m.state.browser.card_type.is_none());

    assert!(
        apply(&mut m, Msg::Import(MsgImport::ImportClicked)).is_empty(),
        "no file yet"
    );
    assert_eq!(
        apply(&mut m, Msg::Import(MsgImport::BrowseImportClicked)),
        [Effect::Import(EffectImport::PickImportFile)]
    );
    apply(&mut m, Msg::Import(MsgImport::ImportFilePicked(None)));
    assert!(
        m.state.browser.import_path.is_empty(),
        "cancelling changes nothing"
    );
    apply(
        &mut m,
        Msg::Import(MsgImport::ImportFilePicked(Some("/tmp/deck.csv".into()))),
    );
    assert_eq!(
        apply(&mut m, Msg::Import(MsgImport::ImportClicked)),
        [Effect::Import(EffectImport::ImportFile {
            folder: "french".into(),
            path: "/tmp/deck.csv".into()
        })]
    );
}

#[test]
fn the_outcome_is_reported_and_the_stack_is_reloaded() {
    let mut m = model_in_stack();
    m.state.browser.import_path = "/x.csv".into();
    let info = m.state.vault.clone().unwrap();
    let ok = ImportOutcome {
        info: info.clone(),
        report: ImportReport {
            created: 3,
            titles: Vec::new(),
            skipped: vec!["card 2: bad".into()],
        },
    };
    let fx = apply(&mut m, Msg::Import(MsgImport::Imported(Ok(ok))));
    assert!(matches!(
        &fx[..],
        [
            Effect::Browse(EffectBrowse::LoadItems(_)),
            Effect::Browse(EffectBrowse::LoadStatusCounts(_))
        ]
    ));
    let n = m.state.notice.clone().unwrap();
    assert_eq!(n.kind, NoticeKind::Info);
    assert_eq!(n.text, "Created 3 card(s). Skipped 1: card 2: bad");
    assert!(m.state.browser.import_path.is_empty());

    let none = ImportOutcome {
        info,
        report: ImportReport {
            created: 0,
            titles: Vec::new(),
            skipped: vec!["card 1: bad".into()],
        },
    };
    apply(&mut m, Msg::Import(MsgImport::Imported(Ok(none))));
    assert_eq!(
        m.state.notice.as_ref().unwrap().kind,
        NoticeKind::Error,
        "nothing created is an error"
    );
    apply(&mut m, Msg::Import(MsgImport::Imported(Err("boom".into()))));
    assert!(m.state.notice.as_ref().unwrap().text.contains("boom"));
}
