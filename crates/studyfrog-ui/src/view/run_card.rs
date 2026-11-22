//! The inside of the rehearsal card: the prompt, the answer, the marks and the rating buttons.

mod parts;
use self::parts::{case_controls, case_result, mark_row, reveal_controls, source_link};
use eframe::egui::{self, Button, Color32, FontId, Key, RichText, Stroke, TextStyle, vec2};
use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_md::blocks::Block;
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::session::current_item;

use crate::msg::Msg;
use crate::run_state::{ItemContent, RunView, blocks_of};
use crate::run_update::{awaiting_rating, awaiting_reveal};
use crate::theme::{palette_of, tint};
use crate::view::kit;
use crate::view::preview::draw_blocks;
use crate::view::widgets::text_field;

pub(crate) const RATINGS: [(Rating, &str, Key); 4] = [
    (Rating::Again, "Again (1)", Key::Num1),
    (Rating::Hard, "Hard (2)", Key::Num2),
    (Rating::Good, "Good (3)", Key::Num3),
    (Rating::Easy, "Easy (4)", Key::Num4),
];

pub(crate) const MARKS: [(Difficulty, &str, Key); 3] = [
    (Difficulty::Easy, "Easy (E)", Key::E),
    (Difficulty::Medium, "Medium (M)", Key::M),
    (Difficulty::Difficult, "Difficult (D)", Key::D),
];

fn act(out: &mut Vec<Msg>, action: Action) {
    out.push(Msg::RunAction(action));
}

pub(crate) fn paused(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.add_space(12.0);
    ui.heading("Paused");
    ui.label("The timer is stopped and the question is hidden.");
    ui.add_space(8.0);
    if kit::primary_button(ui, "Resume (P)")
        .on_hover_text("Starts the timer again (P).")
        .clicked()
    {
        act(out, Action::Resume);
    }
}

/// Markdown blocks at a larger size, optionally in one colour.
fn markdown(
    ui: &mut egui::Ui,
    blocks: &[Block],
    size: f32,
    color: Option<Color32>,
    out: &mut Vec<Msg>,
) {
    ui.scope(|ui| {
        ui.set_max_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.style_mut()
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(size));
        ui.visuals_mut().override_text_color = color;
        draw_blocks(ui, blocks, out);
    });
}

pub(crate) fn item(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some((uid, content)) =
        current_item(&run.session).and_then(|uid| run.content.get(&uid).map(|c| (uid, c)))
    else {
        ui.add_space(24.0);
        ui.label("Loading…");
        return;
    };
    ui.add_space(6.0);
    let (prompt, answer) = run
        .blocks
        .get(&uid)
        .cloned()
        .unwrap_or_else(|| blocks_of(content));
    let heading = crate::typo::size(ui, crate::typo::Tier::Heading);
    markdown(ui, &prompt, heading, None, out);
    if let Some(v) = &content.occlusion {
        let shown = awaiting_rating(run) && !crate::exam_ui::is_exam(run);
        crate::view::occlusion_paint::run_picture(ui, v, shown);
    }
    source_link(content, ui, out);
    ui.add_space(8.0);
    if crate::exam_ui::is_exam(run) {
        crate::view::exam_card::controls(run, ui, out);
        return;
    }
    mark_row(run, ui, out);
    ui.add_space(12.0);
    if awaiting_reveal(run) {
        if let Some(case) = content.case.as_deref() {
            case_controls(run, case, ui, out);
        }
        reveal_controls(run, ui, out);
    } else if awaiting_rating(run) {
        ui.separator();
        ui.add_space(6.0);
        // The answer is body text in the text colour (APCA body target), set apart by the rule above it.
        let body = crate::typo::body_size(ui);
        markdown(ui, &answer, body, None, out);
        if content.case.is_some() {
            ui.add_space(8.0);
            case_result(run, ui);
        }
        if let Some(scored) = &run.scored {
            ui.add_space(8.0);
            score_line(scored, ui);
        }
        ui.add_space(14.0);
        rating_buttons(run, ui, out);
    }
}

/// Four equal buttons, coloured from "again" to "easy"; the suggested one is filled stronger.
fn rating_buttons(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    let colors = [p.danger, p.warning, p.success, p.accent];
    let suggested = run.scored.as_ref().map(|s| s.rating);
    ui.columns(4, |cols| {
        for (i, (rating, label, _)) in RATINGS.into_iter().enumerate() {
            let c = colors[i];
            let strong = suggested == Some(rating);
            let width = cols[i].available_width();
            let button = Button::new(RichText::new(label).color(c).strong())
                .fill(tint(c, if strong { 90 } else { 32 }))
                .stroke(Stroke::new(if strong { 2.0 } else { 1.0 }, c))
                .min_size(vec2(width, 46.0));
            if cols[i]
                .add(button)
                .on_hover_text(crate::view::tips::rating(i))
                .clicked()
            {
                act(out, Action::Rate(rating));
            }
        }
    });
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

fn is_item_typed(run: &RunView) -> bool {
    if run.session.run.mode != RunMode::Answer {
        return false;
    }
    let Some((uid, content)) =
        current_item(&run.session).and_then(|uid| run.content.get(&uid).map(|c| (uid, c)))
    else {
        return false;
    };
    if content.is_note {
        return false;
    }
    match run.session.run.plan.typed_sample {
        Some(pct) => {
            studyfrog_rehearsal::typed_sample::is_typed_sample(run.session.run.plan.seed, uid, pct)
        }
        None => true,
    }
}
