//! Putting the user's appearance settings into effect: egui's theme, zoom, text sizes, spacing and animation. [`sync`]
//! runs every frame and does nothing until a setting that matters has changed, so a change in the settings shows at
//! once and no restart is needed.

use eframe::egui::{self, FontId, TextStyle, Theme, ThemePreference};
use studyfrog_core::appearance::{Appearance, Density, ThemeChoice};

use crate::theme::{Look, set_look, visuals};
use crate::typo::{self, Tier};

/// The settings that change how things look (the others, such as the start screen, change nothing on screen).
#[derive(Debug, Clone, PartialEq)]
struct LookKey(Appearance);

fn key_of(a: &Appearance) -> LookKey {
    LookKey(Appearance {
        start: Default::default(),
        last_screen: None,
        sidebar_folded: false,
        zen_in_runs: false,
        remember_window: false,
        window: None,
        ..a.clone()
    })
}

/// Spacing for a density: item spacing (the rhythm's sibling gap, `theme_space`), button padding and the height of
/// a control.
fn spacing_of(d: Density) -> (egui::Vec2, egui::Vec2, f32) {
    let item = crate::theme_space::rhythm(d).item;
    match d {
        Density::Compact => (egui::vec2(item, item), egui::vec2(10.0, 4.0), 26.0),
        Density::Comfortable => (egui::vec2(item, item), egui::vec2(12.0, 6.0), 30.0),
        Density::Spacious => (egui::vec2(item, item), egui::vec2(14.0, 8.0), 34.0),
    }
}

fn preference(t: ThemeChoice) -> ThemePreference {
    match t {
        ThemeChoice::System => ThemePreference::System,
        ThemeChoice::Dark => ThemePreference::Dark,
        ThemeChoice::Light => ThemePreference::Light,
    }
}

/// Install the look for `a`: colours for both themes, spacing, text sizes, animation, the zoom and the theme.
pub fn apply_look(ctx: &egui::Context, a: &Appearance) {
    apply_with(ctx, a, false);
}

/// `motion_back`: reduced motion was on and is now off, so the animations return.
fn apply_with(ctx: &egui::Context, a: &Appearance, motion_back: bool) {
    let look = Look {
        accent: a.accent,
        high_contrast: a.high_contrast,
    };
    set_look(ctx, look);
    crate::theme_space::set_density(ctx, a.density);
    let (item, button, height) = spacing_of(a.density);
    let scale = f32::from(a.text) / 100.0;
    for (theme, dark) in [(Theme::Dark, true), (Theme::Light, false)] {
        ctx.set_visuals_of(theme, visuals(dark, look));
        ctx.style_mut_of(theme, |s| {
            s.spacing.item_spacing = item;
            s.spacing.button_padding = button;
            s.spacing.interact_size.y = height;
            s.spacing.indent = 18.0;
            // A floating scroll bar reserves its own width, so content never runs under it.
            s.spacing.scroll.floating_allocated_width = 12.0;
            s.spacing.scroll.bar_outer_margin = 4.0;
            s.spacing.window_margin = egui::Margin::same(16);
            // Only a change of the setting touches the animation time: egui's own (or a test's) stays otherwise.
            if a.reduced_motion {
                s.animation_time = 0.0;
            } else if motion_back {
                s.animation_time = 1.0 / 12.0;
            }
            s.text_styles = text_styles(scale);
        });
    }
    ctx.set_theme(preference(a.theme));
    ctx.set_zoom_factor(f32::from(a.zoom) / 100.0);
    // Zoom is a setting of ours (saved, with its own keys), not egui's own.
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
}

/// The four tiers of `typo` as egui's text styles; code is set at the label size.
fn text_styles(scale: f32) -> std::collections::BTreeMap<TextStyle, FontId> {
    let body = typo::size_for(typo::BASE * scale, Tier::Body);
    let at = |t| FontId::proportional(typo::size_for(body, t));
    [
        (TextStyle::Heading, at(Tier::Heading)),
        (TextStyle::Body, at(Tier::Body)),
        (TextStyle::Button, at(Tier::Label)),
        (TextStyle::Small, at(Tier::Caption)),
        (
            TextStyle::Monospace,
            FontId::monospace(typo::size_for(body, Tier::Label)),
        ),
    ]
    .into()
}

/// The default look, installed at start-up before the saved settings are known.
pub fn apply(ctx: &egui::Context) {
    apply_look(ctx, &Appearance::default());
}

fn applied_id() -> egui::Id {
    egui::Id::NULL
}

/// Install `a` as the look now (at start-up) and remember that it is installed.
pub fn install(ctx: &egui::Context, a: &Appearance) {
    apply_with(ctx, a, false);
    ctx.data_mut(|d| d.insert_temp(applied_id(), key_of(a)));
}

/// Apply `a` when a setting that matters differs from what is installed. Nothing is touched while no look was
/// installed and the settings are the defaults, so a context that never had one (a test's) keeps egui's own.
pub fn sync(ctx: &egui::Context, a: &Appearance) {
    let key = key_of(a);
    let before = ctx.data(|d| d.get_temp::<LookKey>(applied_id()));
    let untouched = before.is_none() && key == key_of(&Appearance::default());
    if untouched || before.as_ref() == Some(&key) {
        return;
    }
    let motion_back = before.is_some_and(|b| b.0.reduced_motion) && !a.reduced_motion;
    apply_with(ctx, a, motion_back);
    ctx.data_mut(|d| d.insert_temp(applied_id(), key));
}
