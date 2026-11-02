//! The settings screen: tabs by domain, with the effective settings (read-only), the reading-speed test and more.

use eframe::egui::{self, Grid, Key};
use studyfrog_core::scheduler::scheduler_name;
use studyfrog_core::settings::AnswerEngine;

use crate::app_settings_ui::{SHOWN, SettingsTab, SettingsTabMsg, current_tab, tab_label};
use crate::msg::Msg;
use crate::reading_test::{ReadingTestMsg, TEXT, TestPhase, current_wpm};
use crate::state::State;

/// A part of a tab: draws one group of settings.
pub type Section = fn(&State, &mut egui::Ui, &mut Vec<Msg>);

/// The groups of settings on a tab. Adding a setting means adding its section here, to one tab only.
pub fn sections_of(tab: SettingsTab) -> &'static [Section] {
    match tab {
        SettingsTab::Study => &[scheduling, view_tracking],
        SettingsTab::Rehearsal => &[reading_speed],
        SettingsTab::Answers => &[answer_comparison],
        SettingsTab::Data => &[crate::view::backup::draw],
        SettingsTab::Subjects => &[crate::view::subjects::draw],
        SettingsTab::General | SettingsTab::Appearance | SettingsTab::Notifications => &[],
    }
}

/// The settings screen: a tab bar and the groups of the chosen tab.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("Settings");
    let tab = current_tab(&state.app);
    tab_bar(ui, tab, out);
    ui.separator();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for section in sections_of(tab) {
                section(state, ui, out);
                ui.add_space(8.0);
            }
        });
}

fn tab_bar(ui: &mut egui::Ui, current: SettingsTab, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        for tab in SHOWN {
            if ui
                .selectable_label(tab == current, tab_label(tab))
                .clicked()
            {
                out.push(Msg::SettingsTab(SettingsTabMsg::Select(tab)));
            }
        }
    });
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    for (key, by) in [(Key::ArrowLeft, -1), (Key::ArrowRight, 1)] {
        if ui.input(|i| i.key_pressed(key)) {
            out.push(Msg::SettingsTab(SettingsTabMsg::Step(by)));
        }
    }
}

/// Whether the app records how long items are viewed.
fn view_tracking(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("Familiarity");
    let mut on = !state.app.views_untracked;
    if ui
        .checkbox(&mut on, "Record how long I look at items")
        .changed()
    {
        out.push(Msg::SetTrackViews(on));
    }
    ui.weak("Used to tell how familiar you are with an item. Only the time and the item are stored, on this computer, and they are part of backups.");
}

/// The scheduler settings the vault uses (read-only) and where to change them.
fn scheduling(state: &State, ui: &mut egui::Ui, _out: &mut Vec<Msg>) {
    ui.heading("Scheduling");
    let Some(vault) = &state.vault else { return };
    if let Some(e) = &vault.settings_error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("studyfrog.toml is not used: {e}"),
        );
    }
    let s = &vault.settings;
    Grid::new("settings").num_columns(2).show(ui, |ui| {
        ui.label("Default scheduler");
        ui.label(scheduler_name(s.scheduler));
        ui.end_row();
        ui.label("Target retention (FSRS)");
        ui.label(format!("{}", s.request_retention));
        ui.end_row();
        ui.label("Maximum interval (days)");
        ui.label(s.maximum_interval_days.to_string());
        ui.end_row();
        ui.label("FSRS weights");
        ui.label(if s.weights.is_some() {
            "custom"
        } else {
            "published defaults"
        });
        ui.end_row();
    });
    ui.add_space(8.0);
    ui.weak("Edit studyfrog.toml in the vault folder, and _stack.toml next to a stack, to change these.");
}

fn test_msg(out: &mut Vec<Msg>, m: ReadingTestMsg) {
    out.push(Msg::ReadingTest(m));
}

/// Where the reading-speed baseline comes from, and the test that measures it.
fn reading_speed(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("Reading speed");
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
            if ui.button("I have finished reading").clicked() {
                test_msg(out, ReadingTestMsg::Finish);
            }
        }
        TestPhase::Result(Some(wpm)) => {
            ui.label(format!("You read {wpm} words per minute."));
            ui.horizontal(|ui| {
                if ui.button("Use this speed").clicked() {
                    test_msg(out, ReadingTestMsg::Save);
                }
                if ui.button("Discard").clicked() {
                    test_msg(out, ReadingTestMsg::Discard);
                }
            });
        }
        TestPhase::Result(None) => {
            ui.label("That was too fast or too slow to be a reading test. Please try again.");
            if ui.button("Try again").clicked() {
                test_msg(out, ReadingTestMsg::Start);
            }
        }
    }
}

fn idle(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        if ui.button("Measure my reading speed").clicked() {
            test_msg(out, ReadingTestMsg::Start);
        }
        if state.app.reading_wpm.is_some() && ui.button("Use the default speed").clicked() {
            test_msg(out, ReadingTestMsg::Reset);
        }
    });
}

/// How typed answers are compared, and the thresholds that turn reading time into suggestions.
fn answer_comparison(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading("Answer comparison");
    ui.horizontal(|ui| {
        for (engine, label) in [
            (AnswerEngine::Auto, "Language model when installed"),
            (AnswerEngine::Words, "Word matching only"),
        ] {
            if ui
                .selectable_label(state.app.answer_engine == engine, label)
                .clicked()
            {
                out.push(Msg::SetAnswerEngine(engine));
            }
        }
    });
    ui.weak("A high match means similar wording or meaning, not that the facts are right.");
    let p = studyfrog_rehearsal::reading::reading_params(state.app.reading_wpm, &state.app.reading);
    ui.label(format!(
        "Reading suggestions: fast is under {} % of the expected time, slow is over {} %, answers under {} words are ignored.",
        p.fast_percent, p.slow_percent, p.min_words
    ));
    ui.weak("Change reading_fast_percent, reading_slow_percent and reading_min_words in app.toml.");
}
