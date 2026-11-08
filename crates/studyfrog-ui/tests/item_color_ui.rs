#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0097, STORY-0123 through the real worker and the harness: picking a colour in the Properties
//! tab, saving it to the file, the library row, and the rehearsal card's colour in both phases.

mod common;

use common::flow::{opened, send};
use common::views::{harness, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_ui::edit_form::EditAction;
use studyfrog_ui::item_look::run_color;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

#[test]
fn the_properties_tab_offers_the_palette_none_a_picker_and_a_code() {
    let (_vault, mut m, mut w) = opened("color-tab");
    send(&mut m, &mut w, Msg::OpenItem(Uid(11)));
    let mut h = harness(m);
    h.run_steps(2);
    for name in [
        "Color: none",
        "Color: Sage",
        "Color: Sky blue",
        "Color: Slate",
        "Pick a color",
    ] {
        assert!(h.query_all_by_label(name).count() >= 1, "missing {name}");
    }
    assert!(h.query_all_by_label("Custom color (#RRGGBB)").count() >= 1);
    h.get_by_label("Color: Rose").click();
    h.run_steps(2);
    let form = &h.state().0.state.editor.as_ref().unwrap().form;
    assert_eq!(form.color.as_deref(), Some("rose"));
}

#[test]
fn a_chosen_colour_is_saved_to_the_file_and_shown_in_the_library() {
    let (vault, mut m, mut w) = opened("color-save");
    send(&mut m, &mut w, Msg::OpenItem(Uid(11)));
    send(
        &mut m,
        &mut w,
        Msg::Edit(EditAction::Color(Some("#4a90e2".into()))),
    );
    send(&mut m, &mut w, Msg::SaveClicked);
    let text = std::fs::read_to_string(vault.path.join("french/b.md")).unwrap();
    assert!(text.contains("color: \"#4A90E2\""), "{text}");
    send(&mut m, &mut w, Msg::Edit(EditAction::Color(None)));
    send(&mut m, &mut w, Msg::SaveClicked);
    let text = std::fs::read_to_string(vault.path.join("french/b.md")).unwrap();
    assert!(!text.contains("color:"), "None removes it: {text}");
    send(
        &mut m,
        &mut w,
        Msg::Edit(EditAction::Color(Some("amber".into()))),
    );
    send(&mut m, &mut w, Msg::SaveClicked);
    send(&mut m, &mut w, Msg::Navigate(Screen::Library));
    let b = m
        .state
        .library
        .items
        .iter()
        .find(|i| i.uid == Uid(11))
        .unwrap();
    assert_eq!(b.color.as_deref(), Some("amber"));
    let mut h = harness(m);
    h.run_steps(2);
    assert!(
        h.query_all_by_label("Color amber").count() >= 1,
        "the colour dot in the list"
    );
}

#[test]
fn the_rehearsal_card_takes_the_colour_of_the_item_on_show_in_both_phases() {
    let mut m = running_model(RunMode::Flip);
    let run = m.state.rehearse.run.as_mut().unwrap();
    for c in run.content.values_mut() {
        c.color = Some("teal".into());
    }
    assert_eq!(
        run_color(m.state.rehearse.run.as_ref().unwrap()),
        Some("teal")
    );
    common::runs::act(&mut m, 1.0, Action::Flip);
    assert_eq!(
        run_color(m.state.rehearse.run.as_ref().unwrap()),
        Some("teal"),
        "after the reveal"
    );
    apply(&mut m, Msg::RunAction(Action::Pause));
    assert_eq!(
        run_color(m.state.rehearse.run.as_ref().unwrap()),
        None,
        "paused hides the item"
    );
}
