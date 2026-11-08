//! Spatial rhythm (STORY-0115, `docs/specs/ui-colour-and-type.md`): four spacing tokens that scale with the user's
//! density, and the count of high-salience (accent-filled) buttons drawn in a frame, for the one-primary-action
//! rule. Gestalt proximity: label to value < siblings < inside a card < between groups.

use eframe::egui::{self, Id};
use studyfrog_core::appearance::Density;

use crate::theme::{GAP_ITEM, GAP_SECTION, GAP_TIGHT, PAD_CARD};

/// The spacing tokens at one density, in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rhythm {
    /// Label to value, icon to text.
    pub tight: f32,
    /// Between sibling controls in a group.
    pub item: f32,
    /// Inside a card.
    pub card: f32,
    /// Between distinct groups.
    pub section: f32,
}

/// How a density scales the tokens.
pub fn density_scale(d: Density) -> f32 {
    match d {
        Density::Compact => 0.75,
        Density::Comfortable => 1.0,
        Density::Spacious => 1.25,
    }
}

/// The tokens at a density.
pub fn rhythm(d: Density) -> Rhythm {
    let k = density_scale(d);
    Rhythm {
        tight: GAP_TIGHT * k,
        item: GAP_ITEM * k,
        card: PAD_CARD * k,
        section: GAP_SECTION * k,
    }
}

/// The density in use, kept in egui's memory (keyed by its type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct DensityInUse(Density);

/// Remember the density for [`rhythm_of`].
pub fn set_density(ctx: &egui::Context, d: Density) {
    ctx.data_mut(|m| m.insert_temp(Id::NULL, DensityInUse(d)));
}

/// The tokens at the density in use.
pub fn rhythm_of(ui: &egui::Ui) -> Rhythm {
    let d = ui
        .ctx()
        .data(|m| m.get_temp::<DensityInUse>(Id::NULL))
        .unwrap_or_default();
    rhythm(d.0)
}

/// Accent-filled buttons drawn in a pass: (pass number, count).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Primaries(u64, usize);

/// Count one accent-filled button in this pass.
pub fn note_primary(ctx: &egui::Context) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|m| {
        let p = m.get_temp_mut_or_default::<Primaries>(Id::NULL);
        *p = if p.0 == pass {
            Primaries(pass, p.1 + 1)
        } else {
            Primaries(pass, 1)
        };
    });
}

/// How many accent-filled buttons the last pass that drew any drew (`(pass, count)`).
pub fn primaries(ctx: &egui::Context) -> (u64, usize) {
    let p = ctx
        .data(|m| m.get_temp::<Primaries>(Id::NULL))
        .unwrap_or_default();
    (p.0, p.1)
}

/// Nesting of cards in the current pass, so a card inside a card is drawn as an inset (keyed by its type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CardDepth(usize);

/// Run `add` one card deeper; returns the depth it ran at (0 for a card on the page).
pub fn in_card<R>(ctx: &egui::Context, add: impl FnOnce(usize) -> R) -> R {
    let depth = ctx
        .data(|m| m.get_temp::<CardDepth>(Id::NULL))
        .unwrap_or_default()
        .0;
    ctx.data_mut(|m| m.insert_temp(Id::NULL, CardDepth(depth + 1)));
    let r = add(depth);
    ctx.data_mut(|m| m.insert_temp(Id::NULL, CardDepth(depth)));
    r
}
