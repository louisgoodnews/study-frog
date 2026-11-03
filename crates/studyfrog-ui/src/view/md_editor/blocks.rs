//! The parsed preview of an editor's text, kept in egui's memory so it is parsed only when the text changes.

use std::sync::Arc;

use eframe::egui::{self, Id};
use studyfrog_md::blocks::{Block, parse_blocks};

pub(super) type Cached = Arc<(String, Vec<Block>)>;

pub(super) fn blocks_of(ui: &egui::Ui, id: Id, text: &str) -> Arc<(String, Vec<Block>)> {
    let key = id.with("blocks");
    if let Some(hit) = ui
        .data(|d| d.get_temp::<Cached>(key))
        .filter(|c| c.0 == text)
    {
        return hit;
    }
    let fresh: Cached = Arc::new((text.to_owned(), parse_blocks(text)));
    ui.data_mut(|d| d.insert_temp(key, fresh.clone()));
    fresh
}
