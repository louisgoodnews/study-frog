#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0170: the stack page and the item editor show at most three buttons in their header rows, the
//! overflow menu ("More actions") counted among them. The rest of the actions are one click further.
use studyfrog_ui::msg::MsgEditor;

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Selection;

#[test]
fn the_stack_page_header_holds_three_buttons_with_more_actions() {
    let mut m = open_model();
    m.state.browser.selection = Selection::Stack("french".to_owned());
    let mut h = harness(m);
    h.run_steps(3);
    for label in ["New item", "Start rehearsal", "More actions"] {
        assert!(
            h.query_by_label(label).is_some(),
            "{label} is in the header"
        );
    }
    assert!(
        h.query_by_label("Memory game").is_none(),
        "Memory game is in More actions, not the header"
    );
}

#[test]
fn the_item_editor_header_holds_three_buttons_with_more_actions() {
    let mut h = harness(editing_note());
    h.run_steps(3);
    for label in ["Back", "More actions"] {
        assert!(
            h.query_by_label(label).is_some(),
            "{label} is in the header"
        );
    }
}

/// A note open in the editor.
fn editing_note() -> Model {
    use studyfrog_core::uid::Uid;
    use studyfrog_md::parse::parse_doc;
    use studyfrog_service::document::LoadedDoc;
    let mut m = open_model();
    let doc = parse_doc(
        "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Zellmembran\n---\nText.\n",
    )
    .unwrap();
    apply(
        &mut m,
        Msg::Editor(MsgEditor::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "lernen/zellmembran.md".into(),
                doc,
                hash: 1,
            }),
        )),
    );
    m
}
