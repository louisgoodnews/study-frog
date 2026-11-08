//! The rehearsal screen: prompt and timer only, keyboard first. Space reveals, 1-4 rate, S skips,
//! P pauses, Esc stops. Shortcuts are off while a text field has the keyboard.

use eframe::egui::{self, Align, Key, Layout, RichText};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::session::ItemPhase;

use crate::msg::Msg;
use crate::run_state::{RunView, engine_ms};
use crate::run_update::{awaiting_rating, awaiting_reveal, is_over};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::run_card::{MARKS, RATINGS, item, paused};

fn act(out: &mut Vec<Msg>, action: Action) {
    out.push(Msg::RunAction(action));
}

/// The live run, or its summary once it is over.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(run) = &state.rehearse.run else {
        return;
    };
    if is_over(run) && crate::exam_ui::is_exam(run) {
        crate::view::exam_result::draw(state, run, ui, out);
        return;
    }
    if is_over(run) {
        crate::view::summary::draw(state, run, ui, out);
        return;
    }
    shortcuts(run, ui, out);
    top_strip(state, run, ui, out);
    ui.add_space(14.0);
    egui::ScrollArea::vertical()
        .id_salt("rehearsal-card-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_max_width(CARD_WIDTH);
                let p = palette_of(ui);
                let color = crate::item_look::run_color(run);
                let colors = crate::item_look::card_colors(&p, ui.visuals().dark_mode, color);
                kit::card_with(ui, colors, |ui| {
                    ui.set_width(ui.available_width());
                    if run.session.run.state == RunState::Paused {
                        paused(ui, out);
                    } else {
                        item(run, ui, out);
                    }
                });
                ui.add_space(10.0);
                crate::view::run_slow::notice(state, run, ui, out);
                let exam = crate::exam_ui::is_exam(run);
                ui.label(
                    RichText::new(if exam {
                        crate::view::exam_card::EXAM_HINTS
                    } else {
                        HINTS
                    })
                    .color(palette_of(ui).text_weak)
                    .size(12.5),
                );
            });
        });
}

const CARD_WIDTH: f32 = 760.0;
const HINTS: &str =
    "Space shows the answer  ·  1 2 3 4 rate  ·  E M D mark  ·  S skip  ·  P pause  ·  Esc stop";

fn shortcuts(run: &RunView, ui: &egui::Ui, out: &mut Vec<Msg>) {
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    let pressed = |k: Key| ui.input(|i| i.key_pressed(k));
    if crate::exam_ui::is_exam(run) {
        return exam_shortcuts(run, &pressed, out);
    }
    if awaiting_reveal(run) && pressed(Key::Space) {
        act(out, Action::Flip);
    }
    if awaiting_reveal(run) && pressed(Key::S) {
        act(out, Action::Skip);
    }
    for (difficulty, _, key) in MARKS {
        if (awaiting_reveal(run) || awaiting_rating(run)) && pressed(key) {
            act(out, Action::Mark(difficulty));
        }
    }
    for (rating, _, key) in RATINGS {
        if awaiting_rating(run) && pressed(key) {
            act(out, Action::Rate(rating));
        }
    }
    if pressed(Key::P) {
        let paused = run.session.run.state == RunState::Paused;
        act(
            out,
            if paused {
                Action::Resume
            } else {
                Action::Pause
            },
        );
    }
    if pressed(Key::Escape) {
        act(out, Action::Abort);
    }
}

/// An exam answers to P (pause and resume) only: no reveal, skip, marks or ratings, and Esc does not stop it.
fn exam_shortcuts(run: &RunView, pressed: &dyn Fn(Key) -> bool, out: &mut Vec<Msg>) {
    if pressed(Key::P) {
        let paused = run.session.run.state == RunState::Paused;
        act(
            out,
            if paused {
                Action::Resume
            } else {
                Action::Pause
            },
        );
    }
}

/// Position, mode, timer and the pause and stop buttons, with the progress bar under them.
fn top_strip(state: &State, run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let s = &run.session;
    let weak = palette_of(ui).text_weak;
    let position = (s.cursor + 1).min(s.queue.len());
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{position} / {}", s.queue.len()))
                .size(18.0)
                .strong(),
        );
        let mode = match s.run.mode {
            RunMode::Answer => "Type answers",
            RunMode::Exam => "Exam",
            RunMode::Flip => "Flip cards",
        };
        kit::pill(ui, mode, weak);
        if s.run.state == RunState::Running && s.phase != ItemPhase::NoItem {
            let now = engine_ms(state.now, run.started_time, s.last_t_ms);
            let on_item = now
                .saturating_sub(s.shown_at_ms)
                .saturating_sub(s.paused_in_item_ms);
            kit::pill(ui, &format!("{:.1} s", f64::from(on_item) / 1000.0), weak);
        }
        if let Some(clock) = crate::run_clock::clock_of(state, run) {
            crate::view::run_timer::draw(&clock, ui);
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let stop = if s.run.mode == RunMode::Exam {
                "Stop exam (not graded)"
            } else {
                "Stop (Esc)"
            };
            if ui.button(stop).clicked() {
                act(out, Action::Abort);
            }
            if s.run.state == RunState::Running && ui.button("Pause (P)").clicked() {
                act(out, Action::Pause);
            }
        });
    });
    let p = crate::run_progress::progress(s);
    kit::progress_with_text(
        ui,
        crate::run_progress::fraction(p),
        &crate::run_progress::text(p),
        "Run progress",
    );
}
