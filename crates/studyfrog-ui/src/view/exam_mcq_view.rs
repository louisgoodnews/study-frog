//! Rendering implementation for distractor multiple-choice exam view.

use eframe::egui::{self, CornerRadius, RichText, Stroke, Vec2};
use studyfrog_rehearsal::distractor::MultipleChoiceQuestion;

use crate::theme::{Palette, palette_of, tint};
use crate::view::exam_mcq::{McqState, mcq_select_choice, mcq_submit_or_next, new_mcq_state};
use crate::view::kit;

/// Render the MCQ view into an `egui::Ui` using the context's palette.
pub fn render_mcq_view(ui: &mut egui::Ui, state: &mut McqState) {
    let p = palette_of(ui);
    render_mcq_view_with_palette(ui, state, &p);
}

/// Render the MCQ view with an explicitly provided palette.
pub fn render_mcq_view_with_palette(ui: &mut egui::Ui, state: &mut McqState, p: &Palette) {
    if let Some(question) = state.current_question.clone() {
        render_question(ui, state, &question, p);
    } else {
        render_report(ui, state, p);
    }
}

fn choice_tag(idx: usize) -> char {
    if idx < 26 {
        (b'A' + idx as u8) as char
    } else {
        '•'
    }
}

fn render_question(
    ui: &mut egui::Ui,
    state: &mut McqState,
    q: &MultipleChoiceQuestion,
    p: &Palette,
) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new("Distractor Exam").color(p.text_weak));
            if state.total_answered > 0 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    kit::pill(
                        ui,
                        &format!("Score: {}/{}", state.score, state.total_answered),
                        p.accent,
                    );
                });
            }
        });
        ui.add_space(8.0);
        let heading_size = crate::typo::size(ui, crate::typo::Tier::Heading);
        ui.label(
            RichText::new(&q.prompt)
                .size(heading_size)
                .strong()
                .color(p.text),
        );
        ui.add_space(12.0);

        render_choices(ui, state, q, p);
        ui.add_space(12.0);
        render_actions(ui, state, q, p);
    });
}

fn render_choices(
    ui: &mut egui::Ui,
    state: &mut McqState,
    q: &MultipleChoiceQuestion,
    p: &Palette,
) {
    for (idx, choice) in q.all_choices.iter().enumerate() {
        let tag = choice_tag(idx);
        let sel = state.selected_choice == Some(idx);
        let (fill, stroke, text_color, prefix) = if state.is_revealed {
            if idx == q.correct_index {
                (
                    tint(p.success, 40),
                    Stroke::new(2.0, p.success),
                    p.success,
                    "✓ ",
                )
            } else if sel {
                (
                    tint(p.danger, 40),
                    Stroke::new(2.0, p.danger),
                    p.danger,
                    "✗ ",
                )
            } else {
                (p.surface, Stroke::new(1.0, p.border), p.text_weak, "  ")
            }
        } else if sel {
            (
                tint(p.accent, 35),
                Stroke::new(2.0, p.accent),
                p.accent,
                "● ",
            )
        } else {
            (p.surface, Stroke::new(1.0, p.border), p.text, "○ ")
        };

        let label = format!("{prefix}{tag}. {choice}");
        let btn = egui::Button::new(RichText::new(label).color(text_color).strong())
            .fill(fill)
            .stroke(stroke)
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(ui.available_width(), 36.0));

        if !state.is_revealed {
            if ui.add(btn).clicked() {
                mcq_select_choice(state, idx);
            }
        } else {
            ui.add_enabled(false, btn);
        }
        ui.add_space(4.0);
    }
}

fn render_actions(
    ui: &mut egui::Ui,
    state: &mut McqState,
    q: &MultipleChoiceQuestion,
    p: &Palette,
) {
    if !state.is_revealed {
        let ready = state.selected_choice.is_some();
        ui.horizontal(|ui| {
            let col = if ready { p.on_accent } else { p.text_weak };
            let fill = if ready { p.accent } else { tint(p.accent, 50) };
            let btn = egui::Button::new(RichText::new("Submit answer").color(col).strong())
                .fill(fill)
                .min_size(Vec2::new(140.0, 36.0));
            if ui.add_enabled(ready, btn).clicked() {
                mcq_submit_or_next(state);
            }
            if !ready {
                ui.weak("Select a choice to submit");
            }
        });
    } else {
        ui.horizontal(|ui| {
            let correct = state.selected_choice == Some(q.correct_index);
            let (fb, col) = if correct {
                ("Correct!", p.success)
            } else {
                ("Incorrect", p.danger)
            };
            kit::pill(ui, fb, col);
            let btn = egui::Button::new(RichText::new("Next question").color(p.on_accent).strong())
                .fill(p.accent)
                .min_size(Vec2::new(140.0, 36.0));
            if ui.add(btn).clicked() {
                mcq_submit_or_next(state);
            }
        });
    }
}

fn render_report(ui: &mut egui::Ui, state: &mut McqState, p: &Palette) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let heading_size = crate::typo::size(ui, crate::typo::Tier::Heading);
        ui.label(
            RichText::new("Exam Report")
                .size(heading_size)
                .strong()
                .color(p.text),
        );
        ui.add_space(8.0);
        if state.total_answered > 0 {
            let pct = (state.score as f32 / state.total_answered as f32) * 100.0;
            let passed = pct >= 60.0;
            ui.horizontal(|ui| {
                let txt = format!(
                    "Score: {} / {} ({:.1} %)",
                    state.score, state.total_answered, pct
                );
                ui.label(RichText::new(txt).strong().color(p.text));
                let (badge, color) = if passed {
                    ("Passed", p.success)
                } else {
                    ("Not passed", p.danger)
                };
                kit::pill(ui, badge, color);
            });
            ui.add_space(12.0);
            let btn = egui::Button::new(RichText::new("Restart Exam").color(p.on_accent).strong())
                .fill(p.accent)
                .min_size(Vec2::new(140.0, 36.0));
            if ui.add(btn).clicked() {
                *state = new_mcq_state();
            }
        } else {
            ui.weak("No questions active.");
        }
    });
}
