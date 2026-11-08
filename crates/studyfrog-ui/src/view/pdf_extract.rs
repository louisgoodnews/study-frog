//! The "Extract" panel of the PDF screen: make a note from pages or a selection, and the list of
//! what was already made from this PDF.

use eframe::egui::{self, ComboBox, DragValue, TextEdit};
use studyfrog_md::ulid::parse_ulid;
use studyfrog_store::overview::StackRow;

use crate::agent_types::AgentMsg;
use crate::msg::Msg;
use crate::pdf_draft_types::{DraftKind, DraftMsg};
use crate::pdf_extract_types::ExtractMsg;
use crate::pdf_types::{PdfMsg, PdfView};
use crate::state::Screen;

fn send(out: &mut Vec<Msg>, m: ExtractMsg) {
    out.push(Msg::Pdf(PdfMsg::Extract(m)));
}

fn stack_choice(view: &PdfView, stacks: &[StackRow], ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let chosen = stacks.iter().find(|s| s.folder == view.extract.folder);
    let label = chosen.map_or("Choose a stack", |s| s.title.as_str());
    ComboBox::from_label("Stack for the note")
        .selected_text(label)
        .show_ui(ui, |ui| {
            for s in stacks.iter().filter(|s| !s.folder.is_empty()) {
                let picked = ui.selectable_label(s.folder == view.extract.folder, &s.title);
                if picked.clicked() {
                    send(out, ExtractMsg::Folder(s.folder.clone()));
                }
            }
        });
}

fn form(view: &PdfView, stacks: &[StackRow], ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pages = view.info.as_ref().map_or(1, |i| i.page_count);
    stack_choice(view, stacks, ui, out);
    let mut last = view.extract.last.unwrap_or(view.page).max(view.page) + 1;
    ui.horizontal(|ui| {
        ui.label(format!("From page {} to page", view.page + 1));
        ui.add(DragValue::new(&mut last).range(view.page + 1..=pages));
    });
    if last != view.extract.last.unwrap_or(view.page).max(view.page) + 1 {
        send(out, ExtractMsg::Last(last - 1));
    }
    let mut use_selection = view.extract.use_selection;
    let words = format!("Use the {} selected words instead", view.selection.len());
    ui.add_enabled(
        !view.selection.is_empty(),
        egui::Checkbox::new(&mut use_selection, words),
    );
    if use_selection != view.extract.use_selection {
        send(out, ExtractMsg::UseSelection(use_selection));
    }
    let mut title = view.extract.title.clone();
    let name = ui.label("Title (optional)");
    ui.add(TextEdit::singleline(&mut title).desired_width(f32::INFINITY))
        .labelled_by(name.id);
    if title != view.extract.title {
        send(out, ExtractMsg::Title(title));
    }
    if ui
        .add_enabled(!view.extract.busy, egui::Button::new("Make note"))
        .clicked()
    {
        send(out, ExtractMsg::Make);
    }
}

fn sourced(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.separator();
    ui.strong("Made from this PDF");
    if view.sourced.is_empty() {
        ui.weak("Nothing yet.");
    }
    for item in &view.sourced {
        let page = item
            .page
            .map_or(String::new(), |p| format!("page {} - ", p + 1));
        let Some(uid) = parse_ulid(&item.uid) else {
            continue;
        };
        if ui.link(format!("{page}{}", item.title)).clicked() {
            out.push(Msg::OpenItem(uid));
        }
    }
}

fn pictures(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.separator();
    ui.strong("Pictures");
    ui.horizontal_wrapped(|ui| {
        if ui.button("Save the pictures of this page").clicked() {
            send(out, ExtractMsg::SavePictures);
        }
        let snap = egui::Button::selectable(view.extract.snapshot_mode, "Snapshot of a region");
        if ui
            .add(snap)
            .on_hover_text("Then drag a rectangle on the page")
            .clicked()
        {
            send(out, ExtractMsg::ToggleSnapshot);
        }
    });
    for p in &view.extract.pictures {
        ui.horizontal(|ui| {
            ui.label(p);
            let make = crate::recall_types::RecallMsg::OcclusionCard(
                view.extract.folder.clone(),
                p.clone(),
            );
            if ui
                .small_button(format!("Occlusion card from {p}"))
                .clicked()
            {
                out.push(Msg::Recall(make));
            }
            if ui.small_button("Copy Markdown").clicked() {
                let name = p.rsplit('/').next().unwrap_or(p);
                ui.ctx().copy_text(format!(
                    "![{}]({p})",
                    name.rsplit_once('.').map_or(name, |(s, _)| s)
                ));
            }
        });
    }
    let mut include = view.extract.include_pictures;
    let none = view.extract.pictures.is_empty();
    ui.add_enabled(
        !none,
        egui::Checkbox::new(&mut include, "Put these pictures at the end of the note"),
    );
    if include != view.extract.include_pictures {
        send(out, ExtractMsg::IncludePictures(include));
    }
}

fn draft(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(d) = &view.draft else {
        ui.separator();
        ui.strong("Card or question");
        ui.horizontal(|ui| {
            for (kind, label) in [
                (DraftKind::Card, "Card from selection"),
                (DraftKind::Question, "Question from selection"),
            ] {
                if ui
                    .add_enabled(!view.selection.is_empty(), egui::Button::new(label))
                    .clicked()
                {
                    out.push(Msg::Pdf(PdfMsg::Draft(DraftMsg::Start(kind, None))));
                }
            }
        });
        return;
    };
    ui.separator();
    let (front, back) = if d.kind == DraftKind::Card {
        ("Front", "Back")
    } else {
        ("Prompt", "Reference answer")
    };
    ui.strong(format!(
        "New {} from page {}",
        if d.kind == DraftKind::Card {
            "card"
        } else {
            "question"
        },
        d.page + 1
    ));
    for (label, text, is_front) in [(front, &d.front, true), (back, &d.back, false)] {
        let mut t = text.clone();
        let name = ui.label(label);
        ui.add(
            TextEdit::multiline(&mut t)
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        )
        .labelled_by(name.id);
        if &t != text {
            let m = if is_front {
                DraftMsg::Front(t)
            } else {
                DraftMsg::Back(t)
            };
            out.push(Msg::Pdf(PdfMsg::Draft(m)));
        }
    }
    ui.horizontal(|ui| {
        if ui
            .add_enabled(!d.busy, egui::Button::new("Make it"))
            .clicked()
        {
            out.push(Msg::Pdf(PdfMsg::Draft(DraftMsg::Create)));
        }
        if ui.button("Cancel").clicked() {
            out.push(Msg::Pdf(PdfMsg::Draft(DraftMsg::Cancel)));
        }
    });
}

/// Draw the panel.
pub fn draw(view: &PdfView, stacks: &[StackRow], ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong("Make a note from the text");
    ui.weak("The note keeps a link back to the page.");
    form(view, stacks, ui, out);
    pictures(view, ui, out);
    draft(view, ui, out);
    assistant(view, ui, out);
    sourced(view, ui, out);
}

/// Hand the pages to the assistant as extra reading.
fn assistant(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.separator();
    let last = view.extract.last.unwrap_or(view.page).max(view.page);
    let hint = "Opens the assistant with these pages as extra reading; nothing is sent until you press Send";
    if ui
        .button("Ask the assistant about these pages")
        .on_hover_text(hint)
        .clicked()
    {
        out.push(Msg::Agent(AgentMsg::UsePdfPages(
            view.path.clone(),
            view.page,
            last,
        )));
        out.push(Msg::Navigate(Screen::Agent));
    }
}
