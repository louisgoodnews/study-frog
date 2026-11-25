//! The saved-sessions browser under the session form (TASK-0295): one row per session with its
//! title, topic, date and counts, and the resume, export and confirmed delete actions.

use eframe::egui::{self, Button, RichText};
use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_core::classroom::{SourceKind, SourceRef};
use studyfrog_service::classroom_material::pdf_source_uid;

use crate::classroom_types::{ClassroomMsg, SavedMsg, SavedSession};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::home_hero;

const DAY_MS: i64 = 86_400_000;

fn send(out: &mut Vec<Msg>, m: SavedMsg) {
    out.push(Msg::Classroom(ClassroomMsg::Saved(m)));
}

/// The list of saved sessions, or a line saying there are none yet.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let c = &state.classroom;
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new("Saved sessions").strong());
        ui.add_space(6.0);
        if c.saved.is_empty() {
            ui.weak("No saved sessions yet. Start one above; it is kept here.");
        }
        for session in &c.saved {
            row(state, session, ui, out);
            ui.separator();
        }
        if let Some(notice) = &c.saved_notice {
            ui.label(RichText::new(notice).small());
        }
    });
}

fn row(state: &State, s: &SavedSession, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let c = &state.classroom;
    ui.label(RichText::new(&s.title).strong());
    ui.label(format!("Topic: {}", topic_of(state, &s.sources)));
    ui.label(
        RichText::new(meta_line(s))
            .small()
            .color(palette_of(ui).text_weak),
    );
    ui.set_width(ui.available_width());
    ui.horizontal_wrapped(|ui| {
        let resume = ui
            .add_enabled(!c.busy, Button::new("Resume"))
            .on_hover_text("Opens this session where it left off.");
        resume.widget_info(|| access(&resume_label(&s.title)));
        if resume.clicked() {
            send(out, SavedMsg::Resume(s.id));
        }
        let export = ui
            .add(Button::new("Export"))
            .on_hover_text("Writes the transcript as Markdown into the vault's exports folder.");
        export.widget_info(|| access(&format!("Export {} to Markdown", s.title)));
        if export.clicked() {
            send(out, SavedMsg::Export(s.id));
        }
        if c.confirm_delete == Some(s.id) {
            confirm(ui, out);
        } else {
            let delete = ui
                .add(Button::new("Delete"))
                .on_hover_text("Asks for confirmation before this session is deleted.");
            delete.widget_info(|| access(&format!("Delete {}", s.title)));
            if delete.clicked() {
                send(out, SavedMsg::AskDelete(s.id));
            }
        }
    });
}

/// The question and the two answers, shown in place of the delete button until one is given.
fn confirm(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(RichText::new("Delete this session for good?").color(palette_of(ui).warning));
    let yes = ui
        .add(Button::new("Confirm delete"))
        .on_hover_text("Deletes the session and its transcript. This cannot be undone.");
    yes.widget_info(|| access("Confirm delete"));
    if yes.clicked() {
        send(out, SavedMsg::ConfirmDelete);
    }
    let keep = ui
        .add(Button::new("Keep it"))
        .on_hover_text("Keeps the session; nothing is deleted.");
    keep.widget_info(|| access("Keep it"));
    if keep.clicked() {
        send(out, SavedMsg::CancelDelete);
    }
}

fn resume_label(title: &str) -> String {
    format!("Resume {title}")
}

fn access(label: &str) -> egui::WidgetInfo {
    egui::WidgetInfo::labeled(egui::Role::Button, true, label)
}

/// The date, the turn count and the task count on one line.
fn meta_line(s: &SavedSession) -> String {
    let day = s.created_at.0.div_euclid(DAY_MS);
    format!(
        "{} · {} turns · {} tasks",
        format_date(date_from_days(day)),
        s.turns,
        s.tasks
    )
}

/// The names of the stacks and PDFs the session was grounded in, joined.
fn topic_of(state: &State, sources: &[SourceRef]) -> String {
    if sources.is_empty() {
        return "none".to_owned();
    }
    let names: Vec<String> = sources.iter().map(|s| source_name(state, s)).collect();
    names.join(", ")
}

fn source_name(state: &State, source: &SourceRef) -> String {
    match source.kind {
        SourceKind::Stack => state
            .vault
            .iter()
            .flat_map(|v| v.overview.stacks.iter())
            .find(|st| st.uid == source.uid)
            .map_or_else(
                || "a stack no longer in the vault".to_owned(),
                |st| st.title.clone(),
            ),
        SourceKind::Pdf => state
            .pdf_library
            .rows
            .iter()
            .find(|r| pdf_source_uid(&r.path) == source.uid)
            .map_or_else(
                || "a PDF no longer in the library".to_owned(),
                |r| r.path.clone(),
            ),
        SourceKind::Note => "a note".to_owned(),
    }
}
