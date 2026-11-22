#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The appearance commands of the control protocol (STORY-0193): words, class, plan, and the answer with the
//! spacing the density produces.

use serde_json::json;
use studyfrog_ui::appearance_ui::AppearanceMsg;
use studyfrog_ui::control_cmd::{Class, Command, class_of};
use studyfrog_ui::control_parse::parse_words;
use studyfrog_ui::control_plan::plan_for;
use studyfrog_ui::control_reply::reply_line;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::State;

#[test]
fn an_appearance_setting_is_one_write_with_its_value() {
    let cmd = parse_words("appearance density compact").unwrap();
    assert_eq!(
        cmd,
        Command::Appearance(AppearanceMsg::Density(
            studyfrog_core::appearance::Density::Compact
        ))
    );
    assert_eq!(class_of(&cmd), Class::Write);
    let plan = plan_for(&State::default(), &cmd).unwrap();
    assert_eq!(
        plan,
        vec![vec![Msg::Appearance(AppearanceMsg::Density(
            studyfrog_core::appearance::Density::Compact
        ))]]
    );
}

#[test]
fn a_text_size_outside_the_range_and_an_unknown_setting_are_errors_that_name_the_choices() {
    assert!(
        parse_words("appearance text 500")
            .unwrap_err()
            .contains("between 70 and 200")
    );
    assert!(
        parse_words("appearance accent red")
            .unwrap_err()
            .contains("teal")
    );
    assert!(
        parse_words("appearance colour blue")
            .unwrap_err()
            .contains("theme, density, text, accent")
    );
}

#[test]
fn the_answer_gives_the_density_spacing_and_the_settings() {
    let cmd = parse_words("appearance density spacious").unwrap();
    let mut state = State::default();
    state.app.appearance.density = studyfrog_core::appearance::Density::Spacious;
    let line = reply_line(&json!(1), &state, &cmd, &None);
    assert!(line.contains("\"density\":\"spacious\""), "{line}");
    assert!(line.contains("\"spacing\":1.5"), "{line}");
}
