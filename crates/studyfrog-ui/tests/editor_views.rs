#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The editor's real widgets, headless.
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectEditor;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::msg::MsgEditor;
use studyfrog_ui::state::Screen;

mod common;

fn editing() -> Model {
    let mut m = open_model();
    let doc = parse_doc(NOTE).unwrap();
    apply(
        &mut m,
        Msg::Editor(MsgEditor::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "s/alpha.md".into(),
                doc,
                hash: 1,
            }),
        )),
    );
    assert_eq!(m.state.screen, Screen::Editor);
    m
}

const NOTE: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Alpha\n---\n# Heading\n\nSee [[Beta]] and **bold**.\n";

#[test]
fn the_form_and_the_live_preview_show_together() {
    let mut h = harness(editing());
    h.get_by_label("Body: preview below").click(); // a note opens in Write; the rendering goes under the text in Split
    h.run_steps(2);
    for label in [
        "Title",
        "Tags (comma separated)",
        "Body",
        "Body: preview",
        "Heading",
        "Beta",
        "bold",
    ] {
        assert!(
            h.query_all_by_label(label).count() >= 1,
            "missing {label:?}"
        );
    }
    assert!(h.query_by_label("unsaved changes").is_none());
}

#[test]
fn typing_marks_the_document_changed_updates_the_preview_and_saves_by_itself() {
    let mut h = harness(editing());
    h.run_steps(2);
    h.get_by_label("Title").focus();
    h.get_by_label("Title").type_text(" v2");
    h.run_steps(2);
    assert!(h.query_by_label("unsaved changes").is_some());
    assert_eq!(
        h.state().0.state.editor.as_ref().unwrap().form.title,
        "Alpha v2"
    );
    h.get_by_label("Title").type_text(" ");
    h.run_steps(2);

    assert!(
        h.state().0.state.editor.as_ref().unwrap().saving.is_some(),
        "the space after the first word finished a token and a save started"
    );
    assert!(
        h.query_by_label("Save").is_none(),
        "there is no Save button"
    );
}

#[test]
fn revert_is_disabled_until_there_is_something_to_revert() {
    let mut h = harness(editing());
    h.run_steps(2);
    h.get_by_label("More actions").click();
    h.run_steps(2);
    h.get_by_label("Revert").click();
    h.run_steps(2);
    assert!(
        !h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::Editor(EffectEditor::SaveDocument { .. }))),
        "a disabled button does nothing"
    );
}

#[test]
fn a_wikilink_in_the_preview_is_clickable() {
    let mut h = harness(editing());
    h.get_by_label("Body: preview below").click();
    h.run_steps(2);
    h.get_all_by_label("Beta").next().unwrap().click();
    h.run_steps(2);
    assert!(
        h.state()
            .1
            .contains(&Effect::Editor(EffectEditor::ResolveLink("Beta".into())))
    );
}

#[test]
fn back_asks_to_close_and_every_control_is_named() {
    let mut h = harness(editing());
    h.run_steps(2);
    assert_all_named(&h, "editor");
    h.get_by_label("More actions").click();
    h.run_steps(2);
    h.get_by_label("Delete item…").click();
    h.run_steps(4);
    assert!(h.query_by_label("Delete item").is_some());
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label("Delete item").is_none());
    h.get_by_label("More actions").click();
    h.run_steps(2);
    h.get_by_label("Delete item…").click();
    h.run_steps(2);
    h.get_by_label("Delete").click();
    h.run_steps(2);
    assert_eq!(h.state().0.state.screen, Screen::Library);
    assert!(h.state().0.state.editor.is_none());
}

#[test]
fn undo_and_redo_work_in_the_text_areas() {
    use eframe::egui::{Key, Modifiers};
    let mut h = harness(editing());
    h.run_steps(2);
    let body = |h: &egui_kittest::Harness<'_, common::views::App>| {
        h.state().0.state.editor.as_ref().unwrap().form.fields[0]
            .text
            .clone()
    };
    let before = body(&h);
    let role = eframe::egui::accesskit::Role::MultilineTextInput;
    h.get_by_role_and_label(role, "Body").focus();
    h.get_by_role_and_label(role, "Body")
        .type_text(" added words");
    h.run_steps(2);
    assert_ne!(body(&h), before);

    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run_steps(3);
    assert_eq!(body(&h), before, "undo restores the earlier text");
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    h.run_steps(3);
    assert!(
        body(&h).ends_with(" added words"),
        "redo brings it back: {:?}",
        body(&h)
    );
}
