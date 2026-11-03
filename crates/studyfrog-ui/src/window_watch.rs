//! The window and the saved look at start-up, and noticing that the window has moved or changed size. The shell reads
//! egui's viewport; whether to save it is decided in [`crate::appearance_ui`].

use eframe::egui;
use studyfrog_core::appearance::{Appearance, WindowState};
use studyfrog_store::app_config::{profile_dir, read_app_settings, read_profiles};

use crate::appearance_ui::{AppearanceMsg, wants_window};

/// Seconds the window must stay as it is before it is saved, so that dragging does not write the file every frame.
const SETTLE: f64 = 1.0;

/// The saved appearance of the current profile, or the defaults when it cannot be read: used before the application's
/// own start-up has read the settings, so the first frame already has the right look and the window the right size.
pub fn saved_appearance() -> Appearance {
    let Some(root) = crate::runtime::config_dir() else {
        return Appearance::default();
    };
    let profiles = read_profiles(&root).unwrap_or_default();
    read_app_settings(&profile_dir(&root, &profiles.current))
        .map_or_else(|_| Appearance::default(), |s| s.appearance)
}

/// The window as egui reports it now, when it reports a size.
pub fn window_now(ctx: &egui::Context) -> Option<WindowState> {
    ctx.input(|i| {
        let v = i.viewport();
        let size = v.inner_rect?.size();
        let place = v.outer_rect.map(|r| r.min);
        Some(WindowState {
            width: size.x.round().max(1.0) as u32,
            height: size.y.round().max(1.0) as u32,
            x: place.map(|p| p.x.round() as i32),
            y: place.map(|p| p.y.round() as i32),
            maximized: v.maximized.unwrap_or(false),
        })
    })
}

/// The message to save the window, once it has stayed as it is for a moment and differs from what is saved.
pub fn watch(
    seen: &mut Option<(WindowState, f64)>,
    ctx: &egui::Context,
    time: f64,
    a: &Appearance,
) -> Option<AppearanceMsg> {
    let now = window_now(ctx)?;
    let since = match seen {
        Some((w, t)) if *w == now => *t,
        _ => {
            *seen = Some((now, time));
            return None;
        }
    };
    (time - since >= SETTLE && wants_window(a, now)).then_some(AppearanceMsg::Window(now))
}
