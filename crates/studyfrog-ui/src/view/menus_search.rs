//! The top-bar search: the field, its matches, and Create and Import beside it.
use crate::msg::MsgBrowse;

use eframe::egui::{self, Align, Key, Layout, Rect, Role, TextEdit, WidgetInfo};
use studyfrog_core::plan::item_kind_name;

use crate::chrome::ChromeMsg;
use crate::library_ui::LibraryMsg;
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::pdf_types::PdfMsg;
use crate::state::{Selection, State};
use crate::transfer_ui::{Tab, TransferMsg};
use crate::view::kit;
use crate::view::popup::{dismissed, popup};

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
                out.push(Msg::Browse(MsgBrowse::OpenItem(s.uid)));
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
                    out.push(Msg::Browse(MsgBrowse::OpenItem(item.uid)));
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

/// The search field of `width`, then Create, and Import unless `narrow`. A narrow header gives Create its width
/// at the right edge and the search field the room that is left.
pub fn search_cluster(
    state: &State,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    width: f32,
    narrow: bool,
) {
    if narrow {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            create_button(ui, out);
            let room = ui.available_width().max(40.0);
            search_field(state, ui, out, room);
        });
        return;
    }
    search_field(state, ui, out, width);
    create_button(ui, out);
    import_button(state, ui, out);
}

/// Create a new item: the button after the search field.
fn create_button(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    // The view below owns the one accent-filled action; Create stays findable by its accent label.
    let create = kit::quiet_button(ui, "+ Create").on_hover_text("Creates a new item.");
    if create.clicked() {
        out.push(Msg::Modal(ModalMsg::OpenCreate));
    }
}

/// Import, which opens the transfer dialog on its Import tab.
fn import_button(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let import = kit::quiet_button(ui, "Import").on_hover_text(IMPORT_TIP);
    if import.clicked() {
        out.push(Msg::Library(LibraryMsg::Transfer(TransferMsg::Open(
            Tab::Import,
            import_scope(state),
        ))));
    }
}

/// The search field of `width`, with its matches under it.
fn search_field(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>, width: f32) {
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
        out.push(Msg::Browse(MsgBrowse::OpenItem(uid)));
        out.push(Msg::Chrome(ChromeMsg::SearchClear));
    }
    search_results(state, ui, field.rect, out);
}

/// The tooltip of the Import buttons, in the header and in the Profile menu.
pub const IMPORT_TIP: &str =
    "Opens the import dialog: pick a file or folder and see what it holds before importing.";

/// The stack the browser has selected, if any: the import then starts with that stack as its target.
pub fn import_scope(state: &State) -> Option<String> {
    match &state.browser.selection {
        Selection::Stack(folder) => Some(folder.clone()),
        Selection::Nothing | Selection::Inbox => None,
    }
}
