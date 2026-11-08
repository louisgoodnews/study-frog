//! The `[appearance]` table of `app.toml`: theme, size, density, colours, motion, start-up and window. Only what differs
//! from the defaults is written.

use serde::Deserialize;
use studyfrog_core::appearance::{
    Appearance, TEXT_RANGE, WindowState, ZOOM_RANGE, accent_from_name, accent_name,
    appearance_default, density_from_name, density_name, start_from_name, start_name,
    theme_from_name, theme_name,
};

use crate::error::{ConfigError, config_error, line_of_key};
use crate::values::toml_string;

/// The keys of the table.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawAppearance {
    theme: Option<String>,
    zoom: Option<u16>,
    text_size: Option<u16>,
    density: Option<String>,
    accent: Option<String>,
    high_contrast: Option<bool>,
    reduced_motion: Option<bool>,
    start_screen: Option<String>,
    last_screen: Option<String>,
    sidebar_folded: Option<bool>,
    zen_in_runs: Option<bool>,
    remember_window: Option<bool>,
    window_width: Option<u32>,
    window_height: Option<u32>,
    window_x: Option<i32>,
    window_y: Option<i32>,
    window_maximized: Option<bool>,
}

fn word<T>(
    text: &str,
    key: &str,
    value: Option<&str>,
    from: fn(&str) -> Option<T>,
    options: &str,
) -> Result<Option<T>, ConfigError> {
    match value {
        None => Ok(None),
        Some(w) => from(w).map(Some).ok_or_else(|| {
            config_error(
                line_of_key(text, key),
                format!("{key} must be one of {options}, not \"{w}\""),
            )
        }),
    }
}

fn range(
    text: &str,
    key: &str,
    value: Option<u16>,
    allowed: std::ops::RangeInclusive<u16>,
) -> Result<Option<u16>, ConfigError> {
    match value {
        Some(v) if !allowed.contains(&v) => Err(config_error(
            line_of_key(text, key),
            format!(
                "{key} must be between {} and {} percent, not {v}",
                allowed.start(),
                allowed.end()
            ),
        )),
        other => Ok(other),
    }
}

fn window_of(raw: &RawAppearance) -> Option<WindowState> {
    let (width, height) = (raw.window_width?, raw.window_height?);
    Some(WindowState {
        width: width.clamp(320, 20_000),
        height: height.clamp(240, 20_000),
        x: raw.window_x,
        y: raw.window_y,
        maximized: raw.window_maximized == Some(true),
    })
}

/// The settings of the table (the defaults where a key is missing).
pub(crate) fn parse(text: &str, raw: Option<RawAppearance>) -> Result<Appearance, ConfigError> {
    let raw = raw.unwrap_or_default();
    let d = appearance_default();
    Ok(Appearance {
        theme: word(
            text,
            "theme",
            raw.theme.as_deref(),
            theme_from_name,
            "system, dark or light",
        )?
        .unwrap_or(d.theme),
        zoom: range(text, "zoom", raw.zoom, ZOOM_RANGE)?.unwrap_or(d.zoom),
        text: range(text, "text_size", raw.text_size, TEXT_RANGE)?.unwrap_or(d.text),
        density: word(
            text,
            "density",
            raw.density.as_deref(),
            density_from_name,
            "compact, comfortable or spacious",
        )?
        .unwrap_or(d.density),
        accent: word(
            text,
            "accent",
            raw.accent.as_deref(),
            accent_from_name,
            "blue, green, purple, orange, pink or teal",
        )?
        .unwrap_or(d.accent),
        high_contrast: raw.high_contrast.unwrap_or(d.high_contrast),
        reduced_motion: raw.reduced_motion.unwrap_or(d.reduced_motion),
        start: word(
            text,
            "start_screen",
            raw.start_screen.as_deref(),
            start_from_name,
            "home, library or last",
        )?
        .unwrap_or(d.start),
        last_screen: raw.last_screen.clone(),
        sidebar_folded: raw.sidebar_folded.unwrap_or(d.sidebar_folded),
        zen_in_runs: raw.zen_in_runs.unwrap_or(d.zen_in_runs),
        remember_window: raw.remember_window.unwrap_or(d.remember_window),
        window: window_of(&raw),
    })
}

/// The table as text, or nothing when every setting is the default.
pub(crate) fn render(a: &Appearance) -> String {
    let d = appearance_default();
    let mut lines: Vec<String> = Vec::new();
    let mut add = |on: bool, line: String| {
        if on {
            lines.push(line);
        }
    };
    add(
        a.theme != d.theme,
        format!("theme = \"{}\"", theme_name(a.theme)),
    );
    add(a.zoom != d.zoom, format!("zoom = {}", a.zoom));
    add(a.text != d.text, format!("text_size = {}", a.text));
    add(
        a.density != d.density,
        format!("density = \"{}\"", density_name(a.density)),
    );
    add(
        a.accent != d.accent,
        format!("accent = \"{}\"", accent_name(a.accent)),
    );
    add(a.high_contrast, "high_contrast = true".to_owned());
    add(a.reduced_motion, "reduced_motion = true".to_owned());
    add(
        a.start != d.start,
        format!("start_screen = \"{}\"", start_name(a.start)),
    );
    if let Some(s) = &a.last_screen {
        add(true, format!("last_screen = {}", toml_string(s)));
    }
    add(a.sidebar_folded, "sidebar_folded = true".to_owned());
    add(a.zen_in_runs, "zen_in_runs = true".to_owned());
    add(!a.remember_window, "remember_window = false".to_owned());
    if let Some(w) = a.window {
        add(true, format!("window_width = {}", w.width));
        add(true, format!("window_height = {}", w.height));
        add(w.x.is_some(), format!("window_x = {}", w.x.unwrap_or(0)));
        add(w.y.is_some(), format!("window_y = {}", w.y.unwrap_or(0)));
        add(w.maximized, "window_maximized = true".to_owned());
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("\n[appearance]\n{}\n", lines.join("\n"))
}
