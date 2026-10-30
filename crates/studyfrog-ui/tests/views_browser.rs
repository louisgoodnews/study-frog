#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The browser's real widgets, headless.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{ItemRow, Selection};

fn item(n: u128, title: &str) -> ItemRow {
    ItemRow {
        uid: Uid(n),
        title: title.into(),
        kind: ItemKind::Card,
        path: format!("french/{n}.md"),
        due: None,
    }
}

fn stack_open() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    apply(
        &mut m,
        Msg::ItemsLoaded(Ok(vec![item(1, "Bonjour"), item(2, "Au revoir")])),
    );
    m
}

#[test]
fn clicking_a_stack_selects_it_and_asks_for_its_items() {
    let mut h = harness(open_model());
    h.run();
    h.get_all_by_label("Maths").next().unwrap().click();
    h.run();
    assert!(
        h.state()
            .1
            .contains(&Effect::LoadItems(Selection::Stack("maths".into())))
    );
    assert_eq!(
        h.state().0.state.browser.selection,
        Selection::Stack("maths".into())
    );
}

#[test]
fn the_title_filter_narrows_the_list_as_you_type() {
    let mut h = harness(stack_open());
    h.run();
    assert!(h.query_by_label("2 of 2 items").is_some() && h.query_by_label("Bonjour").is_some());
    h.get_by_label("Search titles").focus();
    h.get_by_label("Search titles").type_text("revoir");
    h.run();
    assert!(h.query_by_label("1 of 2 items").is_some());
    assert!(h.query_by_label("Bonjour").is_none() && h.query_by_label("Au revoir").is_some());
}

#[test]
fn the_new_stack_and_new_card_forms_send_their_requests() {
    let mut h = harness(stack_open());
    h.run();
    h.get_by_label("New stack title").focus();
    h.get_by_label("New stack title").type_text("Maths");
    h.run();
    h.get_by_label("Create stack").click();
    h.run();
    assert!(h.state().1.contains(&Effect::CreateStack("Maths".into())));

    h.get_by_label("Card front").focus();
    h.get_by_label("Card front").type_text("2+2?");
    h.get_by_label("Card back").focus();
    h.get_by_label("Card back").type_text("4");
    h.run();
    h.get_by_label("Add card").click();
    h.run();
    let wanted = Effect::CreateCard {
        folder: "french".into(),
        title: String::new(),
        front: "2+2?".into(),
        back: "4".into(),
    };
    assert!(h.state().1.contains(&wanted), "{:?}", h.state().1);
}

#[test]
fn the_card_form_only_appears_inside_a_stack_not_in_the_inbox() {
    let mut m = open_model();
    apply(&mut m, Msg::Select(Selection::Inbox));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("Add card").is_none());
    assert!(
        h.query_by_label("New stack title").is_some(),
        "the stack list stays available"
    );
}

#[test]
fn the_item_list_says_when_each_item_is_next_due() {
    use studyfrog_core::time::UnixMs;
    let mut m = stack_open();
    m.state.browser.items[0].due = Some(UnixMs(1_800_000_000_000)); // 2027-01-15 UTC
    let mut h = harness(m);
    h.run_steps(2);
    assert!(
        h.query_by_label("due 2027-01-15").is_some(),
        "a reviewed item shows its date"
    );
    assert_eq!(
        h.query_all_by_label("new").count(),
        1,
        "the other was never reviewed"
    );
}
