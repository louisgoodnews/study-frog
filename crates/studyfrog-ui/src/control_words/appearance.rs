//! The words of `appearance ...`: `appearance theme dark`, `appearance density compact`, `appearance text
//! 110`, `appearance accent teal`. Each changes one setting, and the setting is saved.

use studyfrog_core::appearance::{
    TEXT_RANGE, accent_from_name, density_from_name, theme_from_name,
};

use crate::appearance_ui::AppearanceMsg;

/// The message for an `appearance <setting> <value>` line; an unknown setting or value is an error that
/// names the choices.
pub fn parse(rest: &[String]) -> Result<AppearanceMsg, String> {
    let text = rest.join(" ");
    let mut words = text.split_whitespace();
    let key = words
        .next()
        .ok_or("appearance needs a setting: theme, density, text or accent")?;
    let value = words
        .next()
        .ok_or_else(|| format!("appearance {key} needs a value"))?;
    match key {
        "theme" => theme_from_name(value)
            .map(AppearanceMsg::Theme)
            .ok_or_else(|| format!("unknown theme {value} (system, dark, light)")),
        "density" => density_from_name(value)
            .map(AppearanceMsg::Density)
            .ok_or_else(|| format!("unknown density {value} (compact, comfortable, spacious)")),
        "text" => {
            let percent: u16 = value
                .parse()
                .map_err(|_| format!("text size {value} is not a number of percent"))?;
            if TEXT_RANGE.contains(&percent) {
                Ok(AppearanceMsg::Text(percent))
            } else {
                Err(format!(
                    "text size must be between {} and {} percent",
                    TEXT_RANGE.start(),
                    TEXT_RANGE.end()
                ))
            }
        }
        "accent" => accent_from_name(value)
            .map(AppearanceMsg::Accent)
            .ok_or_else(|| {
                format!("unknown accent {value} (blue, green, purple, orange, pink, teal)")
            }),
        other => Err(format!(
            "unknown appearance setting {other} (theme, density, text, accent)"
        )),
    }
}
