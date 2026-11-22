//! Parts of the rehearsal card: the reveal and mark controls and the source link.
use super::*;

pub(super) fn mark_row(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let chosen = crate::review::current_mark(run);
    ui.horizontal(|ui| {
        ui.weak("This item is:");
        for (difficulty, label, _) in MARKS {
            if ui
                .selectable_label(chosen == Some(difficulty), label)
                .on_hover_text(crate::view::tips::mark(difficulty))
                .clicked()
            {
                act(out, Action::Mark(difficulty));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(uid) = current_item(&run.session)
                && ui
                    .small_button("Edit")
                    .on_hover_text("Opens this for editing.")
                    .clicked()
            {
                out.push(Msg::RehearsalEdit(
                    crate::rehearsal_edit::RehearsalEditMsg::Open(uid),
                ));
            }
        });
    });
}

pub(super) fn source_link(content: &ItemContent, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if let Some(source) = &content.source {
        crate::view::pdf_split::source_link(source, ui, out);
    }
}

pub(super) fn reveal_controls(run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let answering = is_item_typed(run) && !is_case_item(run);
    if answering {
        if let Some(text) = text_field(ui, "Your answer", &run.answer, "type your answer", 3) {
            out.push(Msg::AnswerEdited(text));
        }
        let ready = !run.answer.trim().is_empty();
        if ui
            .add_enabled_ui(ready, |ui| {
                kit::primary_button(ui, "Submit answer").on_hover_text("Checks your typed answer.")
            })
            .inner
            .clicked()
        {
            act(out, Action::SubmitAnswer(run.answer.clone()));
        }
        ui.add_space(4.0);
    }
    ui.horizontal(|ui| {
        let label = if answering {
            "Show answer without answering (Space)"
        } else {
            "Show answer (Space)"
        };
        let show = if answering {
            ui.button(label)
                .on_hover_text(crate::view::tips::show_answer())
        } else {
            kit::primary_button(ui, label).on_hover_text(crate::view::tips::show_answer())
        };
        if show
            .on_hover_text(crate::view::tips::show_answer())
            .clicked()
        {
            act(out, Action::Flip);
        }
        if ui
            .button("Skip (S)")
            .on_hover_text(crate::view::tips::skip())
            .clicked()
        {
            act(out, Action::Skip);
        }
    });
}

/// Whether the item on screen is a case study (its options replace the typed answer box).
fn is_case_item(run: &RunView) -> bool {
    current_item(&run.session)
        .and_then(|uid| run.content.get(&uid))
        .is_some_and(|c| c.case.is_some())
}

/// The case study's options and justification, and the check of them (STORY-0199).
pub(super) fn case_controls(
    run: &RunView,
    case: &studyfrog_rehearsal::case_study::CaseStudyItem,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    ui.label(egui::RichText::new(&case.decision_prompt).strong());
    for (i, option) in case.options.iter().enumerate() {
        let mut on = run.case_run.picks.get(i).copied().unwrap_or(false);
        if ui.checkbox(&mut on, option).changed() {
            out.push(Msg::Case(crate::setup::case::CaseMsg::RunPick(i, on)));
        }
    }
    if let Some(text) = text_field(
        ui,
        "Your justification",
        &run.answer,
        "explain why this choice",
        4,
    ) {
        out.push(Msg::AnswerEdited(text));
    }
    let ready = !run.answer.trim().is_empty();
    let check = ui.add_enabled_ui(ready, |ui| {
        kit::primary_button(ui, "Check my answer")
            .on_hover_text("Checks the choice and the justification against the case's rubric, then shows the answer.")
    });
    if check.inner.clicked() {
        out.push(Msg::Case(crate::setup::case::CaseMsg::RunCheck));
        out.push(Msg::RunAction(Action::Flip));
    }
    ui.add_space(4.0);
}

/// The check's result under the revealed answer: the choice, and a met or missing pill per rubric point.
pub(super) fn case_result(run: &RunView, ui: &mut egui::Ui) {
    let Some(report) = &run.case_run.report else {
        return;
    };
    let pal = palette_of(ui);
    ui.horizontal_wrapped(|ui| {
        if report.choice_valid {
            kit::pill(ui, "Choice: the expected one", pal.success);
        } else {
            kit::pill(ui, "Choice: not the expected one", pal.danger);
        }
    });
    for result in &report.criteria {
        ui.horizontal_wrapped(|ui| {
            if result.met {
                kit::pill(ui, "Met", pal.success);
            } else {
                kit::pill(ui, "Missing", pal.danger);
            }
            ui.label(&result.criterion);
        });
    }
}

/// The line above the rating button that matches the previous rating: "Last: <word>" in the colour of its
/// difficulty. The other buttons have no line.
pub(super) fn last_time_line(
    p: &crate::theme::Palette,
    previous: Option<Rating>,
    rating: Rating,
    size: f32,
) -> Option<RichText> {
    match previous {
        Some(r) if r == rating => {
            let (word, mark) = previous_mark(p, rating);
            Some(
                RichText::new(format!("Last: {word}"))
                    .size(size)
                    .color(mark)
                    .strong(),
            )
        }
        _ => None,
    }
}

/// The difficulty word a previous rating stands for, and its colour: difficult in red, medium in orange,
/// easy (and good) in green. The word is shown with the colour, so colour is not the only cue (STORY-0196).
fn previous_mark(p: &crate::theme::Palette, rating: Rating) -> (&'static str, Color32) {
    match rating {
        Rating::Again => ("Difficult", p.danger),
        Rating::Hard => ("Medium", p.warning),
        Rating::Good | Rating::Easy => ("Easy", p.success),
    }
}
