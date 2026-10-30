//! The eframe application: glue between the model, the worker and the window.

use eframe::egui;

use crate::effect::Effect;
use crate::model::{Model, apply, frame};
use crate::runtime::{Runtime, spawn};

/// The window's state.
pub struct StudyFrogApp {
    /// UI state and frame bookkeeping.
    pub model: Model,
    /// The worker thread.
    pub runtime: Runtime,
    /// When set, close the window after this many frames (smoke tests).
    pub smoke_frames: Option<u32>,
}

/// Create the app and ask the worker for the saved settings.
pub fn app_new(ctx: egui::Context, smoke_frames: Option<u32>) -> StudyFrogApp {
    let runtime = spawn(ctx);
    let _ = runtime.effects.send(Effect::LoadAppSettings);
    StudyFrogApp {
        model: Model::default(),
        runtime,
        smoke_frames,
    }
}

/// One frame: apply finished work, draw, hand new effects to the worker.
pub fn app_ui(app: &mut StudyFrogApp, ui: &mut egui::Ui) {
    let mut effects: Vec<Effect> = Vec::new();
    while let Ok(msg) = app.runtime.msgs.try_recv() {
        effects.extend(apply(&mut app.model, msg));
    }
    effects.extend(frame(&mut app.model, ui));
    for effect in effects {
        let _ = app.runtime.effects.send(effect);
    }
    close_after_smoke_frames(app, ui);
}

/// With `--smoke-frames N`, ask the window to close once N frames were drawn.
fn close_after_smoke_frames(app: &mut StudyFrogApp, ui: &egui::Ui) {
    if let Some(left) = &mut app.smoke_frames {
        *left = left.saturating_sub(1);
        if *left == 0 {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            ui.ctx().request_repaint(); // do not wait a second between smoke frames
        }
    }
}

impl eframe::App for StudyFrogApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        app_ui(self, ui);
    }
}

/// Open the window and run until it is closed. `smoke_frames` closes it after that many frames.
pub fn run(smoke_frames: Option<u32>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_title("StudyFrog"),
        ..Default::default()
    };
    eframe::run_native(
        "StudyFrog",
        options,
        Box::new(move |cc| Ok(Box::new(app_new(cc.egui_ctx.clone(), smoke_frames)))),
    )
}
