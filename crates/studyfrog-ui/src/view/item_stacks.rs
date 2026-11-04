//! The stacks an item belongs to: its primary stack and the stacks that reference it,
//! with add and remove, and the schedule, due date and familiarity of each (ADR 0011, TASK-0213).

use eframe::egui::{self, RichText, Role, WidgetInfo};
use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_md::doc::Doc;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::fam_refs::RefView;
use studyfrog_store::overview::StackRow;

use crate::editor::Editor;
use crate::msg::{EditAction, Msg};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

fn primary_folder(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(f, _)| f)
}

fn find_primary<'a>(stacks: &'a [StackRow], path: &str) -> Option<&'a StackRow> {
    let folder = primary_folder(path);
    stacks.iter().find(|s| s.folder == folder)
}

fn ref_metrics(view: Option<&RefView>) -> (String, String, String) {
    let Some(r) = view else {
        return ("–".to_owned(), "–".to_owned(), "–".to_owned());
    };
    let sched = r.stability.map_or("–".to_owned(), |s| format!("{s:.1} d"));
    let due = r.due_ms.map_or("–".to_owned(), |ms| {
        format_date(date_from_days(ms.div_euclid(86_400_000)))
    });
    let fam = r.percent.map_or("–".to_owned(), |p| format!("{p} %"));
    (sched, due, fam)
}

/// The Stacks card on the item page.
pub fn card(state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if matches!(e.doc, Doc::Stack(_)) {
        return;
    }
    let Some(vault) = &state.vault else { return };
    kit::section(ui, "Stacks & references");
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let p = palette_of(ui);
        let stacks = &vault.overview.stacks;
        let p_stack = find_primary(stacks, &e.path);
        let p_title = p_stack.map_or_else(|| "Inbox".to_owned(), |s| s.title.clone());

        let p_view = e
            .curve
            .as_ref()
            .and_then(|c| c.references.iter().find(|r| r.primary));
        let (sched, due, fam) = ref_metrics(p_view);

        ui.horizontal(|ui| {
            ui.label(RichText::new(&p_title).strong());
            kit::pill(ui, "primary", p.accent);
        });
        ui.weak(format!(
            "Schedule: {sched} • Due: {due} • Familiarity: {fam}"
        ));
        ui.separator();

        let refs = &e.form.referenced_by;
        if refs.is_empty() {
            ui.weak("Not referenced by other stacks yet.");
        }
        for id in refs {
            let matching_stack = stacks.iter().find(|s| format_ulid(s.uid) == *id);
            let title = matching_stack
                .map(|s| s.title.clone())
                .unwrap_or_else(|| id.clone());

            let r_view = e.curve.as_ref().and_then(|c| {
                c.references
                    .iter()
                    .find(|r| !r.primary && format_ulid(r.stack) == *id)
            });
            let (r_sched, r_due, r_fam) = ref_metrics(r_view);

            ui.horizontal(|ui| {
                ui.label(RichText::new(&title).strong());
                kit::pill(ui, "referenced", p.border);
                let remove_label = format!("Remove from {title}");
                let r = ui.small_button("Remove");
                r.widget_info(|| WidgetInfo::labeled(Role::Button, true, remove_label.clone()));
                if r.clicked() {
                    out.push(Msg::Edit(EditAction::RemoveReference(id.clone())));
                }
            });
            ui.weak(format!(
                "Schedule: {r_sched} • Due: {r_due} • Familiarity: {r_fam}"
            ));
        }

        ui.add_space(4.0);
        add_menu(stacks, p_stack, refs, ui, out);
    });
}

fn add_menu(
    stacks: &[StackRow],
    primary: Option<&StackRow>,
    refs: &[String],
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let primary_folder = primary.map(|s| s.folder.as_str());
    let available: Vec<_> = stacks
        .iter()
        .filter(|s| {
            Some(s.folder.as_str()) != primary_folder && !refs.contains(&format_ulid(s.uid))
        })
        .collect();

    if available.is_empty() {
        return;
    }

    ui.menu_button("Add to stack…", |ui| {
        for s in available {
            let label = format!("Add to stack {}", s.title);
            let r = ui.button(&s.title);
            r.widget_info(|| WidgetInfo::labeled(Role::Button, true, label.clone()));
            if r.clicked() {
                out.push(Msg::Edit(EditAction::AddReference(format_ulid(s.uid))));
                ui.close();
            }
        }
    });
}
