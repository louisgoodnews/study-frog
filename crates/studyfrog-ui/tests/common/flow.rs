#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Driving the UI through update and the real executor.

use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::kanban::BoardMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

use super::{TempDir, run_to_idle, sample_vault};

/// A sample vault opened in a fresh model and worker.
pub fn opened(label: &str) -> (TempDir, Model, Worker) {
    let vault = sample_vault(label);
    let mut worker = worker_new(None);
    let mut model = Model::default();
    model.state.app.tutorial_offered = true;
    let path = vault.path.display().to_string();
    run_to_idle(&mut model, &mut worker, vec![Effect::OpenVault(path)]);
    assert!(model.state.vault.is_some());
    (vault, model, worker)
}

/// Apply a message and run its effects until the UI is idle.
pub fn send(model: &mut Model, worker: &mut Worker, msg: Msg) {
    let effects = apply(model, msg);
    run_to_idle(model, worker, effects);
}

/// Send a board message.
pub fn board(model: &mut Model, worker: &mut Worker, m: BoardMsg) {
    send(model, worker, Msg::Board(m));
}

/// Card titles of one board column.
pub fn titles(model: &Model, column: usize) -> Vec<String> {
    let b = model.state.kanban.board.as_ref().unwrap();
    b.columns[column]
        .cards
        .iter()
        .map(|c| c.title.clone())
        .collect()
}
