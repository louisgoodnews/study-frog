//! `--screenshot FILE [--screen NAME] [--size WxH]`: open the window, go to a screen, save a picture of it as a
//! PNG and exit. For looking at the real rendering without a person at the keyboard.

use std::path::PathBuf;

use eframe::egui;

use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::state::{Screen, Selection};

/// Frames to wait for the vault to open and its data to load before going to the screen.
const SETTLE: u32 = 40;
/// Frames to wait on the screen before taking the picture.
const DRAW: u32 = 40;
/// Frames after going to `run` before its run is started (the count of matching items must have arrived).
const START: u32 = 10;

/// What to capture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shot {
    /// Where to write the PNG.
    pub path: PathBuf,
    /// The screen: `home`, `library`, `rehearse`, `settings`, `help`, `calendar`, `board`, `reports`, `agent`, or
    /// `run` (opens Rehearse and starts a run of everything), `exam-setup`, `exam` (an exam with a draft answer),
    /// `exam-result` (an exam answered to the end), `reports:<tab>` (a tab of the reports: `performance`, `memory`,
    /// `familiarity`, `calibration`, `runs`, `habits`, `leeches`, `workload`), `stack:<folder>` or `item:<ULID>` (the item page), `extract:<ULID>` (a note's sync preview);
    /// `None` keeps the first screen.
    pub screen: Option<String>,
    /// Frames drawn so far.
    pub frames: u32,
    /// The theme to take the picture in, whatever the saved settings say (nothing is saved).
    pub theme: Option<studyfrog_core::appearance::ThemeChoice>,
}

/// The theme named on the command line: `dark`, `light` or `system`.
pub fn theme_arg(word: &str) -> Option<studyfrog_core::appearance::ThemeChoice> {
    studyfrog_core::appearance::theme_from_name(word)
}

fn screen_named(name: &str) -> Option<Screen> {
    Some(match name {
        "home" => Screen::Home,
        "library" => Screen::Library,
        "rehearse" => Screen::Rehearse,
        "settings" => Screen::Settings,
        "help" => Screen::Help,
        "calendar" => Screen::Calendar,
        "board" => Screen::Board,
        "reports" => Screen::Reports,
        "diary" => Screen::Diary,
        "agent" => Screen::Agent,
        "prompts" => Screen::Prompts,
        "achievements" => Screen::Achievements,
        "graph" => Screen::Graph,
        "run" => Screen::Rehearse,
        _ => return None,
    })
}

fn go(model: &mut Model, name: &str) -> Vec<crate::effect::Effect> {
    if let Some(fx) = crate::app_shot_recall::setup(model, name) {
        return fx;
    }
    if let Some(folder) = name.strip_prefix("stack:") {
        let mut fx = apply(model, Msg::Navigate(Screen::Library));
        fx.extend(apply(
            model,
            Msg::Select(Selection::Stack(folder.to_owned())),
        ));
        return fx;
    }
    if let Some(uid) = name
        .strip_prefix("item:")
        .and_then(studyfrog_md::ulid::parse_ulid)
    {
        return apply(model, Msg::OpenItem(uid));
    }
    if let Some(tab) = name
        .strip_prefix("reports:")
        .and_then(crate::explorer_ui::tab_named)
    {
        let mut fx = apply(model, Msg::Navigate(Screen::Reports));
        fx.extend(apply(
            model,
            Msg::Explorer(crate::explorer_ui::ExplorerMsg::Tab(tab)),
        ));
        return fx;
    }
    if name.starts_with("exam") {
        return crate::app_shot_exam::setup(model);
    }
    if name == "create" {
        return apply(model, Msg::Modal(crate::modal_types::ModalMsg::OpenCreate));
    }
    screen_named(name).map_or_else(Vec::new, |s| apply(model, Msg::Navigate(s)))
}

pub(crate) fn save(path: &std::path::Path, image: &egui::ColorImage) -> Result<(), String> {
    let [w, h] = image.size;
    let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    let buffer = image::RgbaImage::from_raw(w as u32, h as u32, rgba)
        .ok_or("the picture has a wrong size")?;
    buffer.save(path).map_err(|e| e.to_string())
}

/// One frame of the capture: go to the screen, ask for the picture, write it when it arrives, then close.
pub fn step(shot: &mut Shot, model: &mut Model, ui: &egui::Ui) -> Vec<crate::effect::Effect> {
    shot.frames += 1;
    if let Some(t) = shot.theme {
        model.state.app.appearance.theme = t;
    }
    let ctx = ui.ctx();
    ctx.request_repaint();
    let mut effects = Vec::new();
    if let (SETTLE, Some(name)) = (shot.frames, shot.screen.clone()) {
        effects = go(model, &name);
    }
    let name = shot.screen.as_deref().unwrap_or("");
    if shot.frames == SETTLE + START && matches!(name, "run" | "exam" | "exam-result") {
        effects = apply(model, Msg::StartRunClicked);
    }
    if shot.frames == SETTLE + START {
        effects.extend(crate::app_shot_recall::later(model, name));
    }
    if shot.frames > SETTLE + START && shot.frames < SETTLE + DRAW - 8 {
        effects.extend(crate::app_shot_exam::play(model, name));
    }
    if shot.frames == SETTLE + DRAW {
        let (path, done) = (shot.path.clone(), ctx.clone());
        let callback = egui::ScreenshotCallback::new(move |image| {
            match save(&path, &image) {
                Ok(()) => eprintln!("screenshot: wrote {}", path.display()),
                Err(e) => eprintln!("screenshot: {e}"),
            }
            done.send_viewport_cmd(egui::ViewportCommand::Close);
        });
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
    }
    effects
}
