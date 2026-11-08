#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Cloze cards in a run and the cloze button of the editor (STORY-0108).

mod common;

use common::runs::act;
use common::views::{assert_all_named, harness, open_model, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::content::ItemContent;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{EditAction, Msg};

fn with_gap(mode: RunMode) -> Model {
    let mut m = running_model(mode);
    let content = ItemContent {
        title: "Sun".into(),
        prompt: "The `[a star]` is hot.".into(),
        answer: "The **<span style=\"color:#1e8449\">sun</span>** is hot.".into(),
        previous: None,
        source: None,
        color: None,
        expected: Some("sun".into()),
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content)));
    m
}

#[test]
fn the_gap_shows_its_hint_and_the_answer_highlights_the_words() {
    let mut h = harness(with_gap(RunMode::Flip));
    h.run_steps(3);
    assert!(
        h.query_by_label("[a star]").is_some(),
        "the gap with its hint"
    );
    assert!(h.query_by_label("sun").is_none(), "hidden before the flip");
    assert_all_named(&h, "cloze prompt");
    let mut m = std::mem::take(&mut h.state_mut().0);
    act(&mut m, 1.0, Action::Flip);
    let mut h = harness(m);
    h.run_steps(3);
    assert!(
        h.query_by_label("sun").is_some(),
        "the gap's words after the flip"
    );
}

#[test]
fn a_typed_answer_is_compared_with_the_gap() {
    let mut m = with_gap(RunMode::Answer);
    let fx = act(&mut m, 1.0, Action::SubmitAnswer("sun".into()));
    assert!(fx.iter().any(|e| matches!(
        e,
        Effect::ScoreAnswer { item: Uid(1), answer, .. }
            if *answer == ("sun".to_owned(), Some("sun".to_owned()))
    )));
}

#[test]
fn the_cloze_button_wraps_the_text_in_a_gap() {
    let card = "---\nid: 0000000000000000000000000A\nkind: card\ntitle: C\n---\nThe sun\n???\nx\n";
    let mut m = open_model();
    let loaded = LoadedDoc {
        path: "s/c.md".into(),
        doc: parse_doc(card).unwrap(),
        hash: 1,
    };
    apply(&mut m, Msg::DocumentLoaded(Uid(0xA), Ok(loaded)));
    let mut h = harness(m);
    h.run_steps(2);
    let tip = "Cloze: hide the selection as a gap (Ctrl+Shift+C; with Alt the same number)";
    h.get_by_label(&format!("{tip} in Front")).click();
    h.run_steps(3);
    let e = h.state().0.state.editor.as_ref().unwrap();
    assert_eq!(e.form.fields[0].text, "The sun{{c1::answer}}");
    let mut m = std::mem::take(&mut h.state_mut().0);
    apply(
        &mut m,
        Msg::Edit(EditAction::Field(0, "{{c1::The}} sun".into())),
    );
    let mut h = harness(m);
    h.run_steps(3);
    assert!(
        h.query_by_label("The").is_some(),
        "the preview shows the gap's words"
    );
}

#[test]
fn ctrl_shift_c_makes_a_gap_and_with_alt_keeps_the_number() {
    use eframe::egui::{Key, Modifiers};
    let card = "---\nid: 0000000000000000000000000A\nkind: card\ntitle: C\n---\nab\n???\nx\n";
    let mut m = open_model();
    let loaded = LoadedDoc {
        path: "s/c.md".into(),
        doc: parse_doc(card).unwrap(),
        hash: 1,
    };
    apply(&mut m, Msg::DocumentLoaded(Uid(0xA), Ok(loaded)));
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Front").focus();
    h.run_steps(1);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::C);
    h.run_steps(3);
    let text = |h: &egui_kittest::Harness<'_, common::views::App>| {
        h.state().0.state.editor.as_ref().unwrap().form.fields[0]
            .text
            .clone()
    };
    assert!(text(&h).contains("{{c1::answer}}"), "{}", text(&h));
    let all = Modifiers::COMMAND | Modifiers::SHIFT | Modifiers::ALT;
    h.key_press_modifiers(all, Key::C);
    h.run_steps(3);
    assert_eq!(text(&h).matches("{{c1::").count(), 2, "{}", text(&h));
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::C);
    h.run_steps(3);
    assert!(text(&h).contains("{{c2::"), "{}", text(&h));
}
