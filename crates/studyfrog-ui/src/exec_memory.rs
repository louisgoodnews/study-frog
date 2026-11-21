//! Memory mode's card reading: the cards of a stack, as (front, back), for a memory game. Thin: the
//! queries live in the store and the content loader.

use studyfrog_core::plan::ItemKind;
use studyfrog_service::content::load_item_content;
use studyfrog_store::overview::list_items;

use crate::exec::Worker;
use crate::msg::Msg;
use crate::setup::memory::MemoryMsg;

/// The most cards read from one stack; a game uses at most eight pairs of them.
const MAX_CANDIDATES: usize = 40;

/// The cards of the stack (or the inbox) that a memory game can use.
pub fn deck(w: &Worker, folder: Option<String>) -> Msg {
    let result = match &w.session {
        None => Err("no vault is open".to_owned()),
        Some(s) => read_cards(s, folder.as_deref()),
    };
    Msg::Memory(MemoryMsg::Loaded(result))
}

fn read_cards(
    s: &studyfrog_service::session::VaultSession,
    folder: Option<&str>,
) -> Result<Vec<(String, String)>, String> {
    let rows = list_items(&s.conn, folder).map_err(|e| e.to_string())?;
    rows.iter()
        .filter(|row| row.kind == ItemKind::Card)
        .take(MAX_CANDIDATES)
        .map(|row| {
            load_item_content(s, row.uid)
                .map(|c| (c.prompt, c.answer))
                .map_err(|e| e.to_string())
        })
        .collect()
}
