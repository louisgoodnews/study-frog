//! The settings screen: tabs by domain, with the effective settings (read-only), the reading-speed test and more.
use crate::msg::MsgVault;

use eframe::egui::{self, Grid, Key};
use studyfrog_core::scheduler::scheduler_name;
use studyfrog_core::settings::AnswerEngine;

use crate::app_settings_ui::{SHOWN, SettingsTab, SettingsTabMsg, current_tab, tab_label};
use crate::msg::Msg;
use crate::state::State;
use crate::view::{home_hero, kit, screen_title};

/// A part of a tab: draws one group of settings.
pub type Section = fn(&State, &mut egui::Ui, &mut Vec<Msg>);

/// The groups of settings on a tab. Adding a setting means adding its section here, to one tab only.
pub fn sections_of(tab: SettingsTab) -> &'static [Section] {
    match tab {
        SettingsTab::Study => &[scheduling, view_tracking, replay],
        SettingsTab::Rehearsal => &[
            super::settings_reading::reading_speed,
            crate::view::run_slow::setting,
            super::settings_rehearsal_fsrs::draw,
        ],
        SettingsTab::Answers => &[answer_comparison],
        SettingsTab::Data => &[
            crate::view::backup::draw,
            transfer,
            super::settings_files::draw,
            super::wizard::reopen,
        ],
        SettingsTab::Subjects => &[crate::view::subjects::draw],
        SettingsTab::Diary => &[crate::view::diary::settings::draw],
        SettingsTab::Appearance => &[
            super::settings_appearance::theme_section,
            super::settings_appearance::size_section,
            super::settings_appearance::colours_section,
            super::settings_appearance::startup_section,
            super::settings_appearance::sample_section,
            super::settings_appearance::defaults_section,
        ],
        SettingsTab::General | SettingsTab::Notifications => &[],
    }
}

/// The tab last logged as shown, so the debug line is written when the tab changes only.
static LAST_TAB: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

fn log_tab_change(name: &str) {
    let mut last = LAST_TAB.lock().unwrap_or_else(|e| e.into_inner());
    if *last != name {
        studyfrog_log::log_debug!("ui", "settings tab shown"; tab = name.to_owned());
        *last = name.to_owned();
    }
}

/// The settings screen: a tab bar and the groups of the chosen tab.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let _timer = studyfrog_debug::timed("ui", "settings draw");
    let tab = current_tab(&state.app);
    log_tab_change(&format!("{tab:?}"));
    screen_title::headline(ui, "Settings");
    ui.add_space(6.0);
    tab_bar(ui, tab, out);
    ui.separator();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            kit::column(ui, 860.0, |ui| {
                for section in sections_of(tab) {
                    // Each group's caption brings the gap between groups (`kit::caption`).
                    section(state, ui, out);
                }
            });
        });
}

/// A heading for a group of settings, then a redesign card that holds its controls.
fn card_section(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    kit::section(ui, title);
    home_hero::card(ui, add);
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
        // Alt+Left and Alt+Right are Back and Forward in the history, not tab keys.
        if ui.input(|i| i.key_pressed(key) && !i.modifiers.alt) {
            out.push(Msg::SettingsTab(SettingsTabMsg::Step(by)));
        }
    }
}

/// Whether the app records how long items are viewed.
fn view_tracking(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    card_section(ui, "Familiarity", |ui| {
        let mut on = !state.app.views_untracked;
        if ui
            .checkbox(&mut on, "Record how long I look at items")
            .changed()
        {
            out.push(Msg::Vault(MsgVault::SetTrackViews(on)));
        }
        ui.weak("Used to tell how familiar you are with an item. Only the time and the item are stored, on this computer, and they are part of backups.");
    });
}

/// Import and export of the library or a stack.
fn transfer(_: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    card_section(ui, "Import and export", |ui| {
        ui.weak("Write the library or a stack as JSON, CSV or Markdown, or read a file, an Anki package or an Obsidian vault, with a preview first.");
        ui.horizontal(|ui| {
            for (label, tab) in [
                ("Export…", crate::transfer_ui::Tab::Export),
                ("Import…", crate::transfer_ui::Tab::Import),
            ] {
                if ui
                    .button(label)
                    .on_hover_text("Opens the transfer dialog on this tab.")
                    .clicked()
                {
                    let open = crate::transfer_ui::TransferMsg::Open(tab, None);
                    out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(open)));
                }
            }
        });
    });
}

/// Whether familiarity improves the predictions on the user's history.
fn replay(state: &State, ui: &mut egui::Ui, _out: &mut Vec<Msg>) {
    super::settings_replay::draw(state, ui);
}

/// The scheduler settings the vault uses (read-only) and where to change them.
fn scheduling(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    card_section(ui, "Scheduling", |ui| scheduling_body(state, ui, out));
}

fn scheduling_body(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(vault) = &state.vault else { return };
    if let Some(e) = &vault.settings_error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("studyfrog.toml is not used: {e}"),
        );
    }
    let s = &vault.settings;
    Grid::new("settings")
        .num_columns(2)
        .spacing([32.0, 10.0])
        .show(ui, |ui| {
            ui.label("Default scheduler");
            ui.label(scheduler_name(s.scheduler));
            ui.end_row();
            ui.label("Target retention (FSRS)");
            ui.label(format!("{}", s.request_retention));
            ui.end_row();
            ui.label("Maximum interval (days)");
            ui.label(s.maximum_interval_days.to_string());
            ui.end_row();
            ui.label("Familiarity strength");
            ui.label(if s.familiarity_strength > 0.0 {
                format!("{} (0 switches it off)", s.familiarity_strength)
            } else {
                "off".to_owned()
            });
            ui.end_row();
            ui.label("Familiarity threshold");
            ui.label(if s.familiarity_threshold < 1.0 {
                format!(
                    "{} % (due items above it are left out)",
                    (s.familiarity_threshold * 100.0).round()
                )
            } else {
                "off".to_owned()
            });
            ui.end_row();
            ui.label("FSRS weights");
            ui.label(if s.weights.is_some() {
                "custom"
            } else {
                "published defaults"
            });
            ui.end_row();
        });
    let mut calibrated = s.calibrated_start;
    if ui
        .checkbox(
            &mut calibrated,
            "Calibrate a new card's starting difficulty from my earlier answers",
        )
        .on_hover_text("On: the first rating of a new card starts from your own answer times, once you have timed answers. Off: the scheduler's standard starting difficulty.")
        .changed()
    {
        out.push(Msg::Srs(crate::app_settings_ui::SrsMsg::CalibratedStart(calibrated)));
    }
    ui.add_space(8.0);
    ui.weak("Edit studyfrog.toml in the vault folder, and _stack.toml next to a stack, to change these (familiarity_strength, familiarity_threshold, and familiarity = false to switch it off for one stack).");
}

/// How typed answers are compared, and the thresholds that turn reading time into suggestions.
fn answer_comparison(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    card_section(ui, "Answer comparison", |ui| {
        ui.horizontal(|ui| {
            for (engine, label) in [
                (AnswerEngine::Auto, "Language model when installed"),
                (AnswerEngine::Words, "Word matching only"),
            ] {
                if ui
                    .selectable_label(state.app.answer_engine == engine, label)
                    .clicked()
                {
                    out.push(Msg::Vault(MsgVault::SetAnswerEngine(engine)));
                }
            }
        });
        ui.weak("A high match means similar wording or meaning, not that the facts are right.");
        let p = studyfrog_rehearsal::reading::reading_params(
            state.app.reading_wpm,
            &state.app.reading,
        );
        ui.label(format!(
            "Reading suggestions: fast is under {} % of the expected time, slow is over {} %, answers under {} words are ignored.",
            p.fast_percent, p.slow_percent, p.min_words
        ));
        ui.weak("Change reading_fast_percent, reading_slow_percent and reading_min_words in app.toml.");
    });
}
