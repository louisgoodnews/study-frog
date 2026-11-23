//! What the top-bar menus contain, and the search results under the search field.

use eframe::egui::{self, Key, Rect, Role, TextEdit, WidgetInfo};
use studyfrog_core::plan::item_kind_name;

use crate::chrome::{ChromeMsg, Menu};
use crate::library_ui::LibraryMsg;
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::pdf_types::PdfMsg;
use crate::state::{Selection, State};
use crate::transfer_ui::{Tab, TransferMsg};
use crate::view::kit;
use crate::view::popup::{close, dismissed, popup};

/// The menu that is open, under its button.
pub fn draw(state: &State, ui: &mut egui::Ui, anchors: &[(Menu, Rect)], out: &mut Vec<Msg>) {
    let Some(open) = state.chrome.menu else {
        return;
    };
    let Some((_, anchor)) = anchors.iter().find(|(m, _)| *m == open) else {
        return;
    };
    let width = match open {
        Menu::Notifications => 420.0,
        Menu::History => 340.0,
        Menu::Vaults => 340.0,
        Menu::Profile => 340.0,
    };
    let at = egui::pos2(
        (anchor.right() - width).max(anchor.left().min(8.0)),
        anchor.bottom() + 4.0,
    );
    let at = if matches!(open, Menu::Vaults | Menu::History) {
        anchor.left_bottom() + egui::vec2(0.0, 4.0)
    } else {
        at
    };
    let rect = popup(ui, "navbar-menu", at, width, |ui| match open {
        Menu::Vaults => super::vault_menu::draw(state, ui, out),
        Menu::Notifications => crate::view::notifications::draw(state, ui, out),
        Menu::Profile => profile(state, ui, out),
        Menu::History => history(state, ui, out),
    });
    if dismissed(ui, rect, *anchor) {
        close(out);
    }
}

/// The places visited, newest first; the current one is marked.
fn history(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let h = &state.history;
    ui.strong("History");
    if h.entries.is_empty() {
        ui.weak("Nothing yet: the places you visit are listed here.");
    }
    for (i, place) in h.entries.iter().enumerate().rev() {
        let text = crate::history::label(state, place);
        let width = ui.available_width();
        let row = egui::Button::selectable(i == h.at, text).min_size(egui::vec2(width, 26.0));
        if ui.add(row).clicked() {
            let go = crate::history::HistoryMsg::Go(i);
            out.push(Msg::Chrome(ChromeMsg::History(go)));
            close(out);
        }
    }
}

fn profile(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = crate::theme::palette_of(ui);
    ui.horizontal(|ui| {
        ui.strong(format!("StudyFrog {}", env!("CARGO_PKG_VERSION")));
        super::kit::pill(ui, "Workspace", p.text_weak);
    });
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(2.0);
    ui.weak("Appearance");
    super::settings_appearance::theme_choice(state, ui, out);
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(2.0);
    super::profiles::draw(state, ui, out);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);
    let close_btn = egui::Button::new(egui::RichText::new("Close vault").color(p.danger).strong())
        .min_size(egui::vec2(ui.available_width(), 26.0));
    if ui
        .add(close_btn)
        .on_hover_text("Closes the vault.")
        .clicked()
    {
        out.push(Msg::CloseVault);
        close(out);
    }
}

/// The matches for the search text, in a list under the field.
pub fn search_results(state: &State, ui: &mut egui::Ui, field: Rect, out: &mut Vec<Msg>) {
    let query = state.chrome.search.trim();
    if query.is_empty() {
        return;
    }
    let rect = popup(
        ui,
        "navbar-search",
        field.left_bottom() + egui::vec2(0.0, 4.0),
        field.width(),
        |ui| {
            let stack = crate::item_look::stack_by_key(state, query);
            if let Some(s) = stack
                && ui
                    .button(format!("stack · {}", s.title))
                    .on_hover_text("Opens this stack.")
                    .clicked()
            {
                out.push(Msg::OpenItem(s.uid));
                out.push(Msg::Chrome(ChromeMsg::SearchClear));
            }
            let none = state.chrome.results.is_empty() && state.chrome.pdf_results.is_empty();
            if none && stack.is_none() {
                ui.weak(format!("No items match \"{query}\"."));
            }
            for item in &state.chrome.results {
                if ui
                    .button(format!("{} · {}", item_kind_name(item.kind), item.title))
                    .on_hover_text("Opens this search result.")
                    .clicked()
                {
                    out.push(Msg::OpenItem(item.uid));
                    out.push(Msg::Chrome(ChromeMsg::SearchClear));
                }
            }
            pdf_hits(state, ui, out);
        },
    );
    if dismissed(ui, rect, field) {
        out.push(Msg::Chrome(ChromeMsg::SearchClear));
    }
}

/// PDFs and pages that match the search, under the items.
fn pdf_hits(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    for hit in &state.chrome.pdf_results {
        let name = crate::pdf_names::label(&hit.path, &hit.title);
        let label = match hit.page {
            Some(p) => format!("PDF · {name} · page {p}\n{}", hit.snippet),
            None => format!("PDF · {name}"),
        };
        if ui
            .button(label)
            .on_hover_text("Opens this PDF at the matching page.")
            .clicked()
        {
            let page = hit.page.map_or(0, |p| p - 1);
            out.push(Msg::Pdf(PdfMsg::Open(hit.path.clone(), page)));
            out.push(Msg::Chrome(ChromeMsg::SearchClear));
        }
    }
}

/// The search field of `width`, then Create, and Import unless `narrow`.
pub fn search_cluster(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    width: f32,
    narrow: bool,
) {
    let mut text = state.chrome.search.clone();
    let field = ui.add(
        TextEdit::singleline(&mut text)
            .id(crate::view::search_focus::top_bar_id())
            .hint_text(crate::view::search_focus::TOP_BAR)
            .desired_width(width),
    );
    if crate::view::search_focus::top_bar_wanted(ui.ctx()) {
        field.request_focus();
    }
    crate::view::search_focus::track(ui.ctx(), &field, crate::view::search_focus::TOP_BAR);
    field.widget_info(|| {
        WidgetInfo::labeled(Role::TextInput, true, crate::view::search_focus::TOP_BAR)
    });
    if text != state.chrome.search {
        out.push(Msg::Chrome(ChromeMsg::SearchEdited(text)));
    }
    if field.has_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
        out.push(Msg::Chrome(ChromeMsg::SearchClear));
    }
    let enter = field.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    let stack = crate::item_look::stack_by_key(state, &state.chrome.search).map(|s| s.uid);
    if let (true, Some(uid)) = (enter, stack.or(state.chrome.results.first().map(|r| r.uid))) {
        out.push(Msg::OpenItem(uid));
        out.push(Msg::Chrome(ChromeMsg::SearchClear));
    }
    search_results(state, ui, field.rect, out);
    // The view below owns the one accent-filled action; Create stays findable by its accent label.
    let create = kit::quiet_button(ui, "+ Create").on_hover_text("Creates a new item.");
    if create.clicked() {
        out.push(Msg::Modal(ModalMsg::OpenCreate));
    }
    if narrow {
        return;
    }
    let import = kit::quiet_button(ui, "Import").on_hover_text(
        "Opens the import dialog: pick a file or folder and see what it holds before importing.",
    );
    if import.clicked() {
        out.push(Msg::Library(LibraryMsg::Transfer(TransferMsg::Open(
            Tab::Import,
            import_scope(state),
        ))));
    }
}

/// The stack the browser has selected, if any: the import then starts with that stack as its target.
fn import_scope(state: &State) -> Option<String> {
    match &state.browser.selection {
        Selection::Stack(folder) => Some(folder.clone()),
        Selection::Nothing | Selection::Inbox => None,
    }
}
