//! Screens of the active-recall workflows for `--screenshot` (EPIC-035): `extract:<ULID>` (a note's page with
//! its sync preview open), `focus` (the timer running on the home screen) and `focus-done` (a finished
//! work block with its card).

use crate::effect::Effect;
use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::recall_types::RecallMsg;

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
