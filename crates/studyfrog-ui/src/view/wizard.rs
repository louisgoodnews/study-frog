//! The first-run setup wizard on screen (STORY-0212, TASK-0299): the stepper, the body of the current step and the
//! buttons. It replaces the central panel while it is open. The steps' bodies are in `wizard/steps.rs`.

pub mod installed;
pub mod pull;
pub mod pull_view;
pub mod steps;
pub mod types;
pub mod update;

use eframe::egui::{self, RichText};
use studyfrog_config::wizard::{self as flow, Installed, Status, Step, Wizard};

use crate::msg::Msg;
use crate::state::State;
use crate::theme::style::Style;
use crate::view::wizard::installed::installed_of;
use crate::view::wizard::types::{WizardMsg, flow_of, title_of};
use crate::view::{gradient_panel, home_hero, kit};

/// The height of the headline panel, in points (EPIC-073, STORY-0242).
const HEADLINE_HEIGHT: f32 = 150.0;

/// The wizard's whole screen: the headline on the gradient, the stepper and the step in cards, the buttons.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let current = flow_of(&state.wizard);
    let installed = installed_of(state);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            kit::column(ui, 860.0, |ui| {
                ui.add_space(16.0);
                gradient_panel::panel(ui, HEADLINE_HEIGHT, headline);
                ui.add_space(12.0);
                home_hero::card(ui, |ui| {
                    ui.set_width(ui.available_width());
                    stepper(&current, &installed, ui, out);
                });
                ui.add_space(12.0);
                home_hero::card(ui, |ui| {
                    steps::body(state, &current, &installed, ui, out);
                });
                ui.add_space(12.0);
                nav(&current, ui, out);
            });
        });
}

/// The title and the note that every step is optional, on the gradient.
fn headline(ui: &mut egui::Ui, s: Style) {
    ui.label(
        RichText::new("Set up StudyFrog")
            .size(crate::typo::size(ui, crate::typo::Tier::Heading))
            .strong()
            .color(s.gradient_text),
    );
    ui.add(
        egui::Label::new(
            RichText::new("Each step is optional. Skip any of them. Settings runs this setup again.")
                .color(s.gradient_text),
        )
        .wrap(),
    );
}

/// The Settings entry that runs the setup again (STORY-0212). Nothing is downloaded by opening it.
pub fn reopen(_state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Setup");
    ui.weak("Runs the setup again: local model, remote provider, MiniLM and a summary. A download starts only after its click.");
    if ui
        .button("Run the setup again")
        .on_hover_text(
            "Opens the setup steps on this screen. Nothing changes until you act on a step.",
        )
        .clicked()
    {
        out.push(Msg::Wizard(WizardMsg::Start));
    }
}

/// One button per step, with its status as the hint. The vault is chosen before the wizard starts, so it has none.
fn stepper(current: &Wizard, installed: &Installed, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        for step in flow::STEPS.into_iter().filter(|s| *s != Step::Vault) {
            let status = flow::status(step, installed, current);
            let hint = match status {
                Status::Current => "The step on screen.",
                Status::Done => "Already set up.",
                Status::Skipped => "Skipped earlier.",
                Status::Pending => "Not set up yet.",
            };
            if ui
                .selectable_label(status == Status::Current, title_of(step))
                .on_hover_text(hint)
                .clicked()
            {
                out.push(Msg::Wizard(WizardMsg::Goto(step)));
            }
        }
    });
}

/// Back, skip, and next or finish. Back is off on the first step; the summary ends with Finish.
fn nav(current: &Wizard, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let step = flow::current(current);
    ui.horizontal(|ui| {
        let first = step == Step::LocalModel;
        if ui
            .add_enabled(!first, egui::Button::new("Back"))
            .on_hover_text("Goes to the step before this one.")
            .on_disabled_hover_text("This is the first step.")
            .clicked()
        {
            out.push(Msg::Wizard(WizardMsg::Previous));
        }
        if step != Step::Summary
            && kit::quiet_button(ui, "Skip this step")
                .on_hover_text("Leaves this step as it is. The summary lists it as skipped, and Settings runs it again.")
                .clicked()
        {
            out.push(Msg::Wizard(WizardMsg::Skip));
        }
        if step == Step::Summary {
            if kit::primary_button(ui, "Finish")
                .on_hover_text("Closes the setup.")
                .clicked()
            {
                out.push(Msg::Wizard(WizardMsg::Finish));
            }
        } else if kit::primary_button(ui, "Next")
            .on_hover_text("Goes to the next step.")
            .clicked()
        {
            out.push(Msg::Wizard(WizardMsg::Next));
        }
    });
}
