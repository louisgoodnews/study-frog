//! One look for the whole app: colours, spacing and type sizes, for the dark and the light theme.
//! The palette is data; [`apply`] turns it into egui's style once at start-up, and views read the
//! colours they need through [`palette`].

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Stroke, TextStyle, Theme, Visuals, style::WidgetVisuals,
};

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

/// The palette for the dark or the light theme.
pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
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
        }
    } else {
        Palette {
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
        }
    }
}

/// The palette of the theme the given `ui` is drawn in.
pub fn palette_of(ui: &egui::Ui) -> Palette {
    palette(ui.visuals().dark_mode)
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

fn visuals(dark: bool) -> Visuals {
    let p = palette(dark);
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

/// Install the look: colours for both themes, roomier spacing and clearer type sizes.
pub fn apply(ctx: &egui::Context) {
    for (theme, dark) in [(Theme::Dark, true), (Theme::Light, false)] {
        ctx.set_visuals_of(theme, visuals(dark));
        ctx.style_mut_of(theme, |s| {
            s.spacing.item_spacing = egui::vec2(8.0, 7.0);
            s.spacing.button_padding = egui::vec2(12.0, 6.0);
            s.spacing.interact_size.y = 30.0;
            s.spacing.indent = 18.0;
            s.spacing.window_margin = egui::Margin::same(16);
            s.text_styles = [
                (TextStyle::Heading, FontId::proportional(22.0)),
                (TextStyle::Body, FontId::proportional(15.0)),
                (TextStyle::Button, FontId::proportional(15.0)),
                (TextStyle::Small, FontId::proportional(12.5)),
                (TextStyle::Monospace, FontId::monospace(14.0)),
            ]
            .into();
        });
    }
}
