//! Match Mode UI state and message handling logic.
//!
//! Complies with Invariant P1: zero inherent impl blocks; all domain logic in free functions.

use studyfrog_rehearsal::match_mode::{MatchModeSession, match_session_new, select_tile};

use crate::effect::Effect;
use crate::state::State;

/// In-memory state of the active Match Mode UI drill.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchUiState {
    /// Active rehearsal match mode session.
    pub session: MatchModeSession,
    /// Vault-relative stack path if drill is bound to a stack.
    pub stack_path: Option<String>,
    /// Source pairs used to initialize or restart this session.
    pub pairs_source: Vec<(String, String)>,
    /// Personal best score in current app session.
    pub high_score: u32,
}

/// Messages emitted by Match Mode UI interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchMsg {
    /// Start a match session with the provided prompt/answer pairs.
    Start(Vec<(String, String)>),
    /// Start a match session using items available in the current stack/vault.
    StartFromVault,
    /// User clicked tile at index `tile_idx`.
    SelectTile(usize),
    /// Re-shuffles and restarts the match drill with the same pairs.
    Restart,
    /// Leave match mode screen.
    Leave,
}

/// Default biological/science terms used when vault or stack has no pairs.
pub fn default_match_pairs() -> Vec<(String, String)> {
    vec![
        (
            "Mitochondria".into(),
            "Cellular energy production (ATP)".into(),
        ),
        (
            "Ribosome".into(),
            "Protein translation and synthesis".into(),
        ),
        (
            "Endoplasmic Reticulum".into(),
            "Lipid synthesis and protein folding".into(),
        ),
        (
            "Chloroplast".into(),
            "Photosynthetic glucose generation".into(),
        ),
        (
            "Golgi Apparatus".into(),
            "Macromolecule sorting and secretion".into(),
        ),
        ("Lysosome".into(), "Enzymatic waste degradation".into()),
    ]
}

/// Creates a new `MatchUiState`.
pub fn new_match_ui(pairs: Vec<(String, String)>, now_ms: u32, seed: u64) -> MatchUiState {
    let effective_pairs = if pairs.is_empty() {
        default_match_pairs()
    } else {
        pairs
    };
    let session = match_session_new(&effective_pairs, now_ms, seed);
    MatchUiState {
        session,
        stack_path: None,
        pairs_source: effective_pairs,
        high_score: 0,
    }
}

/// Handles user click on tile, updating session and monotonic timer.
pub fn handle_select_tile(state: &mut MatchUiState, tile_idx: usize, now_ms: u32) {
    let outcome = select_tile(&mut state.session, tile_idx, now_ms);
    if let studyfrog_rehearsal::match_mode::MatchOutcome::Completed { .. } = outcome {
        let score = studyfrog_rehearsal::match_mode::compute_match_score(&state.session);
        state.high_score = state.high_score.max(score);
    }
}

/// Restarts the match drill using the existing source pairs with a fresh shuffle.
pub fn handle_restart(state: &mut MatchUiState, now_ms: u32, seed: u64) {
    let pairs = state.pairs_source.clone();
    let high = state.high_score;
    let stack = state.stack_path.clone();
    let session = match_session_new(&pairs, now_ms, seed);
    *state = MatchUiState {
        session,
        stack_path: stack,
        pairs_source: pairs,
        high_score: high,
    };
}

/// Advances Match Mode state in response to `MatchMsg`.
pub fn update_match(match_state: &mut MatchUiState, msg: MatchMsg, now_ms: u32, seed: u64) {
    match msg {
        MatchMsg::Start(pairs) => {
            *match_state = new_match_ui(pairs, now_ms, seed);
        }
        MatchMsg::StartFromVault => {
            handle_restart(match_state, now_ms, seed);
        }
        MatchMsg::SelectTile(idx) => {
            handle_select_tile(match_state, idx, now_ms);
        }
        MatchMsg::Restart => {
            handle_restart(match_state, now_ms, seed);
        }
        MatchMsg::Leave => {}
    }
}

/// Top-level update handler for `Msg::Match`.
pub fn update(mut state: State, msg: MatchMsg) -> (State, Vec<Effect>) {
    let now_ms = (state.now * 1000.0) as u32;
    let seed = (state.wall_ms as u64) ^ ((now_ms as u64) << 16);

    if let MatchMsg::Leave = msg {
        state.screen = crate::state::Screen::Home;
        return (state, Vec::new());
    }

    if let MatchMsg::Start(ref pairs) = msg {
        state.match_state = Some(new_match_ui(pairs.clone(), now_ms, seed));
        state.screen = crate::state::Screen::Match;
        return (state, Vec::new());
    }

    if let MatchMsg::StartFromVault = msg {
        let mut pairs = Vec::new();
        for item in &state.browser.items {
            if !item.title.is_empty() {
                pairs.push((item.title.clone(), format!("Answer for {}", item.title)));
            }
        }
        let effective = if pairs.len() >= 6 {
            pairs
        } else {
            default_match_pairs()
        };
        state.match_state = Some(new_match_ui(effective, now_ms, seed));
        state.screen = crate::state::Screen::Match;
        return (state, Vec::new());
    }

    if state.match_state.is_none() {
        state.match_state = Some(new_match_ui(default_match_pairs(), now_ms, seed));
    }

    if let Some(ref mut ms) = state.match_state {
        update_match(ms, msg, now_ms, seed);
    }

    (state, Vec::new())
}
