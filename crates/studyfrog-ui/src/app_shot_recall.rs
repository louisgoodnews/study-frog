//! Screens of the active-recall workflows for `--screenshot` (EPIC-035): `extract:<ULID>` (a note's page with
//! its sync preview open), `focus` (the timer running on the home screen) and `focus-done` (a finished
//! work block with its card). Also the memory game's demo screens (`memory-demo`, `memory-won`).

use crate::effect::Effect;
use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::recall_types::RecallMsg;
use crate::state::Screen;

/// Go to the screen named `name`, if it is one of these.
pub fn setup(model: &mut Model, name: &str) -> Option<Vec<Effect>> {
    if name.starts_with("focus") {
        return Some(focus(model, name == "focus-done"));
    }
    let (kind, uid) = name.split_once(':')?;
    if kind == "contrast" || kind == "compare" {
        let uids: Option<Vec<_>> = uid.split(',').map(studyfrog_md::ulid::parse_ulid).collect();
        let open = crate::contrast::ContrastMsg::Open(uids?);
        return Some(apply(model, crate::contrast::msg(open)));
    }
    let uid = studyfrog_md::ulid::parse_ulid(uid)?;
    match kind {
        "extract" => Some(apply(model, Msg::OpenItem(uid))),
        _ => None,
    }
}

/// Once the screen has loaded: open what the picture should show.
pub fn later(model: &mut Model, name: &str) -> Vec<Effect> {
    if name.starts_with("compare:") {
        return compared(model);
    }
    if name.starts_with("extract:")
        && let Some(e) = &model.state.editor
    {
        let body = e
            .form
            .fields
            .first()
            .map(|f| f.text.clone())
            .unwrap_or_default();
        return apply(model, Msg::Recall(RecallMsg::ExtractPreview(e.uid, body)));
    }
    Vec::new()
}

/// While the picture waits: `run-answer` shows the first item's answer.
pub fn play(model: &mut Model, name: &str) -> Vec<Effect> {
    let ready = model.state.rehearse.run.as_ref().is_some_and(|run| {
        crate::run_update::awaiting_reveal(run)
            && studyfrog_rehearsal::session::current_item(&run.session)
                .is_some_and(|i| run.content.contains_key(&i))
    });
    if name == "run-answer" && ready {
        let flip = studyfrog_rehearsal::input::Action::Flip;
        return apply(model, Msg::RunAction(flip));
    }
    Vec::new()
}

fn focus(model: &mut Model, finished: bool) -> Vec<Effect> {
    use crate::focus_ui::{FocusMsg, msg};
    let first = model
        .state
        .vault
        .as_ref()
        .and_then(|v| v.overview.stacks.first());
    let stack = first.map(|s| s.folder.clone());
    let mut fx = apply(model, Msg::Navigate(crate::state::Screen::Home));
    fx.extend(apply(model, msg(FocusMsg::Stack(stack.clone()))));
    fx.extend(apply(model, msg(FocusMsg::Start)));
    let f = &mut model.state.recall.focus;
    f.banked = 95.0;
    if finished {
        crate::focus_timer::reset(f);
        f.phase = crate::focus_timer::Phase::ShortBreak;
        f.blocks = 1;
        let wall = model.state.wall_ms;
        f.done = Some(crate::focus_timer::FocusLog {
            started_ms: wall - 1_500_000,
            ended_ms: wall,
            minutes: 25,
            stack,
        });
    }
    fx
}

/// The drill with every prompt matched, checked, and a distinction typed.
fn compared(model: &mut Model) -> Vec<Effect> {
    use crate::contrast::{ContrastMsg, msg};
    let Some(c) = &model.state.recall.contrast else {
        return Vec::new();
    };
    let order = c.order.clone();
    for (k, i) in order.iter().enumerate() {
        apply(model, msg(ContrastMsg::Pick(*i, k)));
    }
    apply(model, msg(ContrastMsg::Check));
    let text = "Ionic: electrons are given away; covalent: they are shared.".to_owned();
    apply(model, msg(ContrastMsg::Distinction(text)))
}

/// A memory game with three cards turned, so the screenshot shows a face-up card, a matched pair
/// and a mismatch waiting to hide. The clock is parked far in the future, so nothing hides by itself.
pub fn memory_demo(model: &mut Model) {
    use studyfrog_rehearsal::memory::{flip_tile, new_memory_session};
    let cards: Vec<(String, String)> = (0..6)
        .map(|i| (format!("Front {i}"), format!("Back {i}")))
        .collect();
    let mut s = new_memory_session(&cards, 6, 7, 900, None);
    let of_pair = |s: &studyfrog_rehearsal::memory::MemorySession, p: u32| -> Vec<usize> {
        s.tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.pair == p)
            .map(|(i, _)| i)
            .collect()
    };
    let (a, b, c) = (of_pair(&s, 0), of_pair(&s, 1), of_pair(&s, 2));
    for i in [a[0], a[1], b[0], c[0]] {
        if let Ok(next) = flip_tile(&s, i) {
            s = next;
        }
    }
    model.state.screen = Screen::Rehearse;
    model.state.rehearse.memory = Some(crate::setup::memory::MemoryUi {
        session: Some(s),
        started_ms: i64::MAX / 2,
        ..Default::default()
    });
}

/// A finished memory game with its victory dialog open, as the screenshot of the summary.
pub fn memory_won(model: &mut Model) {
    use studyfrog_rehearsal::memory::{flip_tile, new_memory_session};
    use studyfrog_rehearsal::memory_summary::{MemoryTimer, victory_summary};
    let cards: Vec<(String, String)> = (0..4)
        .map(|i| (format!("Front {i}"), format!("Back {i}")))
        .collect();
    let mut s = new_memory_session(&cards, 4, 7, 900, None);
    s.elapsed_ms = 74_000;
    for p in 0..4u32 {
        let idx: Vec<usize> = s
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.pair == p)
            .map(|(i, _)| i)
            .collect();
        for i in idx {
            if let Ok(next) = flip_tile(&s, i) {
                s = next;
            }
        }
    }
    let prior = Some(95_000);
    model.state.screen = Screen::Rehearse;
    let summary = victory_summary(&s, MemoryTimer::Stopwatch, prior);
    model.state.rehearse.memory = Some(crate::setup::memory::MemoryUi {
        session: Some(s),
        started_ms: i64::MAX / 2,
        prior_best_ms: prior,
        ..Default::default()
    });
    model.state.modal =
        summary.map(|v| crate::modal_types::Modal::MemorySummary(Box::new(v), prior));
}

/// The memory game's demo screens, by name (`memory-demo`, `memory-won`).
pub fn memory_setup(model: &mut Model, name: &str) -> Option<Vec<crate::effect::Effect>> {
    match name {
        "memory-demo" => memory_demo(model),
        "memory-won" => memory_won(model),
        _ => return None,
    }
    Some(Vec::new())
}
