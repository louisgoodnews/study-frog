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
use studyfrog_ui::msg::MsgBrowse;
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
fn the_toolbar_formats_the_text_and_the_side_column_shows_the_details() {
    let mut h = harness(editing());
    h.run_steps(2);
    h.get_by_label("Bold in Body").click();
    h.run_steps(2);
    let body = h.state().0.state.editor.as_ref().unwrap().form.fields[0]
        .text
        .clone();
    assert!(body.ends_with("**bold**"), "{body:?}");
    assert!(h.state().0.state.editor.as_ref().is_some());
    for text in ["Details", "Kind", "note"] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    h.get_by_label("Relations").click();
    h.run_steps(2);
    assert!(h.query_all_by_label("Links to").count() >= 1);
    h.get_by_label("Body: preview").click();
    h.run_steps(2);
    assert!(
        h.query_by_label("Bold in Body").is_none(),
        "no toolbar in preview mode"
    );
    h.get_by_label("Body: write").click();
    h.run_steps(2);
    assert!(h.query_by_label("Bold in Body").is_some());
}

#[test]
fn the_side_column_lists_the_items_that_link_here() {
    use studyfrog_core::plan::ItemKind;
    use studyfrog_store::overview::ItemRow;
    let mut m = editing();
    let row = ItemRow {
        uid: Uid(9),
        title: "Flashcard about Alpha".into(),
        kind: ItemKind::Card,
        path: "s/c.md".into(),
        due: None,
        status: studyfrog_core::status::Status::New,
    };
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::BacklinksLoaded(Uid(0xA), Ok(vec![row]))),
    );
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Relations").click();
    h.run_steps(2);
    assert!(h.query_by_label("Linked from").is_some());
    let node = h.get_by_label("Flashcard about Alpha");
    node.scroll_to_me();
    h.run();
    h.get_by_label("Flashcard about Alpha").click();
    h.run_steps(2);
    assert!(
        h.state()
            .1
            .contains(&Effect::Editor(EffectEditor::LoadDocument(Uid(9))))
    );
}

#[test]
fn priority_and_difficulty_are_chosen_on_the_page_and_saved_in_the_front_matter() {
    use studyfrog_core::event::Difficulty;
    use studyfrog_core::priority::Priority;
    let mut h = harness(editing());
    h.run_steps(2);
    assert!(
        h.query_by_label("Priority: inherit").is_some()
            && h.query_by_label("Difficulty: inherit").is_some()
    );
    h.get_by_label("Priority: high").click();
    h.run_steps(2);
    h.get_by_label("Difficulty: difficult").click();
    h.run_steps(2);
    let e = h.state().0.state.editor.as_ref().unwrap();
    assert_eq!(
        (e.form.priority, e.form.difficulty),
        (Some(Priority::High), Some(Difficulty::Difficult))
    );
    assert!(e.saving.is_some(), "a choice is saved at once");
    let saved = studyfrog_ui::edit_form::apply_form(&e.doc, &e.form).unwrap();
    assert_eq!(
        studyfrog_md::extra::extra_value(&saved, "priority").as_deref(),
        Some("high")
    );
    h.get_by_label("Priority: inherit").click();
    h.run_steps(2);
    let e = h.state().0.state.editor.as_ref().unwrap();
    let saved = studyfrog_ui::edit_form::apply_form(&e.doc, &e.form).unwrap();
    assert_eq!(studyfrog_md::extra::extra_value(&saved, "priority"), None);
    assert_all_named(&h, "item page with priority");
}
