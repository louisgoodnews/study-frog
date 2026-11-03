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
const DRAW: u32 = 20;

/// What to capture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shot {
    /// Where to write the PNG.
    pub path: PathBuf,
    /// The screen: `home`, `library`, `rehearse`, `settings`, `help`, `calendar`, `board`, `reports`, `agent`, or
    /// `stack:<folder>`; `None` keeps the first screen.
    pub screen: Option<String>,
    /// Frames drawn so far.
    pub frames: u32,
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
        "agent" => Screen::Agent,
        "achievements" => Screen::Achievements,
        "graph" => Screen::Graph,
        _ => return None,
    })
}

fn go(model: &mut Model, name: &str) -> Vec<crate::effect::Effect> {
    if let Some(folder) = name.strip_prefix("stack:") {
        let mut fx = apply(model, Msg::Navigate(Screen::Library));
        fx.extend(apply(
            model,
            Msg::Select(Selection::Stack(folder.to_owned())),
        ));
        return fx;
    }
    screen_named(name).map_or_else(Vec::new, |s| apply(model, Msg::Navigate(s)))
}

fn save(path: &std::path::Path, image: &egui::ColorImage) -> Result<(), String> {
    let [w, h] = image.size;
    let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    let buffer = image::RgbaImage::from_raw(w as u32, h as u32, rgba)
        .ok_or("the picture has a wrong size")?;
    buffer.save(path).map_err(|e| e.to_string())
}

/// One frame of the capture: go to the screen, ask for the picture, write it when it arrives, then close.
pub fn step(shot: &mut Shot, model: &mut Model, ui: &egui::Ui) -> Vec<crate::effect::Effect> {
    shot.frames += 1;
    let ctx = ui.ctx();
    ctx.request_repaint();
    let mut effects = Vec::new();
    if let (SETTLE, Some(name)) = (shot.frames, shot.screen.clone()) {
        effects = go(model, &name);
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
