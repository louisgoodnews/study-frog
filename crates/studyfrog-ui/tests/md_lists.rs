#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Enter and Backspace in lists: the pure edits, the real editor, and the preview's list markers.

mod common;

use common::views::{harness, open_model};
use eframe::egui::{Key, Modifiers};
use egui_kittest::kittest::{NodeT, Queryable};
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::md_lists::{on_backspace, on_enter};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn end(text: &str) -> usize {
    text.chars().count()
}

#[test]
fn enter_continues_numbered_bullet_and_task_items_at_their_indent() {
    for (text, expect) in [
        ("1. one", "1. one\n2. "),
        ("9. nine", "9. nine\n10. "),
        ("3) three", "3) three\n4) "),
        ("- a", "- a\n- "),
        ("* a", "* a\n* "),
        ("+ a", "+ a\n+ "),
        ("- [ ] todo", "- [ ] todo\n- [ ] "),
        ("- [x] done", "- [x] done\n- [ ] "),
        ("1. [ ] first", "1. [ ] first\n2. [ ] "),
        ("    - nested", "    - nested\n    - "),
        ("text\n2. two", "text\n2. two\n3. "),
    ] {
        let (new, (a, b)) = on_enter(text, end(text)).unwrap();
        assert_eq!(new, expect, "{text:?}");
        assert_eq!(
            (a, b),
            (end(expect), end(expect)),
            "cursor after the new marker"
        );
    }
}

#[test]
fn enter_in_the_middle_of_an_item_splits_it_and_moves_the_rest_down() {
    let (new, (a, _)) = on_enter("1. onetwo", 6).unwrap();
    assert_eq!(new, "1. one\n2. two");
    assert_eq!(a, 10);
}

#[test]
fn enter_on_an_empty_item_ends_the_list_and_backspace_removes_the_marker() {
    for text in ["- ", "2. ", "- [ ] ", "  - "] {
        let (new, cursor) = on_enter(text, end(text)).unwrap();
        assert_eq!((new.as_str(), cursor), ("", (0, 0)), "{text:?}");
        let (new, cursor) = on_backspace(text, end(text)).unwrap();
        assert_eq!((new.as_str(), cursor), ("", (0, 0)), "{text:?}");
    }
    let (new, cursor) = on_enter("1. a\n2. ", 8).unwrap();
    assert_eq!((new.as_str(), cursor), ("1. a\n", (5, 5)));
}

#[test]
fn other_lines_and_other_places_keep_the_editors_own_behaviour() {
    for text in [
        "plain",
        "",
        "---",
        "1.5 not a list",
        "-no space",
        "# heading",
        "> quote",
    ] {
        assert_eq!(on_enter(text, end(text)), None, "{text:?}");
        assert_eq!(on_backspace(text, end(text)), None, "{text:?}");
    }
    assert_eq!(on_enter("- item", 1), None, "inside the marker");
    assert_eq!(on_backspace("- item", 6), None, "an item with text");
    assert_eq!(on_backspace("- ", 1), None, "not at the end of the line");
}

const NOTE: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Alpha\n---\n1. one";

fn editing(body_marker: &str) -> Model {
    let mut m = open_model();
    let text = NOTE.replace("1. one", body_marker);
    let doc = parse_doc(&text).unwrap();
    apply(
        &mut m,
        Msg::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "s/alpha.md".into(),
                doc,
                hash: 1,
            }),
        ),
    );
    m
}

fn body(h: &egui_kittest::Harness<'_, common::views::App>) -> String {
    h.state().0.state.editor.as_ref().unwrap().form.fields[0]
        .text
        .clone()
}

#[test]
fn pressing_enter_in_the_real_editor_continues_the_list_and_typing_goes_on_in_it() {
    let mut h = harness(editing("1. one"));
    h.run_steps(3);
    h.get_by_label("Body").focus();
    h.key_press_modifiers(Modifiers::COMMAND, Key::End);
    h.run_steps(2);
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(body(&h), "1. one\n2. ");
    h.get_by_label("Body").type_text("two");
    h.run_steps(2);
    assert!(body(&h).contains("1. one\n2. two"), "{:?}", body(&h));
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert!(body(&h).contains("2. two\n3. "), "{:?}", body(&h));
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert!(
        !body(&h).contains("3."),
        "Enter on the empty item ended the list: {:?}",
        body(&h)
    );
}

#[test]
fn list_markers_in_the_preview_are_level_with_their_text() {
    use studyfrog_service::help::{PageRef, UserPage};
    use studyfrog_ui::help_types::HelpMsg;
    let mut m = open_model();
    m.state.toasts.clear();
    let page = UserPage {
        file: "Lists".into(),
        text: "# Lists\n\n- first bullet\n- second bullet\n\n1. first number\n2. second number\n"
            .into(),
    };
    apply(&mut m, Msg::Help(HelpMsg::ForScreen));
    apply(&mut m, Msg::Help(HelpMsg::Loaded(Ok(vec![page]))));
    apply(
        &mut m,
        Msg::Help(HelpMsg::Open(PageRef::User("Lists".into()))),
    );
    let mut h = harness(m);
    h.run_steps(5);
    let centre = |label: &str| {
        let b = h
            .get_by_label(label)
            .accesskit_node()
            .bounding_box()
            .unwrap();
        (b.y0 + b.y1) / 2.0
    };
    let bullets: Vec<f64> = h
        .query_all_by_label("•")
        .map(|n| {
            let b = n.accesskit_node().bounding_box().unwrap();
            (b.y0 + b.y1) / 2.0
        })
        .collect();
    assert_eq!(bullets.len(), 2);
    for (marker, text) in bullets.iter().zip(["first bullet", "second bullet"]) {
        assert!(
            (marker - centre(text)).abs() < 2.0,
            "bullet {marker} text {}",
            centre(text)
        );
    }
    for (marker, text) in [("1.", "first number"), ("2.", "second number")] {
        assert!(
            (centre(marker) - centre(text)).abs() < 2.0,
            "{marker} {} vs {}",
            centre(marker),
            centre(text)
        );
    }
}
