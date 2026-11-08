//! The contrastive drill in the window (STORY-0111, `docs/specs/contrastive-drill.md`): match each prompt to
//! its answer among the shuffled answers, see them side by side with what sets each apart, and keep the
//! distinction. Pure; edits the owned state in place.

use studyfrog_core::uid::Uid;
use studyfrog_service::contrast::{CONTRAST_SIZE, ContrastItem, ContrastSaved, ContrastTarget};

use crate::effect::Effect;
use crate::recall_types::{RecallEffect, RecallMsg};
use crate::recall_ui::{effect, notice};
use crate::state::{NoticeKind, State};

/// A drill on screen.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Contrast {
    /// The items asked for.
    pub uids: Vec<Uid>,
    /// The items, once loaded.
    pub items: Vec<ContrastItem>,
    /// The answer slots: slot `k` shows the answer of item `order[k]`.
    pub order: Vec<usize>,
    /// The slot chosen for each item.
    pub picks: Vec<Option<usize>>,
    /// The matches were checked; the answers show side by side.
    pub checked: bool,
    /// The distinction being written.
    pub distinction: String,
    /// Where it goes.
    pub target: ContrastTarget,
    /// What the last save wrote.
    pub saved: Option<String>,
    /// Why loading or saving failed.
    pub error: Option<String>,
    /// Loading or saving.
    pub busy: bool,
}

/// What to keep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContrastSave {
    /// The items.
    pub uids: Vec<Uid>,
    /// The distinction.
    pub text: String,
    /// Hints or a note.
    pub target: ContrastTarget,
}

/// Something happened in the drill.
#[derive(Debug, Clone, PartialEq)]
pub enum ContrastMsg {
    /// Open a drill of these items (2 to 4).
    Open(Vec<Uid>),
    /// The items arrived.
    Loaded(Result<Vec<ContrastItem>, String>),
    /// Item `0` is matched with answer slot `1`.
    Pick(usize, usize),
    /// Check the matches.
    Check,
    /// The distinction was edited.
    Distinction(String),
    /// Where the distinction goes.
    Target(ContrastTarget),
    /// Keep the distinction.
    Save,
    /// It was kept (or not).
    Saved(Result<Box<ContrastSaved>, String>),
    /// Close the drill.
    Close,
}

/// The message for the drill.
pub fn msg(m: ContrastMsg) -> crate::msg::Msg {
    crate::msg::Msg::Recall(RecallMsg::Contrast(m))
}

/// A shuffle of `n` answer slots in which no answer stands under its own prompt.
pub fn shuffled(n: usize, seed: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    let mut x = seed | 1;
    for i in (1..n).rev() {
        x = x
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        order.swap(i, (x >> 33) as usize % (i + 1));
    }
    if n > 1 && order.iter().enumerate().any(|(k, i)| k == *i) {
        let r = 1 + (seed as usize) % (n - 1);
        order = (0..n).map(|k| (k + r) % n).collect();
    }
    order
}

/// For each item: whether its chosen slot holds its own answer.
pub fn right(c: &Contrast) -> Vec<bool> {
    c.picks
        .iter()
        .enumerate()
        .map(|(i, p)| p.is_some_and(|k| c.order.get(k) == Some(&i)))
        .collect()
}

/// The words of `text` longer than three letters (lower case, each once) that none of `others` has.
pub fn own_words(text: &str, others: &[&str]) -> Vec<String> {
    let norm = |w: &str| {
        w.trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase()
    };
    let theirs: std::collections::HashSet<String> = others
        .iter()
        .flat_map(|o| o.split_whitespace().map(norm))
        .collect();
    let mut seen = std::collections::HashSet::new();
    text.split_whitespace()
        .map(norm)
        .filter(|w| w.chars().count() > 3 && !theirs.contains(w) && seen.insert(w.clone()))
        .collect()
}

/// Handle one message.
pub fn update(state: &mut State, m: ContrastMsg) -> Vec<Effect> {
    if let ContrastMsg::Open(uids) = m {
        return open(state, uids);
    }
    let seed = state.wall_ms as u64;
    let Some(c) = state.recall.contrast.as_mut() else {
        return Vec::new();
    };
    match m {
        ContrastMsg::Loaded(Ok(items)) => {
            c.busy = false;
            c.order = shuffled(items.len(), seed);
            c.picks = vec![None; items.len()];
            c.items = items;
        }
        ContrastMsg::Loaded(Err(e)) | ContrastMsg::Saved(Err(e)) => {
            c.busy = false;
            c.error = Some(e);
        }
        ContrastMsg::Pick(i, k) if i < c.picks.len() && k < c.order.len() && !c.checked => {
            c.picks[i] = Some(k)
        }
        ContrastMsg::Check if c.picks.iter().all(Option::is_some) => c.checked = true,
        ContrastMsg::Distinction(t) => c.distinction = t,
        ContrastMsg::Target(t) => c.target = t,
        ContrastMsg::Save if !c.busy && !c.distinction.trim().is_empty() => {
            c.busy = true;
            let save = ContrastSave {
                uids: c.uids.clone(),
                text: c.distinction.clone(),
                target: c.target,
            };
            return vec![effect(RecallEffect::SaveContrast(Box::new(save)))];
        }
        ContrastMsg::Saved(Ok(done)) => return saved(state, *done),
        ContrastMsg::Close => state.recall.contrast = None,
        _ => {}
    }
    Vec::new()
}

fn open(state: &mut State, uids: Vec<Uid>) -> Vec<Effect> {
    let (lo, hi) = CONTRAST_SIZE;
    if !(lo..=hi).contains(&uids.len()) {
        let text = format!("Choose {lo} to {hi} items for a contrastive drill.");
        notice(state, NoticeKind::Error, text);
        return Vec::new();
    }
    state.recall.contrast = Some(Contrast {
        uids: uids.clone(),
        busy: true,
        ..Contrast::default()
    });
    vec![effect(RecallEffect::LoadContrast(uids))]
}

fn saved(state: &mut State, done: ContrastSaved) -> Vec<Effect> {
    let text = format!("Kept the distinction in {}.", done.written.join(", "));
    if let Some(c) = state.recall.contrast.as_mut() {
        c.busy = false;
        c.saved = Some(text.clone());
    }
    state.vault = Some(done.info);
    state.library.undo = Some(done.undo);
    state.library.stale = true;
    notice(state, NoticeKind::Info, text);
    crate::browse::reload(&state.browser.selection)
}
