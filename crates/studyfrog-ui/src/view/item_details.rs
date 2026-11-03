//! The side column of the item page: kind, stack, subjects, priority, difficulty, tags, dates and the
//! links to and from the item.

use eframe::egui::{self, RichText, Role, WidgetInfo};
use studyfrog_core::event::{Difficulty, difficulty_name};
use studyfrog_core::priority::{PRIORITIES, priority_name};
use studyfrog_md::blocks::{Block, Link, Span};
use studyfrog_md::doc::Doc;

use crate::editor::Editor;
use crate::msg::{EditAction, Msg};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::widgets::text_field;

fn kind_word(doc: &Doc) -> &'static str {
    match doc {
        Doc::Note(_) => "note",
        Doc::Card(_) => "card",
        Doc::Question(_) => "question",
        Doc::Stack(_) => "stack",
    }
}

pub fn details(state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Details");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let p = palette_of(ui);
        ui.horizontal(|ui| {
            ui.label("Kind");
            kit::pill(ui, kind_word(&e.doc), p.accent);
        });
        if let Some(stack) = stack_path(state, &e.path) {
            ui.label(format!("In {stack}"));
        }
        if let Some(t) = text_field(ui, "Tags (comma separated)", &e.form.tags, "", 1) {
            out.push(Msg::Edit(EditAction::Tags(t)));
        }
        subject_rows(state, e, ui, out);
        priority_row(e, ui, out);
        difficulty_row(e, ui, out);
        status_row(state, e, ui, out);
        source_link(e, ui, out);
        let meta = studyfrog_md::doc::doc_meta(&e.doc);
        for (word, at) in [("Created", meta.created), ("Updated", meta.updated)] {
            if let Some(ms) = at.and_then(studyfrog_md::rfc3339::format_rfc3339) {
                ui.label(RichText::new(format!("{word} {}", &ms[..10])).color(p.text_weak));
            }
        }
    });
    ui.add_space(10.0);
    crate::view::item_fam::card(e, ui);
    links(e, ui, out);
    ui.add_space(10.0);
    linked_from(e, ui, out);
}

/// A stack picks its subjects; an item shows the ones it takes from its stack, with their teachers.
fn subject_rows(state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let book = &state.subjects.book;
    if matches!(e.doc, Doc::Stack(_)) {
        ui.label("Subjects");
        if book.subjects.is_empty() {
            ui.weak("Add subjects in Settings.");
        }
        ui.horizontal_wrapped(|ui| {
            for s in &book.subjects {
                if chip(ui, e.form.subjects.contains(&s.name), ("Subject", &s.name)) {
                    let on = !e.form.subjects.contains(&s.name);
                    out.push(Msg::Edit(EditAction::Subject(s.name.clone(), on)));
                }
            }
        });
        return;
    }
    let Some(vault) = &state.vault else { return };
    let folder = e.path.rsplit_once('/').map_or("", |(f, _)| f);
    let names = studyfrog_service::subjects::stack_subjects(&vault.overview.stacks, folder);
    if names.is_empty() {
        return;
    }
    ui.label("Subjects (from the stack)");
    for (name, teacher) in studyfrog_service::subjects::subjects_with_teachers(book, &names) {
        match teacher {
            Some(t) => ui.label(format!("{name}, taught by {t}")),
            None => ui.label(name),
        };
    }
}

/// A choice among several, named "Priority: low" for assistive technology.
fn chip(ui: &mut egui::Ui, selected: bool, (group, word): (&str, &str)) -> bool {
    let r = ui.selectable_label(selected, word);
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("{group}: {word}")));
    r.clicked()
}

fn priority_row(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label("Priority");
    ui.horizontal_wrapped(|ui| {
        if chip(ui, e.form.priority.is_none(), ("Priority", "inherit")) {
            out.push(Msg::Edit(EditAction::Priority(None)));
        }
        for p in PRIORITIES {
            if chip(
                ui,
                e.form.priority == Some(p),
                ("Priority", priority_name(p)),
            ) {
                out.push(Msg::Edit(EditAction::Priority(Some(p))));
            }
        }
    });
}

/// The status menu; an item with no stored status shows what its history makes of it, when the list is loaded.
fn status_row(state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label("Status");
    let derived = state
        .browser
        .items
        .iter()
        .find(|i| i.path == e.path)
        .map(|i| i.status);
    if let Some(choice) = crate::view::status_ui::status_menu(ui, e.form.status, derived) {
        out.push(Msg::Edit(EditAction::Status(choice)));
    }
}

fn difficulty_row(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label("Difficulty");
    ui.horizontal_wrapped(|ui| {
        if chip(ui, e.form.difficulty.is_none(), ("Difficulty", "inherit")) {
            out.push(Msg::Edit(EditAction::Difficulty(None)));
        }
        for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Difficult] {
            if chip(
                ui,
                e.form.difficulty == Some(d),
                ("Difficulty", difficulty_name(d)),
            ) {
                out.push(Msg::Edit(EditAction::Difficulty(Some(d))));
            }
        }
    });
}

/// "French / Verbs": the titles of the stacks the file sits in.
fn stack_path(state: &State, path: &str) -> Option<String> {
    let overview = &state.vault.as_ref()?.overview;
    let labels = crate::modal::stack_labels(overview);
    let folder = path.rsplit_once('/').map_or("", |(f, _)| f);
    labels
        .into_iter()
        .find(|(f, _)| f == folder)
        .map(|(_, l)| l)
}

fn wiki_targets(blocks: &[Block], into: &mut Vec<String>) {
    let spans = |spans: &[Span], into: &mut Vec<String>| {
        for s in spans {
            match &s.link {
                Some(Link::Wiki(t)) if !into.contains(t) => into.push(t.clone()),
                _ => {}
            }
        }
    };
    for b in blocks {
        match b {
            Block::Heading { spans: s, .. } | Block::Paragraph(s) => spans(s, into),
            Block::Quote(inner) => wiki_targets(inner, into),
            Block::List { items, .. } => items.iter().for_each(|i| wiki_targets(&i.blocks, into)),
            Block::Table { header, rows, .. } => {
                for cell in header.iter().chain(rows.iter().flatten()) {
                    spans(cell, into);
                }
            }
            Block::Code { .. } | Block::Rule | Block::Toc => {}
        }
    }
}

fn linked_from(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Linked from");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        if e.backlinks.is_empty() {
            ui.weak("Nothing links here yet.");
        }
        let weak = palette_of(ui).text_weak;
        for item in &e.backlinks {
            ui.horizontal(|ui| {
                kit::pill(ui, studyfrog_core::plan::item_kind_name(item.kind), weak);
                if ui.link(&item.title).clicked() {
                    out.push(Msg::OpenItem(item.uid));
                }
            });
        }
    });
}

fn links(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut targets = Vec::new();
    for f in &e.form.fields {
        wiki_targets(&studyfrog_md::blocks::parse_blocks(&f.text), &mut targets);
    }
    kit::section(ui, "Links to");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        if targets.is_empty() {
            ui.weak("No links yet. Write [[Title]] to link a note.");
        }
        for t in targets {
            if ui.link(&t).clicked() {
                out.push(Msg::OpenLink(t));
            }
        }
    });
}

/// An item made from a PDF page links back to it.
fn source_link(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(source) = studyfrog_md::extra::extra_value(&e.doc, "source") else {
        return;
    };
    let (path, page) = studyfrog_store::pdf_notes::split_source(&source);
    if !path.to_lowercase().ends_with(".pdf") {
        ui.label(format!("Source: {source}"));
        return;
    }
    let label = page.map_or(format!("Open source ({path})"), |p| {
        format!("Open source ({path}, page {})", p + 1)
    });
    if ui.link(label).clicked() {
        out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Open(
            path.to_owned(),
            page.unwrap_or(0),
        )));
    }
}
