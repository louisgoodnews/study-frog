#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The editor: form model, and the open - edit - save flow against a real vault.

mod common;

use std::fs;

use common::{run_to_idle, temp_dir};
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_md::parse::parse_doc;
use studyfrog_ui::edit_form::{apply_form, form_from_doc};
use studyfrog_ui::editor::{has_unsaved_changes, is_dirty};
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{EditAction, Msg};
use studyfrog_ui::state::{NoticeKind, Screen};

const ALPHA: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Alpha\ntags: [one]\ncolour: red   # keep me\n---\nSee [[Beta]] and [[Nowhere]].\n";
const BETA: &str =
    "---\nid: 0000000000000000000000000B\nkind: card\ntitle: Beta\n---\nfront\n???\nback\n";

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
    let fx = apply(&mut m, Msg::OpenRecent(dir.path.display().to_string()));
    run_to_idle(&mut m, &mut w, fx);
    (m, w, dir)
}

fn open_item(m: &mut Model, w: &mut Worker, id: u128) {
    let fx = apply(m, Msg::OpenItem(Uid(id)));
    run_to_idle(m, w, fx);
}

fn edit(m: &mut Model, a: EditAction) {
    apply(m, Msg::Edit(a));
}

#[test]
fn the_form_shows_the_editable_parts_and_validates_what_comes_back() {
    let Ok(doc @ Doc::Note(_)) = parse_doc(ALPHA) else {
        panic!()
    };
    let mut form = form_from_doc(&doc);
    assert_eq!(
        (form.title.as_str(), form.tags.as_str(), form.fields.len()),
        ("Alpha", "one", 1)
    );
    let Ok(card) = parse_doc(BETA) else { panic!() };
    assert_eq!(form_from_doc(&card).fields[1].label, "Back");
    form.title = "  ".into();
    assert!(apply_form(&doc, &form).unwrap_err().contains("title"));
    form.title = "New\ntitle".into();
    assert!(apply_form(&doc, &form).unwrap_err().contains("single line"));
    form.title = "Fine".into();
    form.tags = "a, b ,, c".into();
    form.fields[0].text = "changed".into();
    let Doc::Note(n) = apply_form(&doc, &form).unwrap() else {
        panic!()
    };
    assert_eq!(
        (n.meta.title.as_str(), n.meta.tags.len(), n.body.as_str()),
        ("Fine", 3, "changed")
    );
    assert_eq!(n.extra.len(), 1, "unknown keys ride along untouched");
}

#[test]
fn open_edit_save_changes_the_file_but_keeps_everything_else() {
    let (mut m, mut w, dir) = open();
    open_item(&mut m, &mut w, 0xA);
    assert_eq!(m.state.screen, Screen::Editor);
    let e = m.state.editor.as_ref().unwrap();
    assert_eq!(e.path, "s/alpha.md");
    assert!(!is_dirty(e));
    assert!(
        apply(&mut m, Msg::SaveClicked).is_empty(),
        "nothing to save yet"
    );

    edit(
        &mut m,
        EditAction::Field(0, "Rewritten, see [[Beta]].\n".into()),
    );
    edit(&mut m, EditAction::Title(" Alpha two".into()));
    assert!(is_dirty(m.state.editor.as_ref().unwrap()));
    let fx = apply(&mut m, Msg::SaveClicked);
    assert!(
        apply(&mut m, Msg::SaveClicked).is_empty(),
        "a second click while saving does nothing"
    );
    run_to_idle(&mut m, &mut w, fx);

    let text = fs::read_to_string(dir.path.join("s/alpha.md")).unwrap();
    assert!(
        text.contains("title: \" Alpha two\"")
            || text.contains("title:  Alpha two")
            || text.contains("Alpha two"),
        "{text}"
    );
    assert!(
        text.contains("colour: red   # keep me"),
        "unknown key and its comment are intact: {text}"
    );
    assert!(text.contains("Rewritten, see [[Beta]]."));
    let e = m.state.editor.as_ref().unwrap();
    assert!(!is_dirty(e) && e.saving.is_none());

    edit(&mut m, EditAction::Field(0, "second edit".into()));
    let fx = apply(&mut m, Msg::SaveClicked);
    run_to_idle(&mut m, &mut w, fx);
    assert!(
        fs::read_to_string(dir.path.join("s/alpha.md"))
            .unwrap()
            .contains("second edit"),
        "the new fingerprint allows saving again"
    );
}

#[test]
fn a_file_changed_elsewhere_blocks_the_save_until_reloaded() {
    let (mut m, mut w, dir) = open();
    open_item(&mut m, &mut w, 0xA);
    edit(&mut m, EditAction::Field(0, "mine".into()));
    fs::write(
        dir.path.join("s/alpha.md"),
        ALPHA.replace("See", "Elsewhere:"),
    )
    .unwrap();
    let fx = apply(&mut m, Msg::SaveClicked);
    run_to_idle(&mut m, &mut w, fx);
    assert_eq!(m.state.notice.as_ref().unwrap().kind, NoticeKind::Error);
    assert!(
        m.state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("outside StudyFrog")
    );
    assert!(
        fs::read_to_string(dir.path.join("s/alpha.md"))
            .unwrap()
            .contains("Elsewhere:"),
        "their edit is intact"
    );
    assert!(
        is_dirty(m.state.editor.as_ref().unwrap()),
        "my edit is still in the form"
    );

    let fx = apply(&mut m, Msg::ReloadClicked);
    run_to_idle(&mut m, &mut w, fx);
    let e = m.state.editor.as_ref().unwrap();
    assert!(!is_dirty(e) && e.form.fields[0].text.contains("Elsewhere:"));
}

#[test]
fn revert_and_leaving_with_unsaved_changes() {
    let (mut m, mut w, dir) = open();
    open_item(&mut m, &mut w, 0xA);
    edit(&mut m, EditAction::Tags(String::from("x, y")));
    assert!(has_unsaved_changes(&m.state));
    let fx = apply(&mut m, Msg::CloseEditor);
    assert!(
        fx.iter()
            .any(|e| matches!(e, studyfrog_ui::effect::Effect::SaveDocument { .. })),
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
    apply(&mut m, Msg::RevertClicked);
    assert!(!has_unsaved_changes(&m.state));
    apply(&mut m, Msg::CloseEditor);
    assert_eq!(
        (m.state.screen, m.state.editor.is_none()),
        (Screen::Library, true)
    );
}

#[test]
fn clicking_a_wikilink_opens_the_target_or_says_there_is_none() {
    let (mut m, mut w, _dir) = open();
    open_item(&mut m, &mut w, 0xA);
    let fx = apply(&mut m, Msg::OpenLink("beta".into()));
    run_to_idle(&mut m, &mut w, fx);
    let e = m.state.editor.as_ref().unwrap();
    assert_eq!(
        (e.uid, e.form.fields.len()),
        (Uid(0xB), 2),
        "Beta is a card with two sides"
    );

    let fx = apply(&mut m, Msg::OpenLink("Nowhere".into()));
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
