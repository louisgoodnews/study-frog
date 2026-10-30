#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Helpers for the headless view tests.

use egui_kittest::Harness;
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::settings::vault_settings_default;
use studyfrog_core::uid::Uid;
use studyfrog_store::overview::{Problem, StackRow, VaultOverview};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply, frame};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::VaultInfo;

pub type App = (Model, Vec<Effect>);

pub fn harness(model: Model) -> Harness<'static, App> {
    Harness::builder()
        .with_size(eframe::egui::vec2(900.0, 600.0))
        .build_ui_state(
            |ui, (model, effects): &mut App| effects.extend(frame(model, ui)),
            (model, Vec::new()),
        )
}

pub fn stack(title: &str, folder: &str, notes: usize, cards: usize) -> StackRow {
    StackRow {
        uid: Uid(1),
        title: title.into(),
        folder: folder.into(),
        notes,
        cards,
        questions: 0,
        scheduler: None,
        daily_new_limit: Some(20),
        daily_review_limit: None,
    }
}

pub fn info() -> VaultInfo {
    VaultInfo {
        root: "/v/notes".into(),
        name: "notes".into(),
        overview: VaultOverview {
            stacks: vec![
                stack("French", "french", 1, 2),
                StackRow {
                    scheduler: Some(Scheduler::Sm2),
                    ..stack("Maths", "maths", 0, 5)
                },
            ],
            inbox_items: 3,
            problems: vec![Problem {
                path: "broken.md".into(),
                message: "missing `---` front-matter block".into(),
            }],
        },
        settings: vault_settings_default(),
        settings_error: None,
    }
}

pub fn open_model() -> Model {
    let mut m = Model::default();
    apply(&mut m, Msg::VaultOpened(Ok(info())));
    m
}

/// A model with a two-card run on screen (Paris / 7x8), content loaded, the first item shown.
pub fn running_model(mode: studyfrog_core::run::RunMode) -> Model {
    use studyfrog_core::plan::plan_spaced;
    use studyfrog_core::run::{Run, RunState};
    use studyfrog_service::content::ItemContent;
    let run = Run {
        uid: Uid(100),
        mode,
        plan: plan_spaced(),
        queue: vec![Uid(1), Uid(2)],
        state: RunState::Idle,
        started_at: None,
    };
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(studyfrog_ui::state::Screen::Rehearse));
    m.state.now = 0.0;
    apply(&mut m, Msg::RunReady(Ok(run)));
    for (uid, prompt, answer) in [(1, "Capital of France?", "Paris"), (2, "7 x 8?", "56")] {
        let content = ItemContent {
            title: prompt.into(),
            prompt: prompt.into(),
            answer: answer.into(),
            previous: None,
        };
        apply(&mut m, Msg::ItemLoaded(Uid(uid), Ok(content)));
    }
    m
}

/// Every control must be reachable by a screen reader, so it needs a name.
pub fn assert_all_named(h: &Harness<'_, App>, screen: &str) {
    use eframe::egui::accesskit::Role;
    use egui_kittest::kittest::{NodeT, Queryable};
    for role in [Role::Button, Role::TextInput, Role::Label] {
        for node in h.query_all_by_role(role) {
            // Static text carries its content as `value`; controls must have a real `label`.
            let n = node.accesskit_node();
            let name = if role == Role::Label {
                n.label().or_else(|| n.value())
            } else {
                n.label()
            };
            assert!(
                !name.unwrap_or_default().trim().is_empty(),
                "{screen}: a {role:?} has no accessible name"
            );
        }
    }
}
