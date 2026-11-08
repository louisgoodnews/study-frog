//! The Prompts tab: ready-made instructions for an AI, to copy or save, that make it produce files StudyFrog imports.

use eframe::egui::{self, RichText};
use studyfrog_service::prompts::{Prompt, PromptSource, prompt_with};
use studyfrog_service::prompts_meta::{CATEGORIES, category_name, params_of};

use crate::msg::Msg;
use crate::prompt_run::PromptRunMsg;
use crate::prompts_ui::{PromptsMsg, PromptsView, pick_refs, visible_prompts};
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: PromptsMsg) {
    out.push(Msg::Prompts(m));
}

/// The category pills: all, then one per category.
fn category_pills(view: &PromptsView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        if ui
            .selectable_label(view.category.is_none(), "All")
            .clicked()
        {
            send(out, PromptsMsg::Category(None));
        }
        for category in CATEGORIES {
            let here = view.category == Some(category);
            if ui.selectable_label(here, category_name(category)).clicked() {
                send(out, PromptsMsg::Category(Some(category)));
            }
        }
    });
}

/// The selectors of the options a prompt offers, such as the depth of an explanation.
fn options(view: &PromptsView, id: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    for param in params_of(id) {
        let picked = view.picks.iter().find(|(k, _)| k == param.key);
        let at = picked.map_or(0, |(_, at)| *at);
        ui.horizontal_wrapped(|ui| {
            ui.label(param.label);
            for (i, (name, _)) in param.options.iter().enumerate() {
                if ui.selectable_label(i == at, *name).clicked() {
                    send(out, PromptsMsg::Pick(param.key, i));
                }
            }
        });
    }
}

fn entry(p: &Prompt, view: &PromptsView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let id = p.id;
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(&p.title).strong());
        ui.label(RichText::new(&p.summary).color(palette_of(ui).text_weak));
        options(view, id, ui, out);
        ui.horizontal(|ui| {
            if ui
                .button("Copy to clipboard")
                .on_hover_text("Paste it into any AI chat and attach your file.")
                .clicked()
            {
                ui.ctx().copy_text(p.text.clone());
            }
            if ui.button("Save as .md").clicked() {
                send(out, PromptsMsg::Save(id));
            }
            if ui
                .button("Run with the assistant")
                .on_hover_text("Choose it below, then attach a file.")
                .clicked()
            {
                send(out, PromptsMsg::Run(PromptRunMsg::Choose(id)));
            }
        });
        egui::CollapsingHeader::new("Show the prompt")
            .id_salt(("prompt", id))
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(&p.text).monospace().size(12.0)).wrap());
            });
    });
}

fn shown(source: &PromptSource, view: &PromptsView) -> Prompt {
    prompt_with(source, &pick_refs(&view.picks))
}

/// What the Assistant's Prompts tab shows now that the prompts have a screen of their own.
pub fn moved_link(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label("The prompts have their own screen now: copy, save or run them there.");
    if ui.button("Open Prompts").clicked() {
        out.push(Msg::Navigate(crate::state::Screen::Prompts));
    }
}

/// The tab.
pub fn prompts_tab(state: &crate::state::State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (weak, view) = (palette_of(ui).text_weak, &state.prompts_view);
    ui.label(RichText::new("Prompts for any AI chat. Copy one, attach your file (notes, code, data or text from a PDF) and paste the answer into a file ending in .csv; then import it. No connection to an assistant is needed.").color(weak));
    category_pills(view, ui, out);
    if let Some(query) = text_field(ui, "Search prompts", &view.query, "Search by title", 1) {
        send(out, PromptsMsg::Query(query));
    }
    if ui.button("Save all prompts to a folder…").clicked() {
        send(out, PromptsMsg::SaveAll);
    }
    ui.add_space(6.0);
    let list = visible_prompts(view);
    if list.is_empty() {
        ui.label(RichText::new("No prompt matches.").color(weak));
    }
    for source in list {
        entry(&shown(source, view), view, ui, out);
    }
    crate::view::agent_prompt_run::run(state, ui, out);
}
