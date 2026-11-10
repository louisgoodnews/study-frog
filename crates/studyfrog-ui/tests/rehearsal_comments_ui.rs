#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for expandable comments drawer during rehearsal and exam modes (STORY-0140).

mod common;

use common::views::{harness, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::comment::new_comment;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::rehearsal_edit::{RehearsalEditEffect, RehearsalEditMsg};

#[test]
fn rehearsal_comments_drawer_toggle_and_display() {
    let mut m = running_model(RunMode::Flip);
    let uid = Uid(1);

    // Attach an initial comment to item 1
    let c1 = new_comment(
        "c_1",
        "learner",
        1731140000000,
        "Remember: Paris is the capital",
    );
    if let Some(run) = &mut m.state.rehearse.run
        && let Some(content) = run.content.get_mut(&uid)
    {
        content.comments.push(c1);
    }

    let mut h = harness(m);
    h.run_steps(4);

    // Initial button indicates 1 comment
    assert!(h.query_by_label("Comments (1)").is_some());
    // Drawer not shown yet
    assert!(h.query_by_label("Card Comments").is_none());

    // Toggle comments drawer open
    h.get_by_label("Comments (1)").click();
    h.run_steps(2);

    assert!(
        h.state()
            .0
            .state
            .rehearse
            .run
            .as_ref()
            .unwrap()
            .show_comments
    );
    assert!(h.query_by_label("Card Comments").is_some());
    assert!(h.query_by_label("learner").is_some());
    assert!(h.query_by_label("Remember: Paris is the capital").is_some());

    // Close button inside drawer closes it
    assert!(h.query_by_label("Close (C)").is_some());
    h.get_by_label("Close (C)").click();
    h.run_steps(2);

    assert!(
        !h.state()
            .0
            .state
            .rehearse
            .run
            .as_ref()
            .unwrap()
            .show_comments
    );
    assert!(h.query_by_label("Card Comments").is_none());
}

#[test]
fn rehearsal_comment_draft_and_submit_emits_effect() {
    let mut m = running_model(RunMode::Flip);
    let uid = Uid(1);

    // Toggle comments open
    apply(&mut m, Msg::RehearsalEdit(RehearsalEditMsg::ToggleComments));
    assert!(m.state.rehearse.run.as_ref().unwrap().show_comments);

    // Draft a comment
    apply(
        &mut m,
        Msg::RehearsalEdit(RehearsalEditMsg::DraftComment(
            "Useful mnemonic here".into(),
        )),
    );
    assert_eq!(
        m.state.rehearse.run.as_ref().unwrap().comment_draft,
        "Useful mnemonic here"
    );

    // Submit comment
    let effects = apply(
        &mut m,
        Msg::RehearsalEdit(RehearsalEditMsg::SubmitComment(uid)),
    );

    // Draft is cleared
    assert_eq!(m.state.rehearse.run.as_ref().unwrap().comment_draft, "");

    // Comment was pushed to run content immediately
    let comments = &m
        .state
        .rehearse
        .run
        .as_ref()
        .unwrap()
        .content
        .get(&uid)
        .unwrap()
        .comments;
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text, "Useful mnemonic here");

    // Effect to append to doc on disk was emitted
    assert_eq!(effects.len(), 1);
    let Effect::RehearsalEdit(RehearsalEditEffect::AppendComment(item_uid, comment)) = &effects[0]
    else {
        panic!("expected RehearsalEditEffect::AppendComment");
    };
    assert_eq!(*item_uid, uid);
    assert_eq!(comment.text, "Useful mnemonic here");

    // Run clock and queue position unaffected
    assert_eq!(m.state.rehearse.run.as_ref().unwrap().session.cursor, 0);
}
