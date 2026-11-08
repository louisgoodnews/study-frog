//! The Appearance tab of the settings: theme, size, density, colours, motion, start-up and the window. Every change is
//! stored, saved and shown at once; the sample at the end shows what the choices make of the interface.

use eframe::egui::{self, Color32, RichText, Role, Sense, Stroke, WidgetInfo, vec2};
use studyfrog_core::appearance::{
    ACCENT_LIST, Accent, Density, StartScreen, TEXT_SIZES, ThemeChoice, ZOOMS, accent_name,
};

use crate::appearance_ui::AppearanceMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::{Look, palette_of, palette_with};
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: AppearanceMsg) {
    out.push(Msg::Appearance(m));
}

/// A row of choices, one selected; the new choice when one was picked. Each is named "`name`: `label`" for screen readers.
pub(super) fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    name: &str,
    current: T,
    options: &[(T, &str)],
) -> Option<T> {
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        for (value, label) in options {
            let r = ui.selectable_label(current == *value, *label);
            r.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("{name}: {label}")));
            if r.clicked() {
                picked = Some(*value);
            }
        }
    });
    picked
}

/// Dark, light or the system's: shared by the settings and the profile menu.
pub(super) fn theme_choice(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let options = [
        (ThemeChoice::Dark, "Dark"),
        (ThemeChoice::Light, "Light"),
        (ThemeChoice::System, "Follow system"),
    ];
    if let Some(t) = choice(ui, "Theme", state.app.appearance.theme, &options) {
        send(out, AppearanceMsg::Theme(t));
    }
}

pub(super) fn theme_section(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::group(ui, "Theme", |ui| {
        theme_choice(state, ui, out);
        ui.weak("Follow system uses the theme of your desktop and changes with it while StudyFrog is open.");
    });
}

fn percent_box(ui: &mut egui::Ui, name: &str, steps: &[u16], current: u16) -> Option<u16> {
    let mut picked = None;
    let r = egui::ComboBox::from_id_salt(name)
        .selected_text(format!("{current} %"))
        .show_ui(ui, |ui| {
            for s in steps {
                if ui
                    .selectable_label(current == *s, format!("{s} %"))
                    .clicked()
                {
                    picked = Some(*s);
                }
            }
        });
    r.response
        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, name));
    picked
}

pub(super) fn size_section(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.app.appearance;
    kit::group(ui, "Size and density", |ui| {
        ui.horizontal(|ui| {
            ui.label("Interface size");
            if let Some(z) = percent_box(ui, "Interface size", &ZOOMS, a.zoom) {
                send(out, AppearanceMsg::Zoom(z));
            }
            if ui.button("Smaller").clicked() {
                send(out, AppearanceMsg::ZoomStep(-1));
            }
            if ui.button("Larger").clicked() {
                send(out, AppearanceMsg::ZoomStep(1));
            }
            if ui.button("100 %").clicked() {
                send(out, AppearanceMsg::ZoomReset);
            }
        });
        ui.weak("Everything gets larger or smaller. Ctrl and plus, Ctrl and minus and Ctrl and 0 do the same anywhere.");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Text size");
            if let Some(t) = percent_box(ui, "Text size", &TEXT_SIZES, a.text) {
                send(out, AppearanceMsg::Text(t));
            }
        });
        ui.weak("Only the text, on top of the interface size.");
        ui.add_space(6.0);
        ui.label("Density");
        let options = [
            (Density::Compact, "Compact"),
            (Density::Comfortable, "Comfortable"),
            (Density::Spacious, "Spacious"),
        ];
        if let Some(d) = choice(ui, "Density", a.density, &options) {
            send(out, AppearanceMsg::Density(d));
        }
    });
}

fn swatch(ui: &mut egui::Ui, accent: Accent, selected: bool) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::click());
    let dark = ui.visuals().dark_mode;
    let fill = palette_with(
        dark,
        Look {
            accent,
            high_contrast: false,
        },
    )
    .accent;
    let ring = if selected {
        ui.visuals().text_color()
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().circle_filled(rect.center(), 11.0, fill);
    ui.painter()
        .circle_stroke(rect.center(), 14.0, Stroke::new(2.0, ring));
    let name = accent_name(accent);
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Accent colour: {name}")));
    r.on_hover_text(name)
}

pub(super) fn colours_section(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.app.appearance;
    kit::group(ui, "Colours and motion", |ui| {
        ui.label("Accent colour");
        ui.horizontal(|ui| {
            for c in ACCENT_LIST {
                if swatch(ui, c, a.accent == c).clicked() {
                    send(out, AppearanceMsg::Accent(c));
                }
            }
        });
        ui.add_space(6.0);
        let mut high = a.high_contrast;
        if ui.checkbox(&mut high, "High contrast").changed() {
            send(out, AppearanceMsg::HighContrast(high));
        }
        ui.weak("Text and borders stand out more.");
        let mut calm = a.reduced_motion;
        if ui.checkbox(&mut calm, "Reduce motion").changed() {
            send(out, AppearanceMsg::ReducedMotion(calm));
        }
        ui.weak("No fading or sliding.");
    });
}

pub(super) fn startup_section(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.app.appearance;
    kit::group(ui, "Start-up and window", |ui| {
        ui.label("Open this screen first");
        let options = [
            (StartScreen::Home, "Home"),
            (StartScreen::Library, "Library"),
            (StartScreen::Last, "The screen I used last"),
        ];
        if let Some(s) = choice(ui, "Start screen", a.start, &options) {
            send(out, AppearanceMsg::Start(s));
        }
        ui.add_space(6.0);
        let mut folded = a.sidebar_folded;
        if ui
            .checkbox(&mut folded, "Start with the sidebar folded away")
            .changed()
        {
            send(out, AppearanceMsg::SidebarFolded(folded));
        }
        let mut remember = a.remember_window;
        if ui
            .checkbox(&mut remember, "Remember the window's size and place")
            .changed()
        {
            send(out, AppearanceMsg::RememberWindow(remember));
        }
        ui.weak(
            "Some desktops do not tell the place of a window; then only its size is remembered.",
        );
    });
}

pub(super) fn sample_section(_: &State, ui: &mut egui::Ui, _: &mut Vec<Msg>) {
    kit::group(ui, "Sample", |ui| {
        let p = palette_of(ui);
        ui.label(
            RichText::new("A heading")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.label("Body text with a link:");
        ui.hyperlink_to("a link", "https://example.org");
        ui.horizontal(|ui| {
            let _ = ui.button("A button");
            let _ = kit::primary_button(ui, "The main action");
            kit::pill(ui, "a status", p.success);
            kit::pill(ui, "a warning", p.warning);
            kit::pill(ui, "an error", p.danger);
        });
        kit::progress_with_text(ui, 0.4, "40 % of the sample", "Sample progress");
        ui.weak("Weak text for hints.");
    });
}

pub(super) fn defaults_section(_: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.add_space(6.0);
    if ui.button("Restore the default appearance").clicked() {
        send(out, AppearanceMsg::Defaults);
    }
}
