//! The ScriptStudio text box (STORY-0150): FrogScript coloured as it is typed, the bracket matching the cursor marked,
//! the first error's line underlined and named, and the names that complete the word before the cursor offered. Tab
//! accepts the highlighted name, Up and Down move the highlight, Esc hides the list until the word changes, and the
//! cursor goes to the end of an inserted name. The pieces are worked out by `studyfrog_script::editor`; this draws them.

use eframe::egui::{self, Key, Modifiers, TextStyle, text::CCursor, text_selection::CCursorRange};
use studyfrog_script::editor::{completions, error_of, step, with_completion, word_at};

mod layout;

use self::layout::layout;

use crate::console_ui::ConsoleMsg;
use crate::msg::Msg;
use crate::theme::palette_of;

/// The byte index of the cursor, as the editor had it at the last frame.
#[derive(Clone, Copy)]
struct CursorAt(usize);

/// The text box's own id, so the cursor can be moved after a completion.
#[derive(Clone, Copy)]
struct BoxId(egui::Id);

/// Where the cursor is (a byte index) and which text box it is in.
#[derive(Clone, Copy)]
struct Caret {
    at: usize,
    box_id: Option<egui::Id>,
}

/// The completion list: the highlighted name, the word it was made for, and a word the user hid with Esc.
#[derive(Clone, Default)]
struct Popup {
    selected: usize,
    prefix: String,
    dismissed: Option<String>,
}

/// How many completions are offered at once.
const OFFERED: usize = 8;

/// The text box, its completions and its first error.
pub fn show(ui: &mut egui::Ui, code: &mut String, out: &mut Vec<Msg>) {
    let cursor = ui
        .data(|d| d.get_temp::<CursorAt>(egui::Id::NULL))
        .map_or(0, |c| c.0);
    let box_id = ui
        .data(|d| d.get_temp::<BoxId>(egui::Id::NULL))
        .map(|b| b.0);
    let mut popup = ui
        .data(|d| d.get_temp::<Popup>(egui::Id::NULL))
        .unwrap_or_default();
    let word = word_at(code, cursor);
    if popup.prefix != word {
        popup.prefix = word.clone();
        popup.selected = 0;
    }
    let names = completions(code, cursor);
    let shown = !names.is_empty() && popup.dismissed.as_deref() != Some(word.as_str());
    let caret = Caret { at: cursor, box_id };
    if shown {
        keys(ui, code, &names, &mut popup, caret, out);
    }
    let error = error_of(code);
    let error_line = error.as_ref().and_then(|e| e.line);
    let p = palette_of(ui);
    let font = TextStyle::Monospace.resolve(ui.style());
    let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap: f32| {
        layout(ui, text.as_str(), Some(cursor), error_line, wrap, &p, &font)
    };
    let edit = egui::TextEdit::multiline(code)
        .id_salt("scriptstudio-multiline-text")
        .font(TextStyle::Monospace)
        .desired_rows(6)
        .lock_focus(true)
        .desired_width(f32::INFINITY)
        .layouter(&mut layouter)
        .show(ui);
    let response = edit.response;
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Script text"));
    ui.data_mut(|d| d.insert_temp(egui::Id::NULL, BoxId(response.id)));
    if let Some(cr) = edit.cursor_range {
        let chars = cr.primary.index.0;
        let at = code
            .char_indices()
            .nth(chars)
            .map_or(code.len(), |(b, _)| b);
        ui.data_mut(|d| d.insert_temp(egui::Id::NULL, CursorAt(at)));
    }
    if let Some(e) = &error {
        let line = e
            .line
            .map_or_else(|| "at the end".to_owned(), |l| format!("line {l}"));
        ui.label(egui::RichText::new(format!("{line}: {}", e.message)).color(p.danger));
    }
    if response.changed() {
        out.push(Msg::Console(ConsoleMsg::StudioEditCode(code.clone())));
    }
    if shown {
        completion_row(ui, &names, &mut popup, code, caret, out);
    }
    ui.data_mut(|d| d.insert_temp(egui::Id::NULL, popup));
}

/// Tab accepts the highlighted name, Up and Down move the highlight, Esc hides the list. The keys are taken before the
/// text box sees them, so they do not also move the text.
fn keys(
    ui: &mut egui::Ui,
    code: &mut String,
    names: &[String],
    popup: &mut Popup,
    caret: Caret,
    out: &mut Vec<Msg>,
) {
    let (tab, down, up, esc) = ui.input_mut(|i| {
        (
            i.consume_key(Modifiers::NONE, Key::Tab),
            i.consume_key(Modifiers::NONE, Key::ArrowDown),
            i.consume_key(Modifiers::NONE, Key::ArrowUp),
            i.consume_key(Modifiers::NONE, Key::Escape),
        )
    });
    if down || up {
        popup.selected = step(popup.selected, names.len(), down);
    }
    if esc {
        popup.dismissed = Some(popup.prefix.clone());
    }
    if tab {
        let name = names[popup.selected.min(names.len() - 1)].clone();
        accept(ui, code, &name, caret, out);
        popup.selected = 0;
    }
}

/// Put `name` in place of the word before the cursor, send the text, and move the cursor to the end of the name. The
/// cursor move is made now, so the box shows it in this frame.
fn accept(ui: &mut egui::Ui, code: &mut String, name: &str, caret: Caret, out: &mut Vec<Msg>) {
    let (text, at) = with_completion(code, caret.at, name);
    *code = text.clone();
    out.push(Msg::Console(ConsoleMsg::StudioEditCode(text)));
    let Some(id) = caret.box_id else { return };
    let chars = code[..at.min(code.len())].chars().count();
    if let Some(mut st) = egui::TextEdit::load_state(ui.ctx(), id) {
        st.cursor
            .set_char_range(Some(CCursorRange::one(CCursor::new(chars))));
        egui::TextEdit::store_state(ui.ctx(), id, st);
    }
    ui.data_mut(|d| d.insert_temp(egui::Id::NULL, CursorAt(at)));
}

/// The names that can complete the word before the cursor, as buttons; the highlighted one is marked, and a click puts
/// a name in place of the word.
fn completion_row(
    ui: &mut egui::Ui,
    names: &[String],
    popup: &mut Popup,
    code: &mut String,
    caret: Caret,
    out: &mut Vec<Msg>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.weak("Complete (Tab accepts, Up/Down choose, Esc hides):");
        for (i, name) in names.iter().take(OFFERED).enumerate() {
            if ui
                .selectable_label(popup.selected == i, name)
                .on_hover_text("Puts this name in place of the word before the cursor.")
                .clicked()
            {
                accept(ui, code, name, caret, out);
                popup.selected = 0;
            }
        }
    });
}
