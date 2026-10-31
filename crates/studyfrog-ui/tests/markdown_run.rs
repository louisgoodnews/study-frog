#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Markdown in the rehearsal card: the text is formatted, not shown with its markers.

mod common;

use common::views::{harness, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_service::content::ItemContent;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

#[test]
fn the_prompt_and_the_answer_are_drawn_as_markdown() {
    let mut m = running_model(RunMode::Flip);
    let content = ItemContent {
        title: "t".into(),
        prompt: "Capital of **France** and *Spain*?".into(),
        answer: "- `Paris`\n- Madrid".into(),
        previous: None,
        source: None,
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content)));
    let mut h = harness(m);
    h.run_steps(2);
    assert!(
        h.query_by_label("France").is_some(),
        "bold text without its stars"
    );
    assert!(
        h.query_by_label_contains("**").is_none() && h.query_by_label_contains("*Spain*").is_none()
    );
    h.get_by_label("Show answer (Space)").click();
    h.run_steps(2);
    assert!(h.query_by_label("Paris").is_some() && h.query_by_label("Madrid").is_some());
    assert!(
        h.query_by_label_contains("`").is_none(),
        "code markers are gone"
    );
}
