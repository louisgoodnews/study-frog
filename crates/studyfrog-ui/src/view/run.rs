//! The rehearsal screen: prompt and timer only, keyboard first. Space reveals, 1-4 rate, S skips,
//! P pauses, Esc stops. Shortcuts are off while a text field has the keyboard.

use eframe::egui::{self, Button, Key, RichText};
use studyfrog_core::review::Rating;
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::session::{ItemPhase, current_item};

use crate::msg::Msg;
use crate::run_state::{RunView, engine_ms};
use crate::run_update::{awaiting_rating, awaiting_reveal, is_over};
use crate::state::State;
use crate::summary::summarize;
use crate::view::widgets::text_field;

const RATINGS: [(Rating, &str, Key); 4] = [
    (Rating::Again, "Again (1)", Key::Num1),
    (Rating::Hard, "Hard (2)", Key::Num2),
    (Rating::Good, "Good (3)", Key::Num3),
    (Rating::Easy, "Easy (4)", Key::Num4),
];

fn act(out: &mut Vec<Msg>, action: Action) {
    out.push(Msg::RunAction(action));
}

/// The live run, or its summary once it is over.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(run) = &state.rehearse.run else {
        return;
    };
    if is_over(run) {
        summary(run, ui, out);
        return;
    }
    shortcuts(run, ui, out);
    header(state, run, ui);
    if run.session.run.state == RunState::Paused {
        ui.add_space(24.0);
        ui.heading("Paused");
        ui.label("The timer is stopped and the question is hidden.");
        if ui.button("Resume (P)").clicked() {
            act(out, Action::Resume);
        }
    } else {
        item(run, ui, out);
    }
    ui.add_space(16.0);
    ui.separator();
    footer(run, ui, out);
}

fn shortcuts(run: &RunView, ui: &egui::Ui, out: &mut Vec<Msg>) {
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    let pressed = |k: Key| ui.input(|i| i.key_pressed(k));
    if awaiting_reveal(run) && pressed(Key::Space) {
        act(out, Action::Flip);
    }
    if awaiting_reveal(run) && pressed(Key::S) {
        act(out, Action::Skip);
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

fn header(state: &State, run: &RunView, ui: &mut egui::Ui) {
    let s = &run.session;
    ui.horizontal(|ui| {
        ui.label(format!(
            "{} / {}",
            (s.cursor + 1).min(s.queue.len()),
            s.queue.len()
        ));
        if s.run.state == RunState::Running && s.phase != ItemPhase::NoItem {
            let now = engine_ms(state.now, run.started_time, s.last_t_ms);
            let on_item = now
                .saturating_sub(s.shown_at_ms)
                .saturating_sub(s.paused_in_item_ms);
            ui.label(format!("{:.1} s", f64::from(on_item) / 1000.0));
        }
    });
}

fn item(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(content) = current_item(&run.session).and_then(|uid| run.content.get(&uid)) else {
        ui.add_space(24.0);
        ui.label("Loading…");
        return;
    };
    ui.add_space(24.0);
    ui.label(RichText::new(&content.prompt).size(24.0));
    if awaiting_reveal(run) {
        ui.add_space(16.0);
        reveal_controls(run, ui, out);
    } else if awaiting_rating(run) {
        ui.add_space(16.0);
        ui.separator();
        ui.label(RichText::new(&content.answer).size(20.0));
        if let Some(scored) = &run.scored {
            score_line(scored, ui);
        }
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            for (rating, label, _) in RATINGS {
                if ui.button(label).clicked() {
                    act(out, Action::Rate(rating));
                }
            }
        });
    }
}

fn score_line(scored: &crate::run_state::Scored, ui: &mut egui::Ui) {
    use studyfrog_slm::score::Verdict;
    let verdict = match scored.score.verdict {
        Verdict::Correct => "Correct",
        Verdict::Partial => "Partly right",
        Verdict::Incorrect => "Not quite",
    };
    ui.label(format!(
        "Your answer matched {:.0} %: {verdict}. Suggested rating: {:?}.",
        scored.score.value * 100.0,
        scored.rating
    ));
    ui.weak(format!("Compared by: {}", scored.engine));
    let missed = scored.score.rubric_hits.iter().filter(|h| !**h).count();
    if missed > 0 {
        ui.label(format!("{missed} key point(s) were not covered."));
    }
}

fn reveal_controls(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if run.session.run.mode == RunMode::Answer {
        if let Some(text) = text_field(ui, "Your answer", &run.answer, "type your answer", 3) {
            out.push(Msg::AnswerEdited(text));
        }
        if ui
            .add_enabled(!run.answer.trim().is_empty(), Button::new("Submit answer"))
            .clicked()
        {
            act(out, Action::SubmitAnswer(run.answer.clone()));
        }
    }
    ui.horizontal(|ui| {
        let label = if run.session.run.mode == RunMode::Answer {
            "Show answer without answering (Space)"
        } else {
            "Show answer (Space)"
        };
        if ui.button(label).clicked() {
            act(out, Action::Flip);
        }
        if ui.button("Skip (S)").clicked() {
            act(out, Action::Skip);
        }
    });
}

fn footer(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        if run.session.run.state == RunState::Running && ui.button("Pause (P)").clicked() {
            act(out, Action::Pause);
        }
        if ui.button("Stop (Esc)").clicked() {
            act(out, Action::Abort);
        }
    });
}

fn summary(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stopped = run.session.run.state == RunState::Aborted;
    ui.heading(if stopped {
        "Run stopped"
    } else {
        "Run finished"
    });
    let s = summarize(&run.events, &run.latencies);
    let rated: usize = s.rated.iter().sum();
    if rated + s.skipped == 0 {
        ui.label("There was nothing to rehearse.");
    } else {
        ui.label(format!(
            "{rated} rated, {} skipped, in {:.0} s",
            s.skipped,
            f64::from(s.total_ms) / 1000.0
        ));
        for (i, name) in ["Again", "Hard", "Good", "Easy"].into_iter().enumerate() {
            ui.label(format!("{name}: {}", s.rated[i]));
        }
        if let Some(ms) = s.median_latency_ms {
            ui.label(format!(
                "Median time to answer: {:.1} s",
                f64::from(ms) / 1000.0
            ));
        }
    }
    if !run
        .session
        .run
        .plan
        .effect
        .eq(&studyfrog_core::plan::ScheduleEffect::UpdateSchedule)
    {
        ui.weak("Practice only: your schedule was not changed.");
    }
    if ui.button("Back to setup").clicked() {
        out.push(Msg::LeaveRun);
    }
}
