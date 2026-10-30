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
