//! The job editor's form (STORY-0151): each job with its status and next run, its details (schedule, events, recent
//! runs), runs by hand, and the save. A run by hand is typed into the Console, so it is the same as `script run`.

use eframe::egui::{self, RichText};

use crate::console_ui::ConsoleMsg;
use crate::console_ui::jobs::{JobsEditor, JobsMsg};
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::theme::palette_of;

fn send(out: &mut Vec<Msg>, m: JobsMsg) {
    out.push(Msg::Console(ConsoleMsg::Jobs(m)));
}

/// Run a job or a script by hand: the Console runs it, as a typed `script run` would.
fn run_by_hand(out: &mut Vec<Msg>, what: &str) {
    out.push(Msg::Console(ConsoleMsg::Edit(format!("script run {what}"))));
    out.push(Msg::Console(ConsoleMsg::Submit));
}

/// The job editor's contents.
pub fn draw(e: &JobsEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    // The form is taller than a small window: it scrolls inside the dialog, which stays within the window.
    let max = (ui.ctx().content_rect().height() * 0.85).max(240.0);
    ui.set_height(max);
    egui::ScrollArea::vertical()
        .max_height(max)
        .auto_shrink([false, true])
        .show(ui, |ui| body(e, ui, out));
}

fn body(e: &JobsEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    ui.horizontal(|ui| {
        ui.heading("Scheduled jobs");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("Close")
                .on_hover_text("Closes the job editor. Saved changes stay saved.")
                .clicked()
            {
                out.push(Msg::Modal(ModalMsg::Close));
            }
        });
    });
    ui.weak("Jobs run on their schedule or when their event happens, from the app's scripts folder. They run only while StudyFrog runs.");
    for err in &e.errors {
        ui.label(RichText::new(format!("Not used: {err}")).color(p.danger));
    }
    ui.add_space(4.0);
    if e.rows.is_empty() {
        ui.weak("No jobs yet. Add them to scripts.toml in the scripts folder.");
    }
    for (i, r) in e.rows.iter().enumerate() {
        ui.horizontal(|ui| {
            let (status, color) = if !r.enabled {
                ("off", p.text_weak)
            } else if !r.on.is_empty() {
                ("on event", p.accent)
            } else if !r.schedule.is_empty() {
                ("on schedule", p.success)
            } else {
                ("by hand", p.text_weak)
            };
            ui.label(RichText::new(status).color(color).strong());
            let selected = e.selected == Some(i);
            if ui
                .selectable_label(selected, &r.name)
                .on_hover_text("Shows this job's details.")
                .clicked()
            {
                send(out, JobsMsg::Select(i));
            }
            let next = r.next_run.as_deref().unwrap_or("no next run");
            ui.weak(format!("next: {next}"));
            if ui
                .button("Run now")
                .on_hover_text("Runs this job now, as a typed script run in the Console.")
                .clicked()
            {
                run_by_hand(out, &r.name);
            }
            let mut on = r.enabled;
            if ui
                .checkbox(&mut on, "enabled")
                .on_hover_text("A job that is off never runs by its schedule or event.")
                .changed()
            {
                send(out, JobsMsg::Enabled(i, on));
            }
        });
    }
    if let Some(i) = e.selected.filter(|i| *i < e.rows.len()) {
        details(e, i, ui, out);
    }
    ui.separator();
    new_job(e, ui, out);
    ui.separator();
    adhoc(e, ui, out);
    ui.separator();
    ui.horizontal(|ui| {
        if ui
            .add_enabled(e.dirty, egui::Button::new("Save jobs"))
            .on_hover_text(
                "Writes the jobs to scripts.toml, if the file reads back with no errors.",
            )
            .clicked()
        {
            send(out, JobsMsg::Save);
        }
        if let Some(n) = &e.notice {
            ui.label(n);
        }
    });
}

fn details(e: &JobsEditor, i: usize, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let r = &e.rows[i];
    ui.separator();
    ui.label(RichText::new(format!("{} runs {}", r.name, r.script)).strong());
    let mut schedule = r.schedule.clone();
    ui.horizontal(|ui| {
        ui.label("Schedule");
        if ui
            .add(
                egui::TextEdit::singleline(&mut schedule)
                    .hint_text("0 9 * * mon-fri, @daily or empty")
                    .desired_width(220.0),
            )
            .changed()
        {
            send(out, JobsMsg::Schedule(i, schedule.clone()));
        }
    });
    ui.label("Starts on these events:");
    ui.horizontal_wrapped(|ui| {
        for event in crate::script_events::EVENT_KINDS {
            let mut on = r.on.iter().any(|x| x == event);
            if ui.checkbox(&mut on, *event).changed() {
                send(out, JobsMsg::EventOn(i, (*event).to_owned(), on));
            }
        }
    });
    ui.weak(format!(
        "Timeout {} s, access {:?}.",
        r.timeout_secs, r.access
    ));
    ui.label("Recent runs:");
    if e.history.is_empty() {
        ui.weak("No runs in the app log yet.");
    }
    for line in &e.history {
        ui.monospace(line);
    }
}

fn adhoc(e: &JobsEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(RichText::new("Run a script by hand").strong());
    if e.scripts.is_empty() {
        ui.weak("No scripts in the scripts folder yet.");
        return;
    }
    let mut script = e.adhoc_script.clone();
    let mut args = e.adhoc_args.clone();
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("jobs-adhoc-script")
            .selected_text(if script.is_empty() { "choose a script" } else { script.as_str() })
            .show_ui(ui, |ui| {
                for s in &e.scripts {
                    ui.selectable_value(&mut script, s.clone(), s);
                }
            });
        if ui
            .add(egui::TextEdit::singleline(&mut args).hint_text("arguments").desired_width(200.0))
            .changed()
        {
            send(out, JobsMsg::AdhocArgs(args.clone()));
        }
        if ui
            .add_enabled(!script.is_empty(), egui::Button::new("Run"))
            .on_hover_text("Runs the chosen script once with these arguments, as a typed script run in the Console.")
            .clicked()
        {
            let line = if args.trim().is_empty() {
                script.clone()
            } else {
                format!("{script} {}", args.trim())
            };
            run_by_hand(out, &line);
        }
    });
    if script != e.adhoc_script {
        send(out, JobsMsg::AdhocScript(script));
    }
}

fn new_job(e: &JobsEditor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(RichText::new("New job").strong());
    if e.scripts.is_empty() {
        ui.weak("Put a script in the scripts folder first.");
        return;
    }
    let mut name = e.new_name.clone();
    let mut script = e.new_script.clone();
    ui.horizontal(|ui| {
        if ui
            .add(egui::TextEdit::singleline(&mut name).hint_text("job name").desired_width(160.0))
            .changed()
        {
            send(out, JobsMsg::NewName(name.clone()));
        }
        egui::ComboBox::from_id_salt("jobs-new-script")
            .selected_text(if script.is_empty() { "choose a script" } else { script.as_str() })
            .show_ui(ui, |ui| {
                for s in &e.scripts {
                    ui.selectable_value(&mut script, s.clone(), s);
                }
            });
        if ui
            .button("Add job")
            .on_hover_text("Adds a job for this script, switched on, with no schedule or events yet; set those, then save.")
            .clicked()
        {
            send(out, JobsMsg::AddJob);
        }
    });
    if script != e.new_script {
        send(out, JobsMsg::NewScript(script));
    }
}
