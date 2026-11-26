//! Match Mode UI state, effects, and message handling logic (EPIC-075).
//!
//! Complies with Invariant P1: zero inherent impl blocks; all domain logic in free functions.

use studyfrog_rehearsal::match_mode::{MatchModeSession, match_session_new, select_tile};
use studyfrog_service::match_pairs::{MatchFilter, sample_match_round};

use crate::effect::Effect;
use crate::state::State;

/// Default number of pairs per Match round.
pub const DEFAULT_PAIR_COUNT: usize = 6;

/// Worker effects requested by Match Mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchEffect {
    /// Load cleaned `(prompt, answer)` pairs from the open vault for the chosen scope and filter.
    LoadPairs {
        /// Vault-relative stack folder, or `None` for the entire vault.
        stack_folder: Option<String>,
        /// Prioritization filter.
        filter: MatchFilter,
    },
}

/// In-memory state of the active Match Mode UI drill.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchUiState {
    /// Active rehearsal match mode session.
    pub session: MatchModeSession,
    /// Vault-relative stack path if drill is bound to a stack (`None` = entire vault).
    pub stack_path: Option<String>,
    /// Active prioritization filter.
    pub filter: MatchFilter,
    /// Configured number of pairs per round (4, 6, or 8).
    pub pair_count: usize,
    /// Full pool of cleaned `(prompt, answer)` pairs loaded from the vault for the current scope.
    pub pool: Vec<(String, String)>,
    /// Source pairs used to initialize or restart the current round.
    pub pairs_source: Vec<(String, String)>,
    /// Monotonic round counter mixed into the shuffle seed so successive rounds always differ.
    pub round_seq: u64,
    /// Whether the worker is currently loading pairs from the vault.
    pub loading: bool,
    /// Last load error, if any.
    pub error: Option<String>,
    /// Personal best score in current app session.
    pub high_score: u32,
}

/// Messages emitted by Match Mode UI interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchMsg {
    /// Start a match session with the provided prompt/answer pairs.
    Start(Vec<(String, String)>),
    /// Load real pairs from the active vault for the current scope/filter.
    StartFromVault,
    /// Worker reply containing the loaded pool of vault pairs.
    PairsLoaded(Result<Vec<(String, String)>, String>),
    /// Switch scope (`None` = entire vault, `Some(folder)` = specific stack) and reload.
    SetScope(Option<String>),
    /// Switch prioritization filter and reload.
    SetFilter(MatchFilter),
    /// Switch pairs per round (4, 6, or 8) and deal a fresh round from the loaded pool.
    SetPairCount(usize),
    /// Deal a fresh randomized subset of pairs from the loaded vault pool.
    NewRound,
    /// User clicked tile at index `tile_idx`.
    SelectTile(usize),
    /// Re-shuffles and restarts the match drill with the same current-round pairs.
    Restart,
    /// Leave match mode screen.
    Leave,
}

/// Default biological/science terms reserved for unit tests and fixtures.
pub fn default_match_pairs() -> Vec<(String, String)> {
    [
        ("Mitochondria", "Cellular energy production (ATP)"),
        ("Ribosome", "Protein translation and synthesis"),
        (
            "Endoplasmic Reticulum",
            "Lipid synthesis and protein folding",
        ),
        ("Chloroplast", "Photosynthetic glucose generation"),
        ("Golgi Apparatus", "Macromolecule sorting and secretion"),
        ("Lysosome", "Enzymatic waste degradation"),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b.into()))
    .collect()
}

fn empty_match_ui(stack_path: Option<String>, filter: MatchFilter, loading: bool) -> MatchUiState {
    MatchUiState {
        session: match_session_new(&[], 0, 0),
        stack_path,
        filter,
        pair_count: DEFAULT_PAIR_COUNT,
        pool: Vec::new(),
        pairs_source: Vec::new(),
        round_seq: 0,
        loading,
        error: None,
        high_score: 0,
    }
}

/// Creates a new `MatchUiState` from explicit `pairs` (falls back to test fixtures when empty).
pub fn new_match_ui(pairs: Vec<(String, String)>, now_ms: u32, seed: u64) -> MatchUiState {
    let effective = if pairs.is_empty() {
        default_match_pairs()
    } else {
        pairs
    };
    let mut ms = empty_match_ui(None, MatchFilter::All, false);
    ms.pair_count = effective.len().clamp(2, 8);
    ms.session = match_session_new(&effective, now_ms, seed);
    ms.pool = effective.clone();
    ms.pairs_source = effective;
    ms.round_seq = 1;
    ms
}

fn deal_from_pool(ms: &mut MatchUiState, now_ms: u32, seed: u64) {
    ms.round_seq = ms.round_seq.wrapping_add(1);
    let mixed = seed ^ ms.round_seq.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    if ms.pool.len() < 2 {
        ms.pairs_source.clear();
        ms.session = match_session_new(&[], now_ms, mixed);
        return;
    }
    let round = sample_match_round(&ms.pool, ms.pair_count, mixed, ms.filter);
    ms.session = match_session_new(&round, now_ms, mixed ^ 0x517c_c1b7_2722_0a95);
    ms.pairs_source = round;
}

/// Request loading real vault pairs when entering `Screen::Match`.
pub fn request_load(state: &mut State) -> Vec<Effect> {
    let (stack_folder, filter) = match state.match_state.as_mut() {
        Some(ms) => {
            ms.loading = true;
            ms.error = None;
            (ms.stack_path.clone(), ms.filter)
        }
        None => {
            state.match_state = Some(empty_match_ui(None, MatchFilter::All, true));
            (None, MatchFilter::All)
        }
    };
    vec![Effect::Match(MatchEffect::LoadPairs {
        stack_folder,
        filter,
    })]
}

/// Handles user click on tile, updating session and monotonic timer.
pub fn handle_select_tile(state: &mut MatchUiState, tile_idx: usize, now_ms: u32) {
    if let studyfrog_rehearsal::match_mode::MatchOutcome::Completed { .. } =
        select_tile(&mut state.session, tile_idx, now_ms)
    {
        let score = studyfrog_rehearsal::match_mode::compute_match_score(&state.session);
        state.high_score = state.high_score.max(score);
    }
}

/// Restarts the match drill using the existing round's pairs with a fresh shuffle.
pub fn handle_restart(state: &mut MatchUiState, now_ms: u32, seed: u64) {
    state.round_seq = state.round_seq.wrapping_add(1);
    let mixed = seed ^ state.round_seq.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    state.session = match_session_new(&state.pairs_source, now_ms, mixed);
}

/// Top-level update handler for `Msg::Match`.
pub fn update(mut state: State, msg: MatchMsg) -> (State, Vec<Effect>) {
    let now_ms = (state.now * 1000.0) as u32;
    let seed = (state.wall_ms as u64) ^ ((now_ms as u64) << 16);
    let mut fx = Vec::new();
    match msg {
        MatchMsg::Leave => state.screen = crate::state::Screen::Home,
        MatchMsg::Start(pairs) => {
            state.match_state = Some(new_match_ui(pairs, now_ms, seed));
            state.screen = crate::state::Screen::Match;
        }
        MatchMsg::StartFromVault => {
            state.screen = crate::state::Screen::Match;
            fx = request_load(&mut state);
        }
        MatchMsg::PairsLoaded(res) => {
            let ms = state
                .match_state
                .get_or_insert_with(|| empty_match_ui(None, MatchFilter::All, false));
            ms.loading = false;
            match res {
                Ok(pool) => {
                    ms.error = None;
                    ms.pool = pool;
                    deal_from_pool(ms, now_ms, seed);
                }
                Err(e) => {
                    ms.pool.clear();
                    ms.pairs_source.clear();
                    ms.session = match_session_new(&[], now_ms, seed);
                    ms.error = Some(e);
                }
            }
        }
        MatchMsg::SetScope(stack_folder) => {
            state
                .match_state
                .get_or_insert_with(|| empty_match_ui(None, MatchFilter::All, true))
                .stack_path = stack_folder;
            fx = request_load(&mut state);
        }
        MatchMsg::SetFilter(filter) => {
            state
                .match_state
                .get_or_insert_with(|| empty_match_ui(None, filter, true))
                .filter = filter;
            fx = request_load(&mut state);
        }
        MatchMsg::SetPairCount(c) => {
            if let Some(ref mut ms) = state.match_state {
                ms.pair_count = if c <= 4 {
                    4
                } else if c <= 6 {
                    6
                } else {
                    8
                };
                deal_from_pool(ms, now_ms, seed);
            }
        }
        MatchMsg::NewRound | MatchMsg::SelectTile(_) | MatchMsg::Restart => {
            if let Some(ref mut ms) = state.match_state {
                match msg {
                    MatchMsg::NewRound => deal_from_pool(ms, now_ms, seed),
                    MatchMsg::SelectTile(idx) => handle_select_tile(ms, idx, now_ms),
                    MatchMsg::Restart => handle_restart(ms, now_ms, seed),
                    _ => {}
                }
            }
        }
    }
    (state, fx)
}
