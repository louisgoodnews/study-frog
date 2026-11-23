//! The top bar: sidebar toggle and vault switcher, brand, global search and Create, and the
//! notification, help, settings and profile buttons on the right. Menus open under their button.

use eframe::egui::{self, Align, Layout, Panel, Rect, RichText, vec2};

use crate::chrome::{ChromeMsg, Menu, unread, vault_label};
use crate::help_types::HelpMsg;
use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::theme::palette_of;
use crate::view::icons::{Glyph, icon_button};
use crate::view::menus;

const HEIGHT: f32 = 40.0;
const LEFT_WIDTH: f32 = 440.0;
/// The room the search keeps clear for the Create and Import buttons after it.
const TRAILING_W: f32 = 176.0;
/// Below this much room after the right-hand buttons, the header keeps only the sidebar toggle and the logo on
/// the left, and Import moves out of the way (BUG-0021: at 640 by 420 the items overlapped).
const NARROW_ROOM: f32 = 320.0;
/// The brand's width in the narrow header: the sidebar toggle and the logo.
const NARROW_LEFT_W: f32 = 72.0;

fn chrome(out: &mut Vec<Msg>, m: ChromeMsg) {
    out.push(Msg::Chrome(m));
}

/// The bar, then whichever menu is open. The right-hand buttons are laid out first and anchored to the window's
/// right edge, and the rest takes the room that is left, so nothing can push the window buttons out of the window.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut anchors: Vec<(Menu, Rect)> = Vec::new();
    Panel::top("navbar").show(ui, |ui| {
        super::window_frame::drag_area(ui, ui.max_rect());
        ui.horizontal(|ui| {
            let signed_in = state.vault.is_some();
            let row = vec2(ui.available_width(), HEIGHT);
            ui.allocate_ui_with_layout(row, Layout::right_to_left(Align::Center), |ui| {
                super::window_frame::controls(ui, out);
                ui.add_space(8.0);
                if signed_in {
                    right_cluster(state, ui, out, &mut anchors);
                } else {
                    help_button(state, ui, out);
                }
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    left_and_middle(state, ui, out, &mut anchors);
                });
            });
        });
    });
    menus::draw(state, ui, &anchors, out);
}

/// The brand on the left and the search in the middle of the room that is left.
fn left_and_middle(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    anchors: &mut Vec<(Menu, Rect)>,
) {
    let total = ui.available_width();
    let narrow = total < NARROW_ROOM;
    let left_w = if narrow {
        NARROW_LEFT_W.min(total)
    } else {
        LEFT_WIDTH.min(total * 0.45)
    };
    let middle_w = (total - left_w - ui.spacing().item_spacing.x).max(0.0);
    let left_to_right = Layout::left_to_right(Align::Center);
    // A section keeps its full width even when its content is narrower, so the sections stay where they are.
    let section = |ui: &mut egui::Ui, w: f32, add: &mut dyn FnMut(&mut egui::Ui)| {
        ui.allocate_ui_with_layout(vec2(w, HEIGHT), left_to_right, |ui| {
            ui.set_min_size(vec2(w, HEIGHT));
            ui.set_max_width(w);
            add(ui);
        });
    };
    section(ui, left_w, &mut |ui| {
        brand_cluster(state, ui, out, anchors, narrow)
    });
    section(ui, middle_w, &mut |ui| {
        // The search, Create and Import are clipped to their room, so at a narrow width they do not draw over the
        // window buttons beside them.
        ui.set_clip_rect(ui.max_rect());
        if state.vault.is_some() {
            let search_w = (middle_w * 0.8 - TRAILING_W).clamp(60.0, 560.0);
            ui.add_space(((middle_w - search_w - TRAILING_W) / 2.0).max(0.0));
            menus::search_cluster(state, ui, out, search_w, narrow);
        }
    });
}

fn toggle(out: &mut Vec<Msg>, menu: Menu) {
    chrome(out, ChromeMsg::ToggleMenu(menu));
}

/// The sidebar toggle, the vault switcher, history, the logo and the name. A narrow header keeps only the toggle and
/// the logo.
fn brand_cluster(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    anchors: &mut Vec<(Menu, Rect)>,
    narrow: bool,
) {
    if state.vault.is_some() {
        if icon_button(
            ui,
            Glyph::Sidebar,
            "Show or hide the sidebar",
            !state.chrome.sidebar_hidden,
        )
        .clicked()
        {
            chrome(out, ChromeMsg::ToggleSidebar);
        }
        if !narrow {
            vault_and_history(state, ui, out, anchors);
        }
    }
    frog(ui);
    if narrow {
        return;
    }
    let brand =
        ui.add(egui::Label::new(RichText::new("StudyFrog").strong()).sense(egui::Sense::click()));
    if state.vault.is_some() && brand.clicked() {
        out.push(Msg::Navigate(Screen::Home));
    }
    if let Some(vault) = &state.vault {
        let name = vault_label(&vault.root);
        // On a narrow window the name is cut short rather than drawn under the search.
        ui.add(egui::Label::new(RichText::new(name).color(palette_of(ui).text_weak)).truncate())
            .on_hover_text(format!("Open vault: {}", vault.root));
    }
}

/// The vault switcher and the back, forward and history buttons.
fn vault_and_history(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    anchors: &mut Vec<(Menu, Rect)>,
) {
    let switcher = icon_button(
        ui,
        Glyph::Grid,
        "Switch vault",
        state.chrome.menu == Some(Menu::Vaults),
    );
    if switcher.clicked() {
        toggle(out, Menu::Vaults);
    }
    anchors.push((Menu::Vaults, switcher.rect));
    super::navbar_history::buttons(state, ui, out, anchors);
}

fn help_button(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let help = icon_button(
        ui,
        Glyph::Help,
        "Help for this screen (F1)",
        state.screen == Screen::Help,
    );
    if help.clicked() {
        out.push(Msg::Help(HelpMsg::ForScreen));
    }
}

/// Right to left: profile, settings, help, notifications.
fn right_cluster(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    anchors: &mut Vec<(Menu, Rect)>,
) {
    let profile = icon_button(
        ui,
        Glyph::Person,
        "Profile and vault",
        state.chrome.menu == Some(Menu::Profile),
    );
    if profile.clicked() {
        toggle(out, Menu::Profile);
    }
    anchors.push((Menu::Profile, profile.rect));
    if icon_button(
        ui,
        Glyph::Gear,
        "Application settings",
        state.screen == Screen::Settings,
    )
    .clicked()
    {
        out.push(Msg::Navigate(Screen::Settings));
    }
    help_button(state, ui, out);
    super::zen::button(state, ui, out);
    let bell = icon_button(
        ui,
        Glyph::Bell,
        "Notifications",
        state.chrome.menu == Some(Menu::Notifications),
    );
    if bell.clicked() {
        toggle(out, Menu::Notifications);
    }
    let n = unread(&state.chrome);
    if n > 0 {
        let badge = egui::pos2(bell.rect.right() - 4.0, bell.rect.top() + 6.0);
        let p = palette_of(ui);
        ui.painter().circle_filled(badge, 7.0, p.danger);
        ui.painter().text(
            badge,
            egui::Align2::CENTER_CENTER,
            n.min(9).to_string(),
            egui::FontId::proportional(10.0),
            p.background,
        );
    }
    anchors.push((Menu::Notifications, bell.rect));
    crate::view::focus_bar::pill(state, ui, out);
}

/// The logo: stylized Breviceps macrops brand mascot.
fn frog(ui: &mut egui::Ui) {
    crate::view::navbar_logo::frog(ui);
}
