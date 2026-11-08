//! One look for the whole app: colours, spacing and type sizes, for the dark and the light theme.
//! The palette is OKLCH data (`theme_tokens`); [`apply`] turns it into egui's style once at start-up, and views read the
//! colours they need through [`palette`].

use eframe::egui::{self, Color32, CornerRadius, Id, Stroke, Visuals, style::WidgetVisuals};
use std::sync::OnceLock;

use studyfrog_core::appearance::{ACCENT_LIST, Accent};

/// The colours of one theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// The window background.
    pub background: Color32,
    /// Panels (top bar, sidebar, status bar).
    pub panel: Color32,
    /// Cards and popups.
    pub surface: Color32,
    /// A surface under the pointer.
    pub surface_hover: Color32,
    /// Outlines.
    pub border: Color32,
    /// Normal text.
    pub text: Color32,
    /// Secondary text.
    pub text_weak: Color32,
    /// The main colour: primary buttons, selection, progress.
    pub accent: Color32,
    /// Text on an accent fill.
    pub on_accent: Color32,
    /// Good / done / correct.
    pub success: Color32,
    /// Careful / due soon.
    pub warning: Color32,
    /// Wrong / overdue / error.
    pub danger: Color32,
    /// A fourth colour that is not the accent, for kinds that need one (appointments, notes in the graph).
    pub alt: Color32,
}

/// What of the user's appearance settings the palette depends on; kept in egui's memory by [`crate::theme_apply`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Look {
    /// The accent colour.
    pub accent: Accent,
    /// Text and borders at a higher contrast.
    pub high_contrast: bool,
}

/// Every palette (2 themes x 2 contrasts x 6 accents), computed once from the OKLCH tokens.
fn palettes() -> &'static [Palette] {
    static ALL: OnceLock<Vec<Palette>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut all = Vec::with_capacity(24);
        for dark in [true, false] {
            for high_contrast in [false, true] {
                for accent in ACCENT_LIST {
                    all.push(crate::theme_tokens::build(dark, high_contrast, accent));
                }
            }
        }
        all
    })
}

/// The palette for a theme with the user's accent colour and contrast.
pub fn palette_with(dark: bool, look: Look) -> Palette {
    let a = ACCENT_LIST
        .iter()
        .position(|x| *x == look.accent)
        .unwrap_or(0);
    let i = (usize::from(!dark) * 2 + usize::from(look.high_contrast)) * ACCENT_LIST.len() + a;
    palettes()[i]
}

/// The palette for the dark or the light theme with the default accent.
pub fn palette(dark: bool) -> Palette {
    palette_with(dark, Look::default())
}

/// The data is keyed by its type as well, so the null id is enough.
fn look_id() -> Id {
    Id::NULL
}

/// Remember the look for [`palette_of`].
pub(crate) fn set_look(ctx: &egui::Context, look: Look) {
    ctx.data_mut(|d| d.insert_temp(look_id(), look));
}

/// The look in use.
pub fn look_of(ctx: &egui::Context) -> Look {
    ctx.data(|d| d.get_temp::<Look>(look_id()))
        .unwrap_or_default()
}

/// The palette of the theme the given `ui` is drawn in, with the user's accent colour and contrast.
pub fn palette_of(ui: &egui::Ui) -> Palette {
    palette_with(ui.visuals().dark_mode, look_of(ui.ctx()))
}

/// A colour with some of its opacity, for tinted backgrounds.
pub fn tint(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// Spacing tokens at the Comfortable density (STORY-0115): label to value, icon to text.
pub const GAP_TIGHT: f32 = 4.0;
/// Between sibling controls in a group.
pub const GAP_ITEM: f32 = 8.0;
/// Inside a card.
pub const PAD_CARD: f32 = 16.0;
/// Between distinct groups.
pub const GAP_SECTION: f32 = 24.0;

const RADIUS: u8 = 8;

fn widget(fill: Color32, edge: Color32, text: Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, edge),
        corner_radius: CornerRadius::same(RADIUS),
        fg_stroke: Stroke::new(1.0, text),
        expansion: 0.0,
    }
}

/// egui's visuals for a theme and a look.
pub(crate) fn visuals(dark: bool, look: Look) -> Visuals {
    let p = palette_with(dark, look);
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.panel_fill = p.panel;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(10);
    v.extreme_bg_color = p.background;
    v.faint_bg_color = tint(p.text, 8);
    v.code_bg_color = p.background;
    v.hyperlink_color = p.accent;
    v.error_fg_color = p.danger;
    v.warn_fg_color = p.warning;
    v.selection.bg_fill = tint(p.accent, 90);
    v.selection.stroke = Stroke::new(1.0, p.accent);
    let w = &mut v.widgets;
    w.noninteractive = widget(p.panel, p.border, p.text);
    w.inactive = widget(p.surface_hover, p.border, p.text);
    w.hovered = widget(p.surface_hover, p.accent, p.text);
    w.active = widget(tint(p.accent, 70), p.accent, p.text);
    w.open = widget(p.surface_hover, p.border, p.text);
    v
}
