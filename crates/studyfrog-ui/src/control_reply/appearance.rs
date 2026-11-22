//! The answer of `appearance ...`: the appearance as it is now, with the spacing the density produces.

use serde_json::{Value, json};
use studyfrog_core::appearance::{accent_name, density_name, theme_name};

use crate::state::State;
use crate::theme_space::density_scale;

/// The appearance settings, and the spacing factor the density gives.
pub fn appearance_json(state: &State) -> Value {
    let a = &state.app.appearance;
    json!({
        "theme": theme_name(a.theme),
        "density": density_name(a.density),
        "spacing": density_scale(a.density),
        "text_percent": a.text,
        "accent": accent_name(a.accent),
    })
}
