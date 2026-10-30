//! The top bar: sidebar toggle and vault switcher, brand, global search and Create, and the
//! notification, help, settings and profile buttons on the right. Menus open under their button.

use eframe::egui::{
    self, Align, Color32, Key, Layout, Panel, Rect, RichText, Role, TextEdit, WidgetInfo, vec2,
};

use crate::chrome::{ChromeMsg, Menu, unread};
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::view::icons::{Glyph, icon_button};
use crate::view::kit;
use crate::view::menus;

const HEIGHT: f32 = 40.0;
const LEFT_WIDTH: f32 = 330.0;
const RIGHT_WIDTH: f32 = 150.0;

fn chrome(out: &mut Vec<Msg>, m: ChromeMsg) {
    out.push(Msg::Chrome(m));
}

/// The bar, then whichever menu is open.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut anchors: Vec<(Menu, Rect)> = Vec::new();
    Panel::top("navbar").show(ui, |ui| {
        ui.horizontal(|ui| {
            let total = ui.available_width();
            let signed_in = state.vault.is_some();
            let (left_w, right_w) = (
                LEFT_WIDTH.min(total * 0.4),
                if signed_in { RIGHT_WIDTH } else { 40.0 },
            );
            let left_to_right = Layout::left_to_right(Align::Center);
            // A section keeps its full width even when its content is narrower, so the sections
            // stay where they are: left, centre, right.
            let section =
                |ui: &mut egui::Ui, w: f32, layout: Layout, add: &mut dyn FnMut(&mut egui::Ui)| {
                    ui.allocate_ui_with_layout(vec2(w, HEIGHT), layout, |ui| {
                        ui.set_min_size(vec2(w, HEIGHT));
                        add(ui);
                    });
                };
            section(ui, left_w, left_to_right, &mut |ui| {
                brand_cluster(state, ui, out, &mut anchors)
            });
            let middle_w = (total - left_w - right_w - 2.0 * ui.spacing().item_spacing.x).max(0.0);
            section(ui, middle_w, left_to_right, &mut |ui| {
                if signed_in {
                    let search_w = (middle_w * 0.8 - 90.0).clamp(160.0, 560.0);
                    ui.add_space(((middle_w - search_w - 90.0) / 2.0).max(0.0));
                    search_cluster(state, ui, out, search_w);
                }
            });
            section(
                ui,
                right_w,
                Layout::right_to_left(Align::Center),
                &mut |ui| {
                    if signed_in {
                        right_cluster(state, ui, out, &mut anchors);
                    } else {
                        help_button(state, ui, out, &mut anchors);
                    }
                },
            );
        });
    });
    menus::draw(state, ui, &anchors, out);
}

fn toggle(out: &mut Vec<Msg>, menu: Menu) {
    chrome(out, ChromeMsg::ToggleMenu(menu));
}

fn brand_cluster(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    anchors: &mut Vec<(Menu, Rect)>,
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
    }
    frog(ui);
    let brand =
        ui.add(egui::Label::new(RichText::new("StudyFrog").strong()).sense(egui::Sense::click()));
    if state.vault.is_some() && brand.clicked() {
        out.push(Msg::Navigate(Screen::Home));
    }
    if let Some(v) = &state.vault {
        ui.weak(&v.name);
    }
}

fn search_cluster(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>, width: f32) {
    let mut text = state.chrome.search.clone();
    let field = ui.add(
        TextEdit::singleline(&mut text)
            .hint_text("Search items")
            .desired_width(width),
    );
    field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Search items"));
    if text != state.chrome.search {
        chrome(out, ChromeMsg::SearchEdited(text));
    }
    if field.has_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
        chrome(out, ChromeMsg::SearchClear);
    }
    let enter = field.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    if let (true, Some(first)) = (enter, state.chrome.results.first()) {
        out.push(Msg::OpenItem(first.uid));
        chrome(out, ChromeMsg::SearchClear);
    }
    menus::search_results(state, ui, field.rect, out);
    let create = kit::primary_button(ui, "+ Create");
    if create.clicked() {
        out.push(Msg::Modal(ModalMsg::OpenCreate));
    }
}

fn help_button(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    anchors: &mut Vec<(Menu, Rect)>,
) {
    let help = icon_button(
        ui,
        Glyph::Help,
        "Help",
        state.chrome.menu == Some(Menu::Help),
    );
    if help.clicked() {
        toggle(out, Menu::Help);
    }
    anchors.push((Menu::Help, help.rect));
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
    help_button(state, ui, out, anchors);
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
        ui.painter()
            .circle_filled(badge, 7.0, Color32::from_rgb(210, 60, 60));
        ui.painter().text(
            badge,
            egui::Align2::CENTER_CENTER,
            n.min(9).to_string(),
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
    }
    anchors.push((Menu::Notifications, bell.rect));
}

/// The logo: a green frog face.
fn frog(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::hover());
    let (c, green, dark) = (
        rect.center(),
        Color32::from_rgb(111, 207, 127),
        Color32::from_rgb(16, 40, 26),
    );
    let p = ui.painter();
    p.circle_filled(egui::pos2(c.x - 6.0, c.y - 5.0), 5.0, green);
    p.circle_filled(egui::pos2(c.x + 6.0, c.y - 5.0), 5.0, green);
    p.add(egui::Shape::ellipse_filled(
        egui::pos2(c.x, c.y + 2.0),
        vec2(11.0, 8.5),
        green,
    ));
    p.circle_filled(egui::pos2(c.x - 6.0, c.y - 5.0), 2.0, dark);
    p.circle_filled(egui::pos2(c.x + 6.0, c.y - 5.0), 2.0, dark);
}
