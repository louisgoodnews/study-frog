//! The eframe application: glue between the model, the worker and the window.

use eframe::egui;

use crate::effect::Effect;
use crate::host::{Host, extensions_of};
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
    /// When set, take a picture of a screen and exit (`--screenshot`).
    pub shot: Option<crate::app_shot::Shot>,
}

/// How the window is opened.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Close the window after this many frames.
    pub smoke_frames: Option<u32>,
    /// Take a picture of a screen and exit.
    pub shot: Option<crate::app_shot::Shot>,
    /// The window's inner size in points; `None`: 960 by 640.
    pub size: Option<(f32, f32)>,
}

/// Create the app and ask the worker for the saved settings.
pub fn app_new(ctx: egui::Context, smoke_frames: Option<u32>, host: Host) -> StudyFrogApp {
    app_new_with(
        ctx,
        RunOptions {
            smoke_frames,
            ..RunOptions::default()
        },
        host,
    )
}

/// Create the app with all the options.
pub fn app_new_with(ctx: egui::Context, options: RunOptions, host: Host) -> StudyFrogApp {
    crate::theme::apply(&ctx);
    egui_extras::install_image_loaders(&ctx);
    let mut model = Model::default();
    model.state.extensions = extensions_of(&host.registry);
    let runtime = spawn(ctx, host);
    let _ = runtime.effects.send(Effect::LoadAppSettings);
    StudyFrogApp {
        model,
        runtime,
        smoke_frames: options.smoke_frames,
        shot: options.shot,
    }
}

/// One frame: apply finished work, draw, hand new effects to the worker.
pub fn app_ui(app: &mut StudyFrogApp, ui: &mut egui::Ui) {
    let mut effects: Vec<Effect> = Vec::new();
    while let Ok(msg) = app.runtime.msgs.try_recv() {
        effects.extend(apply(&mut app.model, msg));
    }
    effects.extend(frame(&mut app.model, ui));
    if let Some(shot) = app.shot.as_mut() {
        effects.extend(crate::app_shot::step(shot, &mut app.model, ui));
    }
    for effect in effects {
        let _ = app.runtime.effects.send(effect);
    }
    close_after_smoke_frames(app, ui);
    if app.model.state.chrome.quit {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }
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
pub fn run(smoke_frames: Option<u32>, host: Host) -> eframe::Result {
    run_with(
        RunOptions {
            smoke_frames,
            ..RunOptions::default()
        },
        host,
    )
}

/// Open the window with `options` and run until it is closed.
pub fn run_with(options: RunOptions, host: Host) -> eframe::Result {
    let (w, h) = options.size.unwrap_or((960.0, 640.0));
    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([w, h])
            .with_min_inner_size([640.0, 420.0])
            .with_decorations(false)
            .with_title("StudyFrog"),
        ..Default::default()
    };
    eframe::run_native(
        "StudyFrog",
        native,
        Box::new(move |cc| Ok(Box::new(app_new_with(cc.egui_ctx.clone(), options, host)))),
    )
}
