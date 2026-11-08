//! The contrastive drill window (STORY-0111): the prompts side by side with the shuffled answers to match,
//! then both full items side by side with the words that set each apart, and the distinction to keep.

use eframe::egui::{self, RichText};
use studyfrog_service::contrast::ContrastTarget;

use crate::contrast::{Contrast, ContrastMsg, msg, own_words, right};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

const LETTERS: [&str; 4] = ["A", "B", "C", "D"];

fn match_column(c: &Contrast, i: usize, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let item = &c.items[i];
    ui.label(
        RichText::new(&item.prompt)
            .strong()
            .size(crate::typo::size(ui, crate::typo::Tier::Heading)),
    );
    ui.horizontal_wrapped(|ui| {
        ui.weak("Its answer:");
        for (k, letter) in LETTERS.iter().enumerate().take(c.order.len()) {
            let r = ui.selectable_label(c.picks[i] == Some(k), *letter);
            let name = format!("Answer {letter} for {}", item.prompt);
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name.clone()));
            if r.clicked() {
                out.push(msg(ContrastMsg::Pick(i, k)));
            }
        }
    });
    if c.checked {
        let ok = right(c)[i];
        let p = palette_of(ui);
        let (word, color) = if ok {
            ("Right", p.success)
        } else {
            ("Not this one", p.danger)
        };
        ui.label(RichText::new(word).color(color).strong());
    }
}

fn answers(c: &Contrast, ui: &mut egui::Ui) {
    for (k, item) in c.order.iter().enumerate() {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{}:", LETTERS[k])).strong());
            ui.label(&c.items[*item].answer);
        });
    }
}

fn reveal_column(c: &Contrast, i: usize, ui: &mut egui::Ui) {
    let item = &c.items[i];
    let others: Vec<&str> = c
        .items
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != i)
        .map(|(_, x)| x.answer.as_str())
        .collect();
    ui.label(RichText::new(&item.title).strong());
    ui.weak(&item.prompt);
    ui.label(&item.answer);
    let own = own_words(&item.answer, &others);
    if !own.is_empty() {
        ui.label(
            RichText::new(format!("Only here: {}", own.join(", "))).color(palette_of(ui).accent),
        );
    }
}

fn keep(c: &Contrast, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let prompt = ui.label("What is the key difference that distinguishes them?");
    let mut text = c.distinction.clone();
    let edit = egui::TextEdit::multiline(&mut text)
        .desired_rows(2)
        .desired_width(f32::INFINITY);
    if ui.add(edit).labelled_by(prompt.id).changed() {
        out.push(msg(ContrastMsg::Distinction(text)));
    }
    ui.horizontal_wrapped(|ui| {
        for (t, name) in [
            (ContrastTarget::Hints, "Add it as a hint to each item"),
            (ContrastTarget::Note, "Create or update a comparison note"),
        ] {
            if ui.radio(c.target == t, name).clicked() {
                out.push(msg(ContrastMsg::Target(t)));
            }
        }
    });
    let ready = !c.busy && !c.distinction.trim().is_empty();
    if ui
        .add_enabled_ui(ready, |ui| kit::primary_button(ui, "Keep the distinction"))
        .inner
        .clicked()
    {
        out.push(msg(ContrastMsg::Save));
    }
    if let Some(s) = &c.saved {
        ui.weak(s);
    }
}

fn body(c: &Contrast, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if let Some(e) = &c.error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    if c.items.is_empty() {
        ui.weak("Loading…");
        return;
    }
    let n = c.items.len();
    ui.weak("Match each prompt with its answer, then compare them side by side.");
    ui.columns(n, |cols| {
        for (i, col) in cols.iter_mut().enumerate() {
            kit::card(col, |ui| {
                ui.set_width(ui.available_width());
                match_column(c, i, ui, out);
            });
        }
    });
    if !c.checked {
        answers(c, ui);
        let all = c.picks.iter().all(Option::is_some);
        if ui
            .add_enabled_ui(all, |ui| kit::primary_button(ui, "Check the matches"))
            .inner
            .clicked()
        {
            out.push(msg(ContrastMsg::Check));
        }
        return;
    }
    ui.separator();
    ui.columns(n, |cols| {
        for (i, col) in cols.iter_mut().enumerate() {
            reveal_column(c, i, col);
        }
    });
    ui.separator();
    keep(c, ui, out);
}

/// The drill window while one is open.
pub fn window(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(c) = &state.recall.contrast else {
        return;
    };
    let mut open = true;
    egui::Window::new("Contrastive drill")
        .collapsible(false)
        .resizable(true)
        .default_width(820.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            egui::ScrollArea::vertical()
                .max_height(560.0)
                .show(ui, |ui| body(c, ui, out));
            if ui.button("Close the drill").clicked() {
                out.push(msg(ContrastMsg::Close));
            }
        });
    if !open {
        out.push(msg(ContrastMsg::Close));
    }
}
