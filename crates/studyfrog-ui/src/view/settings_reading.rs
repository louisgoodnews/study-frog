//! Settings section for reading speed baseline and testing.

use eframe::egui;

use crate::msg::Msg;
use crate::reading_test::{ReadingTestMsg, TEXT, TestPhase, current_wpm};
use crate::state::State;

fn test_msg(out: &mut Vec<Msg>, m: ReadingTestMsg) {
    out.push(Msg::ReadingTest(m));
}

/// Where the reading-speed baseline comes from, and the test that measures it.
pub fn reading_speed(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::kit::section(ui, "Reading speed");
    let source = if state.app.reading_wpm.is_some() {
        "measured"
    } else {
        "default, not measured"
    };
    ui.label(format!(
        "{} words per minute ({source})",
        current_wpm(state)
    ));
    ui.weak("Rehearsal compares how long you read an answer with this speed to suggest easier or harder ratings.");
    match state.reading_test.phase {
        TestPhase::Idle => idle(state, ui, out),
        TestPhase::Reading(_) => {
            ui.group(|ui| ui.label(TEXT));
            ui.label("Read at your normal pace, then press the button.");
            if ui
                .button("I have finished reading")
                .on_hover_text("Marks this text as read.")
                .clicked()
            {
                test_msg(out, ReadingTestMsg::Finish);
            }
        }
        TestPhase::Result(Some(wpm)) => {
            ui.label(format!("You read {wpm} words per minute."));
            ui.horizontal(|ui| {
                if ui
                    .button("Use this speed")
                    .on_hover_text("Keeps the speed you measured.")
                    .clicked()
                {
                    test_msg(out, ReadingTestMsg::Save);
                }
                if ui
                    .button("Discard")
                    .on_hover_text("Throws away the changes and closes.")
                    .clicked()
                {
                    test_msg(out, ReadingTestMsg::Discard);
                }
            });
        }
        TestPhase::Result(None) => {
            ui.label("That was too fast or too slow to be a reading test. Please try again.");
            if ui
                .button("Try again")
                .on_hover_text("Runs the check again.")
                .clicked()
            {
                test_msg(out, ReadingTestMsg::Start);
            }
        }
    }
}

fn idle(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        if ui
            .button("Measure my reading speed")
            .on_hover_text("Measures how fast you read, to set the timing.")
            .clicked()
        {
            test_msg(out, ReadingTestMsg::Start);
        }
        if state.app.reading_wpm.is_some()
            && ui
                .button("Use the default speed")
                .on_hover_text("Uses the usual reading speed instead.")
                .clicked()
        {
            test_msg(out, ReadingTestMsg::Reset);
        }
    });
}
