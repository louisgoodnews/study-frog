//! How an item's key and colour show (STORY-0096, STORY-0097, STORY-0123): the stack a typed key
//! names, and the tint a colour gives a surface without making its text harder to read. Pure.

use eframe::egui::Color32;
use studyfrog_core::item_color::color_rgb;
use studyfrog_core::item_key::normalize_key;
use studyfrog_store::overview::StackRow;

use crate::color_science::apca_abs;
use crate::state::State;
use crate::theme::Palette;

/// The stack whose key is the typed text (`stack_3` finds `STACK_3`).
pub fn stack_by_key<'a>(state: &'a State, text: &str) -> Option<&'a StackRow> {
    let key = normalize_key(text)?;
    let vault = state.vault.as_ref()?;
    vault
        .overview
        .stacks
        .iter()
        .find(|s| s.key.as_deref() == Some(key.as_str()))
}

/// The share of the item's colour mixed into a surface: a little more on a dark surface, where a
/// tint is harder to see (the story asks for 10 to 14 % dark, 8 to 12 % light).
pub fn tint_share(dark: bool) -> f32 {
    if dark { 0.14 } else { 0.10 }
}

/// The contrast (WCAG 2) a tint must leave text: the 4.5 normal text needs.
pub const MIN_CONTRAST: f32 = 4.5;

fn linear(c: u8) -> f32 {
    let v = f32::from(c) / 255.0;
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// The relative luminance of a colour (WCAG 2).
pub fn luminance(c: Color32) -> f32 {
    0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
}

/// The contrast ratio of two colours, 1 to 21 (WCAG 2).
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

fn mix(surface: Color32, [r, g, b]: [u8; 3], share: f32) -> Color32 {
    crate::color_science::mix_oklab(surface, Color32::from_rgb(r, g, b), share)
}

/// The APCA contrast (|Lc|) a tint must leave the first text colour (body text) and the others (secondary
/// text), unless the plain surface already has less: the targets of `docs/specs/ui-colour-and-type.md`.
pub const MIN_APCA: [f32; 2] = [75.0, 58.0];

fn apca_kept(out: Color32, surface: Color32, i: usize, t: Color32) -> bool {
    let floor = MIN_APCA[i.min(1)];
    let (now, before) = (apca_abs(t, out), apca_abs(t, surface));
    now >= floor.min(before)
}

/// Whether every text colour keeps its contrast with `out`: at least [`MIN_CONTRAST`] (WCAG 2) and
/// [`MIN_APCA`] (APCA), or what it has with the plain surface if that is already less.
fn readable(out: Color32, surface: Color32, texts: &[Color32]) -> bool {
    texts.iter().enumerate().all(|(i, t)| {
        contrast(out, *t) >= MIN_CONTRAST.min(contrast(surface, *t)) - 0.005
            && apca_kept(out, surface, i, *t)
    })
}

/// The surface with the colour mixed in at the theme's share, less if that would make any of the
/// text colours harder to read than [`readable`] allows.
pub fn tinted(surface: Color32, texts: &[Color32], rgb: [u8; 3], dark: bool) -> Color32 {
    let mut share = tint_share(dark);
    while share > 0.0 {
        let out = mix(surface, rgb, share);
        if readable(out, surface, texts) {
            return out;
        }
        share -= 0.01;
    }
    surface
}

/// The tinted surface for an item's stored colour value; `None` when it names no colour.
pub fn item_surface(
    surface: Color32,
    texts: &[Color32],
    color: Option<&str>,
    dark: bool,
) -> Option<Color32> {
    let rgb = color_rgb(color?)?;
    Some(tinted(surface, texts, rgb, dark))
}

/// The colour itself, for a swatch or a dot.
pub fn swatch(color: &str) -> Option<Color32> {
    color_rgb(color).map(|[r, g, b]| Color32::from_rgb(r, g, b))
}

/// How many stacks and items have no key yet, as the library knows them.
pub fn keyless_count(state: &State) -> usize {
    let items = state
        .library
        .items
        .iter()
        .filter(|i| i.row.key.is_none())
        .count();
    let stacks = state.vault.as_ref().map_or(0, |v| {
        v.overview.stacks.iter().filter(|s| s.key.is_none()).count()
    });
    items + stacks
}

/// The fill and border of a card that shows an item: the surface tinted by the item's colour and a
/// border halfway to the colour; the plain surface and border for an item without one.
pub fn card_colors(p: &Palette, dark: bool, color: Option<&str>) -> (Color32, Color32) {
    let Some(rgb) = color.and_then(color_rgb) else {
        return (p.surface, p.border);
    };
    (
        tinted(p.surface, &[p.text, p.text_weak], rgb, dark),
        mix(p.border, rgb, 0.5),
    )
}

/// The window background tinted by the item's colour, for the page around an item; `None` without one.
pub fn page_fill(p: &Palette, dark: bool, color: Option<&str>) -> Option<Color32> {
    item_surface(p.background, &[p.text, p.text_weak], color, dark)
}

/// The colour of the item a run shows now (prompt or answer); `None` while paused or without one.
pub fn run_color(run: &crate::run_state::RunView) -> Option<&str> {
    if run.session.run.state == studyfrog_core::run::RunState::Paused {
        return None;
    }
    let uid = studyfrog_rehearsal::session::current_item(&run.session)?;
    run.content.get(&uid)?.color.as_deref()
}
