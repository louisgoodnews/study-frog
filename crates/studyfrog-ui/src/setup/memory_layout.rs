//! The memory grid's shape: how many columns to use, and how large each card is in the space given.
//! Pure: no egui.

/// A card's height over its width.
pub const CARD_ASPECT: f32 = 1.4;
/// Space between cards, in points.
pub const GAP: f32 = 10.0;
/// The fewest and most columns tried.
const COLUMNS: std::ops::RangeInclusive<usize> = 2..=8;

/// The chosen grid: its shape and the size of one card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    /// Columns.
    pub cols: usize,
    /// Rows (the last one may be short).
    pub rows: usize,
    /// Width of one card.
    pub tile_w: f32,
    /// Height of one card.
    pub tile_h: f32,
}

/// The grid with the largest cards that fits `tiles` cards into `width` by `height` points, trying
/// each column count in range. `None` when there are no cards or no room.
pub fn grid_for(tiles: usize, width: f32, height: f32) -> Option<Grid> {
    if tiles == 0 || width <= 0.0 || height <= 0.0 {
        return None;
    }
    COLUMNS
        .filter_map(|cols| fit(tiles, cols, width, height))
        .max_by(|a, b| a.tile_w.total_cmp(&b.tile_w))
}

/// The grid of `cols` columns, with cards as large as the space allows at the card aspect.
fn fit(tiles: usize, cols: usize, width: f32, height: f32) -> Option<Grid> {
    let rows = tiles.div_ceil(cols);
    let w = (width - GAP * (cols as f32 - 1.0)) / cols as f32;
    let h = (height - GAP * (rows as f32 - 1.0)) / rows as f32;
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let tile_w = w.min(h / CARD_ASPECT);
    Some(Grid {
        cols,
        rows,
        tile_w,
        tile_h: tile_w * CARD_ASPECT,
    })
}

/// The full width of a grid, gaps included.
pub fn grid_width(g: &Grid) -> f32 {
    g.tile_w * g.cols as f32 + GAP * (g.cols as f32 - 1.0)
}

/// The full height of a grid, gaps included.
pub fn grid_height(g: &Grid) -> f32 {
    g.tile_h * g.rows as f32 + GAP * (g.rows as f32 - 1.0)
}
