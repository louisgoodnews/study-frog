//! Ctrl+F and `/` focus the search field of the screen in front (STORY-0188), and `search --focus`
//! asks for the same. Each search field asks here whether the shortcut was pressed or the protocol
//! asked; only fields drawn on screen can take focus. When none does, the top-bar field takes it
//! (BUG-0049), and the field that has the keyboard is tracked each frame so the state names only it
//! (BUG-0050).

use eframe::egui::{
    self, Key, Response, TextEdit,
    text::{CCursor, CCursorRange},
};

use crate::chrome::ChromeMsg;
use crate::msg::Msg;
use crate::state::State;

/// The "Find in this PDF" field's name (its accessible name adds the shortcut, see [`named`]).
pub const PDF_FIND: &str = "Find in this PDF";
/// The library's main search field's accessible name.
pub const LIBRARY_TITLES: &str = "Search titles";
/// The library's query box's accessible name.
pub const LIBRARY_QUERY: &str = "Query text";
/// The top bar's search field: the fallback that takes the keyboard when no other field can.
pub const TOP_BAR: &str = "Search items";

/// What the search fields did this frame, kept in the egui context from the first field drawn to
/// [`end_of_frame`].
#[derive(Clone, Default)]
struct FrameFocus {
    /// A field took the keyboard this frame (on request or by the shortcut).
    taken: bool,
    /// The search field that has the keyboard this frame, if any.
    focused: Option<String>,
}

/// The id of the top bar's search field (stable across frames).
pub fn top_bar_id() -> egui::Id {
    egui::Id::unique("top_bar_search_field")
}

fn frame_id() -> egui::Id {
    egui::Id::unique("search_focus_frame")
}

fn top_bar_wanted_id() -> egui::Id {
    egui::Id::unique("search_focus_top_bar_wanted")
}

/// Whether the top bar's field was given the keyboard by [`end_of_frame`] (asked once: the top bar
/// takes it in the frame after, once it is drawn, as the page fields do).
pub fn top_bar_wanted(ctx: &egui::Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(top_bar_wanted_id()))
        .unwrap_or(false)
}

/// The accessible name of a search field: its name and the shortcut, so a screen reader says both.
pub fn named(name: &str) -> String {
    format!("{name} (Ctrl+F or /)")
}

/// Whether the shortcut was pressed this frame: Ctrl+F (Cmd+F on macOS), or `/` while no text field has
/// the keyboard.
pub fn pressed(ctx: &egui::Context) -> bool {
    let (ctrl_f, slash) = ctx.input(|i| {
        (
            i.modifiers.command && !i.modifiers.shift && !i.modifiers.alt && i.key_pressed(Key::F),
            i.key_pressed(Key::Slash) && !i.modifiers.any(),
        )
    });
    let typing = ctx.memory(|m| m.focused().is_some());
    ctrl_f || (slash && !typing)
}

/// Note that the search field `name` has the keyboard after it was drawn (the state names it).
pub fn track(ctx: &egui::Context, response: &Response, name: &str) {
    if ctx.memory(|m| m.has_focus(response.id)) {
        ctx.data_mut(|d| {
            d.get_temp_mut_or_default::<FrameFocus>(frame_id()).focused = Some(name.to_owned());
        });
    }
}

/// Give the field the keyboard when the shortcut was pressed or the protocol asked for it (`requested`),
/// selecting its `len` characters so the next key replaces the text. Call it with the field's response,
/// right after the field is drawn. Returns whether the field took the keyboard (then [`taken`] names it).
pub fn focus_field(
    ctx: &egui::Context,
    response: &Response,
    name: &str,
    len: usize,
    requested: bool,
) -> bool {
    track(ctx, response, name);
    if !(requested || pressed(ctx)) {
        return false;
    }
    response.request_focus();
    if let Some(mut st) = TextEdit::load_state(ctx, response.id) {
        st.cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(len))));
        TextEdit::store_state(ctx, response.id, st);
    }
    ctx.data_mut(|d| {
        let f = d.get_temp_mut_or_default::<FrameFocus>(frame_id());
        f.taken = true;
        f.focused = Some(name.to_owned());
    });
    true
}

/// Report that the field `name` took the keyboard (the answer of `search --focus` names it).
pub fn taken(out: &mut Vec<Msg>, name: &str) {
    out.push(Msg::Chrome(ChromeMsg::SearchFocusTaken(name.to_owned())));
}

/// Called once per frame after the views are drawn. A request, or the shortcut, that no search field
/// took goes to the top-bar field. A field named as focused that is not drawn with the keyboard any
/// more is cleared from the state (so `state` never names a field that has left the screen).
pub fn end_of_frame(ctx: &egui::Context, state: &State) -> Vec<Msg> {
    let frame =
        ctx.data_mut(|d| std::mem::take(d.get_temp_mut_or_default::<FrameFocus>(frame_id())));
    let mut out = Vec::new();
    if !frame.taken && (state.chrome.search_focus || pressed(ctx)) {
        ctx.data_mut(|d| d.insert_temp(top_bar_wanted_id(), true));
        taken(&mut out, TOP_BAR);
    } else if frame.focused.is_none() && state.chrome.focused.is_some() {
        out.push(Msg::Chrome(ChromeMsg::SearchFocusLeft));
    }
    out
}
