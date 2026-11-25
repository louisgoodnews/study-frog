//! The rehearsal screen: prompt and timer only, keyboard first. Space reveals, 1-4 rate, S skips,
//! P pauses, Esc stops. Shortcuts are off while a text field has the keyboard.
use crate::msg::MsgRehearsal;

use eframe::egui::{self, Key, RichText};
use studyfrog_core::run::RunState;
use studyfrog_rehearsal::input::Action;

use super::run_strip::top_strip;
use crate::msg::Msg;
use crate::run_state::RunView;
use crate::run_update::{awaiting_rating, awaiting_reveal, is_over};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::run_card::{MARKS, RATINGS, item_scroll, paused};

pub(super) fn act(out: &mut Vec<Msg>, action: Action) {
    out.push(Msg::Rehearsal(MsgRehearsal::RunAction(action)));
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
                // The card takes the whole width, so a long item has room (STORY-0195); its prose runs at most
                // about 1.4 reading measures (see `preview::RUN_PROSE_SHARE`).
                ui.set_max_width(ui.available_width());
                let p = palette_of(ui);
                let color = crate::item_look::run_color(run);
                let colors = crate::item_look::card_colors(&p, ui.visuals().dark_mode, color);
                kit::card_with(ui, colors, |ui| {
                    ui.set_width(ui.available_width());
                    crate::view::gradient_panel::band(ui);
                    if run.session.run.state == RunState::Paused {
                        paused(ui, out);
                    } else {
                        item_scroll(run, ui, out);
                    }
                });
                if run.show_comments {
                    ui.add_space(8.0);
                    super::run_comments::draw(state, run, ui, out);
                }
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
                    .size(crate::typo::size(ui, crate::typo::Tier::Caption)),
                );
            });
        });
}

const HINTS: &str = "Space shows the answer  ·  1 2 3 4 rate  ·  E M D mark  ·  C comments  ·  Ctrl+E edit  ·  S skip  ·  P pause  ·  Esc stop";

fn shortcuts(run: &RunView, ui: &egui::Ui, out: &mut Vec<Msg>) {
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    let pressed = |k: Key| ui.input(|i| i.key_pressed(k));
    if pressed(Key::C) {
        out.push(Msg::RehearsalEdit(
            crate::rehearsal_edit::RehearsalEditMsg::ToggleComments,
        ));
    }
    if crate::exam_ui::is_exam(run) {
        return exam_shortcuts(run, &pressed, out);
    }
    crate::view::run_card::choice::keys(run, ui, out);
    let edit_pressed = ui.input(|i| {
        (i.key_pressed(Key::E) && (i.modifiers.command || i.modifiers.ctrl))
            || i.key_pressed(Key::F2)
    });
    if edit_pressed && let Some(uid) = studyfrog_rehearsal::session::current_item(&run.session) {
        out.push(Msg::RehearsalEdit(
            crate::rehearsal_edit::RehearsalEditMsg::Open(uid),
        ));
    }
    let choosing = crate::view::run_card::choice::pending(run);
    if awaiting_reveal(run) && pressed(Key::Space) && !choosing {
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

/// An exam answers to P (pause and resume) and C (comments drawer) only: no reveal, skip, marks or ratings, and Esc does not stop it.
fn exam_shortcuts(run: &RunView, pressed: &dyn Fn(Key) -> bool, out: &mut Vec<Msg>) {
    if pressed(Key::C) {
        out.push(Msg::RehearsalEdit(
            crate::rehearsal_edit::RehearsalEditMsg::ToggleComments,
        ));
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
}
