//! The appearance and start-up settings in the state: changing them (saved at once, shown at once by
//! [`crate::theme_apply::sync`]), the start screen, the last screen and the window. Pure.

use studyfrog_core::appearance::{
    Accent, Appearance, Density, StartScreen, TEXT_SIZES, ThemeChoice, WindowState, ZOOMS,
    appearance_default, step_of,
};

use crate::effect::Effect;
use crate::state::{Screen, State};

/// What the user did in the appearance settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppearanceMsg {
    /// Dark, light or the system's theme.
    Theme(ThemeChoice),
    /// The interface size in percent.
    Zoom(u16),
    /// One step larger (`1`) or smaller (`-1`).
    ZoomStep(i8),
    /// Back to 100 percent.
    ZoomReset,
    /// The text size in percent.
    Text(u16),
    /// The density.
    Density(Density),
    /// The accent colour.
    Accent(Accent),
    /// Higher contrast on or off.
    HighContrast(bool),
    /// Animations off or on.
    ReducedMotion(bool),
    /// The screen to open first.
    Start(StartScreen),
    /// The sidebar starts folded or shown.
    SidebarFolded(bool),
    /// Zen mode by itself during runs, or not.
    ZenInRuns(bool),
    /// Remember the window's size and place, or not.
    RememberWindow(bool),
    /// The window is now like this (from the shell, once it has stayed so for a moment).
    Window(WindowState),
    /// Every setting back to its default.
    Defaults,
}

/// Screens that can be opened first and are remembered as the last one, with their words in `app.toml`.
const SCREENS: [(Screen, &str); 13] = [
    (Screen::Home, "home"),
    (Screen::Library, "library"),
    (Screen::Calendar, "calendar"),
    (Screen::Board, "board"),
    (Screen::Achievements, "achievements"),
    (Screen::Reports, "reports"),
    (Screen::Diary, "diary"),
    (Screen::Agent, "assistant"),
    (Screen::Prompts, "prompts"),
    (Screen::Graph, "graph"),
    (Screen::Rehearse, "rehearse"),
    (Screen::Settings, "settings"),
    (Screen::Help, "help"),
];

fn screen_key(s: Screen) -> Option<&'static str> {
    SCREENS.iter().find(|(x, _)| *x == s).map(|(_, k)| *k)
}

/// The screen a vault opens on, by the setting.
pub fn start_screen(a: &Appearance) -> Screen {
    match a.start {
        StartScreen::Home => Screen::Home,
        StartScreen::Library => Screen::Library,
        StartScreen::Last => a
            .last_screen
            .as_deref()
            .and_then(|k| SCREENS.iter().find(|(_, key)| *key == k))
            .map_or(Screen::Home, |(s, _)| *s),
    }
}

/// Go to the start screen after a vault opened (it opens on Home, so only another one needs a step).
pub fn go_start(state: State) -> (State, Vec<Effect>) {
    match start_screen(&state.app.appearance) {
        Screen::Home => (state, Vec::new()),
        other => crate::nav::navigate(state, other),
    }
}

/// The sidebar's state at start-up, from the saved setting.
pub fn on_start(state: State) -> State {
    let mut state = state;
    state.chrome.sidebar_hidden = state.app.appearance.sidebar_folded;
    state
}

/// Keep the last screen for the next start when that is what the user asked for.
pub fn remember_screen(state: State) -> (State, Vec<Effect>) {
    let a = &state.app.appearance;
    let key =
        screen_key(state.screen).filter(|_| a.start == StartScreen::Last && state.vault.is_some());
    match key {
        Some(k) if a.last_screen.as_deref() != Some(k) => {
            let mut state = state;
            state.app.appearance.last_screen = Some(k.to_owned());
            let save = Effect::SaveAppSettings(state.app.clone());
            (state, vec![save])
        }
        _ => (state, Vec::new()),
    }
}

/// Whether `now` is a window worth saving: remembering is on and it differs from what is saved.
pub fn wants_window(a: &Appearance, now: WindowState) -> bool {
    a.remember_window && Some(window_after(a.window, now)) != a.window
}

fn window_after(old: Option<WindowState>, new: WindowState) -> WindowState {
    // A maximised window's size is the screen's: keep the size and place the window had before.
    match (new.maximized, old) {
        (true, Some(before)) => WindowState {
            maximized: true,
            ..before
        },
        _ => new,
    }
}

fn changed(a: &mut Appearance, msg: AppearanceMsg) {
    match msg {
        AppearanceMsg::Theme(t) => a.theme = t,
        AppearanceMsg::Zoom(z) => a.zoom = z,
        AppearanceMsg::ZoomStep(by) => a.zoom = step_of(&ZOOMS, a.zoom, by),
        AppearanceMsg::ZoomReset => a.zoom = 100,
        AppearanceMsg::Text(t) => a.text = step_of(&TEXT_SIZES, t, 0),
        AppearanceMsg::Density(d) => a.density = d,
        AppearanceMsg::Accent(c) => a.accent = c,
        AppearanceMsg::HighContrast(on) => a.high_contrast = on,
        AppearanceMsg::ReducedMotion(on) => a.reduced_motion = on,
        AppearanceMsg::Start(s) => a.start = s,
        AppearanceMsg::SidebarFolded(on) => a.sidebar_folded = on,
        AppearanceMsg::ZenInRuns(on) => a.zen_in_runs = on,
        AppearanceMsg::RememberWindow(on) => a.remember_window = on,
        AppearanceMsg::Window(w) => {
            if a.remember_window {
                a.window = Some(window_after(a.window, w));
            }
        }
        AppearanceMsg::Defaults => {
            let keep = (a.window, a.last_screen.clone());
            *a = appearance_default();
            (a.window, a.last_screen) = keep;
        }
    }
}

/// Handle a change of a setting: it is stored and saved at once.
pub fn update(state: State, msg: AppearanceMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    let before = state.app.appearance.clone();
    changed(&mut state.app.appearance, msg);
    if state.app.appearance == before {
        return (state, Vec::new());
    }
    let save = Effect::SaveAppSettings(state.app.clone());
    (state, vec![save])
}
