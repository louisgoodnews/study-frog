//! Memory Mode decks: each flashcard becomes two face-down tiles, its front and its back, shuffled
//! with a seeded generator so the same seed always gives the same layout (STORY-0173). Pure.

/// Which side of the card a tile shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileRole {
    /// The question side.
    Front,
    /// The answer side.
    Back,
}

/// Where a tile is in the flip cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlipState {
    /// Hidden.
    FaceDown,
    /// Shown, waiting for its partner.
    FaceUp,
    /// Shown together with a non-matching partner, until the delay ends.
    MismatchWait,
    /// Paired and out of play.
    Matched,
}

/// One tile on the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryTile {
    /// Position on the grid.
    pub index: usize,
    /// The card this tile belongs to; the front and back of one card share it.
    pub pair: u32,
    /// Which side is shown.
    pub role: TileRole,
    /// The text on that side.
    pub text: String,
    /// Where the tile is in the flip cycle.
    pub state: FlipState,
}

/// Deterministic shuffle source (SplitMix64).
fn next_random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Fisher-Yates with the seeded generator.
pub fn shuffle_seeded<T>(items: &mut [T], seed: u64) {
    let mut state = seed;
    for i in (1..items.len()).rev() {
        let j = (next_random(&mut state) % (i as u64 + 1)) as usize;
        items.swap(i, j);
    }
}

/// Build the tiles for the first `pairs` usable cards (each gives a front and a back tile), shuffled
/// by `seed`. A pair id is the card's position in the deck. Cards with an empty front or back are
/// skipped. Returns no tiles when `pairs` is zero.
pub fn build_tiles(cards: &[(String, String)], pairs: usize, seed: u64) -> Vec<MemoryTile> {
    let mut tiles: Vec<MemoryTile> = Vec::new();
    let usable = cards
        .iter()
        .enumerate()
        .filter(|(_, (f, b))| !f.trim().is_empty() && !b.trim().is_empty())
        .take(pairs);
    for (pair, (front, back)) in usable {
        let pair = u32::try_from(pair).unwrap_or(u32::MAX);
        for (role, text) in [(TileRole::Front, front), (TileRole::Back, back)] {
            tiles.push(MemoryTile {
                index: 0,
                pair,
                role,
                text: text.clone(),
                state: FlipState::FaceDown,
            });
        }
    }
    shuffle_seeded(&mut tiles, seed);
    for (i, tile) in tiles.iter_mut().enumerate() {
        tile.index = i;
    }
    tiles
}
