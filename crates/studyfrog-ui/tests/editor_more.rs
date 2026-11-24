#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The editor: form model, and the open - edit - save flow against a real vault.
use common::{run_to_idle, temp_dir};
use std::fs;
use studyfrog_core::uid::Uid;
use studyfrog_ui::editor::has_unsaved_changes;
use studyfrog_ui::effect::EffectEditor;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::MsgBrowse;
use studyfrog_ui::msg::MsgEditor;
use studyfrog_ui::msg::MsgVault;
use studyfrog_ui::msg::{EditAction, Msg};
use studyfrog_ui::state::Screen;

mod common;

fn open() -> (Model, Worker, common::TempDir) {
    let dir = temp_dir("editor");
    fs::create_dir_all(dir.path.join("s")).unwrap();
    fs::write(
        dir.path.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    fs::write(dir.path.join("s/alpha.md"), ALPHA).unwrap();
    fs::write(dir.path.join("s/beta.md"), BETA).unwrap();
    let (mut m, mut w) = (Model::default(), worker_new(None));
    let fx = apply(
        &mut m,
        Msg::Vault(MsgVault::OpenRecent(dir.path.display().to_string())),
    );
    run_to_idle(&mut m, &mut w, fx);
    (m, w, dir)
}

fn open_item(m: &mut Model, w: &mut Worker, id: u128) {
    let fx = apply(m, Msg::Browse(MsgBrowse::OpenItem(Uid(id))));
    run_to_idle(m, w, fx);
}

fn edit(m: &mut Model, a: EditAction) {
    apply(m, Msg::Editor(MsgEditor::Edit(a)));
}

const ALPHA: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Alpha\ntags: [one]\ncolour: red   # keep me\n---\nSee [[Beta]] and [[Nowhere]].\n";
const BETA: &str =
    "---\nid: 0000000000000000000000000B\nkind: card\ntitle: Beta\n---\nfront\n???\nback\n";

#[test]
fn revert_and_leaving_with_unsaved_changes() {
    let (mut m, mut w, dir) = open();
    open_item(&mut m, &mut w, 0xA);
    edit(&mut m, EditAction::Tags(String::from("x, y")));
    assert!(has_unsaved_changes(&m.state));
    let fx = apply(&mut m, Msg::Editor(MsgEditor::CloseEditor));
    assert!(
        fx.iter().any(|e| matches!(
            e,
            studyfrog_ui::effect::Effect::Editor(EffectEditor::SaveDocument { .. })
        )),
        "leaving saves first: {fx:?}"
    );
    assert_eq!(
        m.state.screen,
        Screen::Editor,
        "and leaves once it is saved"
    );
    run_to_idle(&mut m, &mut w, fx);
    assert_eq!(m.state.screen, Screen::Library);
    let text = fs::read_to_string(dir.path.join("s/alpha.md")).unwrap();
    assert!(text.contains("x") && text.contains("y"), "{text}");

    open_item(&mut m, &mut w, 0xA);
    edit(&mut m, EditAction::Tags(String::from("z")));
    apply(&mut m, Msg::Editor(MsgEditor::RevertClicked));
    assert!(!has_unsaved_changes(&m.state));
    apply(&mut m, Msg::Editor(MsgEditor::CloseEditor));
    assert_eq!(
        (m.state.screen, m.state.editor.is_none()),
        (Screen::Library, true)
    );
}

#[test]
fn clicking_a_wikilink_opens_the_target_or_says_there_is_none() {
    let (mut m, mut w, _dir) = open();
    open_item(&mut m, &mut w, 0xA);
    let fx = apply(&mut m, Msg::Editor(MsgEditor::OpenLink("beta".into())));
    run_to_idle(&mut m, &mut w, fx);
    let e = m.state.editor.as_ref().unwrap();
    assert_eq!(
        (e.uid, e.form.fields.len()),
        (Uid(0xB), 2),
        "Beta is a card with two sides"
    );

    let fx = apply(&mut m, Msg::Editor(MsgEditor::OpenLink("Nowhere".into())));
    run_to_idle(&mut m, &mut w, fx);
    assert!(
        m.state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("No item is titled \"Nowhere\"")
    );
    assert_eq!(
        m.state.editor.as_ref().unwrap().uid,
        Uid(0xB),
        "the open document is unchanged"
    );
}

#[test]
fn previews_follow_the_text() {
    let (mut m, mut w, _dir) = open();
    open_item(&mut m, &mut w, 0xA);
    edit(
        &mut m,
        EditAction::Field(0, "# Heading\n\n- [[Beta]]\n".into()),
    );
    let blocks = &m.state.editor.as_ref().unwrap().previews[0];
    assert!(matches!(
        &blocks[0],
        studyfrog_md::blocks::Block::Heading { level: 1, .. }
    ));
    assert!(matches!(
        &blocks[1],
        studyfrog_md::blocks::Block::List { .. }
    ));
}
