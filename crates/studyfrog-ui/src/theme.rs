//! One look for the whole app: colours, spacing and type sizes, for the dark and the light theme.
//! The palette is data; [`apply`] turns it into egui's style once at start-up, and views read the
//! colours they need through [`palette`].

use eframe::egui::{self, Color32, CornerRadius, Id, Stroke, Visuals, style::WidgetVisuals};
use studyfrog_core::appearance::Accent;

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
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// What of the user's appearance settings the palette depends on; kept in egui's memory by [`crate::theme_apply`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Look {
    /// The accent colour.
    pub accent: Accent,
    /// Text and borders at a higher contrast.
    pub high_contrast: bool,
}

/// The accent colour and the text colour on it, for the dark or the light theme.
fn accent_pair(accent: Accent, dark: bool) -> (Color32, Color32) {
    let (d, l) = match accent {
        Accent::Blue => (rgb(110, 160, 245), rgb(47, 111, 224)),
        Accent::Green => (rgb(88, 196, 132), rgb(28, 128, 74)),
        Accent::Purple => (rgb(184, 146, 247), rgb(111, 66, 193)),
        Accent::Orange => (rgb(240, 152, 72), rgb(174, 83, 6)),
        Accent::Pink => (rgb(241, 128, 182), rgb(186, 44, 116)),
        Accent::Teal => (rgb(72, 192, 192), rgb(12, 118, 128)),
    };
    if dark {
        (d, rgb(12, 20, 38))
    } else {
        (l, rgb(255, 255, 255))
    }
}

fn base(dark: bool, high_contrast: bool) -> Palette {
    match (dark, high_contrast) {
        (true, false) => Palette {
            background: rgb(23, 24, 27),
            panel: rgb(29, 30, 34),
            surface: rgb(37, 38, 43),
            surface_hover: rgb(46, 48, 54),
            border: rgb(55, 57, 65),
            text: rgb(230, 231, 234),
            text_weak: rgb(154, 156, 165),
            accent: rgb(110, 160, 245),
            on_accent: rgb(12, 20, 38),
            success: rgb(95, 191, 122),
            warning: rgb(224, 168, 74),
            danger: rgb(224, 108, 108),
        },
        (true, true) => Palette {
            background: rgb(0, 0, 0),
            panel: rgb(10, 10, 12),
            surface: rgb(22, 22, 26),
            surface_hover: rgb(44, 44, 52),
            border: rgb(160, 160, 170),
            text: rgb(255, 255, 255),
            text_weak: rgb(214, 214, 220),
            accent: rgb(110, 160, 245),
            on_accent: rgb(0, 0, 0),
            success: rgb(120, 220, 150),
            warning: rgb(255, 196, 90),
            danger: rgb(255, 130, 130),
        },
        (false, false) => Palette {
            background: rgb(244, 245, 247),
            panel: rgb(255, 255, 255),
            surface: rgb(255, 255, 255),
            surface_hover: rgb(238, 240, 243),
            border: rgb(216, 219, 225),
            text: rgb(29, 31, 36),
            text_weak: rgb(99, 104, 116),
            accent: rgb(47, 111, 224),
            on_accent: rgb(255, 255, 255),
            success: rgb(46, 139, 87),
            warning: rgb(183, 121, 31),
            danger: rgb(192, 57, 43),
        },
        (false, true) => Palette {
            background: rgb(255, 255, 255),
            panel: rgb(255, 255, 255),
            surface: rgb(246, 246, 248),
            surface_hover: rgb(226, 228, 233),
            border: rgb(70, 72, 80),
            text: rgb(0, 0, 0),
            text_weak: rgb(52, 54, 62),
            accent: rgb(0, 80, 200),
            on_accent: rgb(255, 255, 255),
            success: rgb(10, 100, 55),
            warning: rgb(140, 84, 0),
            danger: rgb(160, 20, 10),
        },
    }
}

/// The palette for a theme with the user's accent colour and contrast.
pub fn palette_with(dark: bool, look: Look) -> Palette {
    let (accent, on_accent) = accent_pair(look.accent, dark);
    let p = base(dark, look.high_contrast);
    if look.high_contrast && look.accent == Accent::Blue {
        return p;
    }
    Palette {
        accent,
        on_accent,
        ..p
    }
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
