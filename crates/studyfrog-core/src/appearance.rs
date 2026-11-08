//! How the application looks and starts: theme, size, density, colours, motion, the start screen and the window.
//! Plain data kept per user in `app.toml`; the names are the words written there.

/// Which theme to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeChoice {
    /// The system's, followed while the application runs.
    #[default]
    System,
    /// Dark.
    Dark,
    /// Light.
    Light,
}

/// How much room controls and lists take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    /// Tight.
    Compact,
    /// The default.
    #[default]
    Comfortable,
    /// Roomy.
    Spacious,
}

/// The colour of selections, links and the main button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Accent {
    /// Blue (the default).
    #[default]
    Blue,
    /// Green.
    Green,
    /// Purple.
    Purple,
    /// Orange.
    Orange,
    /// Pink.
    Pink,
    /// Teal.
    Teal,
}

/// Which screen opens when a vault opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StartScreen {
    /// Home.
    #[default]
    Home,
    /// The library.
    Library,
    /// The screen that was open last.
    Last,
}

/// Where the window was: size, place (not every system tells it) and whether it was maximised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowState {
    /// Inner width in points.
    pub width: u32,
    /// Inner height in points.
    pub height: u32,
    /// Left edge on the desktop, when known.
    pub x: Option<i32>,
    /// Top edge on the desktop, when known.
    pub y: Option<i32>,
    /// Maximised.
    pub maximized: bool,
}

/// Every appearance and start-up setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Appearance {
    /// Dark, light or the system's.
    pub theme: ThemeChoice,
    /// Interface size in percent.
    pub zoom: u16,
    /// Text size in percent, on top of the interface size.
    pub text: u16,
    /// Room around things.
    pub density: Density,
    /// The accent colour.
    pub accent: Accent,
    /// Text and borders at a higher contrast.
    pub high_contrast: bool,
    /// No animations.
    pub reduced_motion: bool,
    /// The screen to open first.
    pub start: StartScreen,
    /// The screen open last (a stable word such as `calendar`), used by [`StartScreen::Last`].
    pub last_screen: Option<String>,
    /// The sidebar starts folded away.
    pub sidebar_folded: bool,
    /// Zen mode (no sidebar, no status bar) comes on by itself during a rehearsal or exam run.
    pub zen_in_runs: bool,
    /// Remember the window's size and place.
    pub remember_window: bool,
    /// The remembered window.
    pub window: Option<WindowState>,
}

/// Interface sizes offered, in percent.
pub const ZOOMS: [u16; 9] = [70, 80, 90, 100, 110, 125, 150, 175, 200];
/// Text sizes offered, in percent.
pub const TEXT_SIZES: [u16; 6] = [85, 90, 100, 110, 125, 150];
/// What the config file accepts for the sizes, in percent.
pub const ZOOM_RANGE: std::ops::RangeInclusive<u16> = 50..=300;
/// What the config file accepts for the text size, in percent.
pub const TEXT_RANGE: std::ops::RangeInclusive<u16> = 70..=200;

/// The defaults: the system's theme, 100 percent, comfortable, blue, Home.
pub fn appearance_default() -> Appearance {
    Appearance {
        theme: ThemeChoice::default(),
        zoom: 100,
        text: 100,
        density: Density::default(),
        accent: Accent::default(),
        high_contrast: false,
        reduced_motion: false,
        start: StartScreen::default(),
        last_screen: None,
        sidebar_folded: false,
        zen_in_runs: false,
        remember_window: true,
        window: None,
    }
}

impl Default for Appearance {
    fn default() -> Self {
        appearance_default()
    }
}

/// The step before (`-1`) or after (`1`) `current` in `steps`; stays at the ends and snaps to the nearest step
/// when `current` is not one of them.
pub fn step_of(steps: &[u16], current: u16, by: i8) -> u16 {
    let nearest = steps
        .iter()
        .enumerate()
        .min_by_key(|(_, s)| s.abs_diff(current))
        .map_or(0, |(i, _)| i);
    let at = nearest
        .saturating_add_signed(isize::from(by))
        .min(steps.len() - 1);
    steps[at]
}

fn pick<T: Copy>(names: &[(T, &str)], name: &str) -> Option<T> {
    names.iter().find(|(_, n)| *n == name).map(|(v, _)| *v)
}

fn word<T: Copy + PartialEq>(names: &[(T, &'static str)], value: T) -> &'static str {
    names
        .iter()
        .find(|(v, _)| *v == value)
        .map_or("", |(_, n)| n)
}

const THEMES: [(ThemeChoice, &str); 3] = [
    (ThemeChoice::System, "system"),
    (ThemeChoice::Dark, "dark"),
    (ThemeChoice::Light, "light"),
];
const DENSITIES: [(Density, &str); 3] = [
    (Density::Compact, "compact"),
    (Density::Comfortable, "comfortable"),
    (Density::Spacious, "spacious"),
];
const ACCENTS: [(Accent, &str); 6] = [
    (Accent::Blue, "blue"),
    (Accent::Green, "green"),
    (Accent::Purple, "purple"),
    (Accent::Orange, "orange"),
    (Accent::Pink, "pink"),
    (Accent::Teal, "teal"),
];
const STARTS: [(StartScreen, &str); 3] = [
    (StartScreen::Home, "home"),
    (StartScreen::Library, "library"),
    (StartScreen::Last, "last"),
];

/// Every accent, in the order they are offered.
pub const ACCENT_LIST: [Accent; 6] = [
    Accent::Blue,
    Accent::Green,
    Accent::Purple,
    Accent::Orange,
    Accent::Pink,
    Accent::Teal,
];

/// The word written in `app.toml`.
pub fn theme_name(v: ThemeChoice) -> &'static str {
    word(&THEMES, v)
}
/// The theme for a word.
pub fn theme_from_name(n: &str) -> Option<ThemeChoice> {
    pick(&THEMES, n)
}
/// The word written in `app.toml`.
pub fn density_name(v: Density) -> &'static str {
    word(&DENSITIES, v)
}
/// The density for a word.
pub fn density_from_name(n: &str) -> Option<Density> {
    pick(&DENSITIES, n)
}
/// The word written in `app.toml`.
pub fn accent_name(v: Accent) -> &'static str {
    word(&ACCENTS, v)
}
/// The accent for a word.
pub fn accent_from_name(n: &str) -> Option<Accent> {
    pick(&ACCENTS, n)
}
/// The word written in `app.toml`.
pub fn start_name(v: StartScreen) -> &'static str {
    word(&STARTS, v)
}
/// The start screen for a word.
pub fn start_from_name(n: &str) -> Option<StartScreen> {
    pick(&STARTS, n)
}
