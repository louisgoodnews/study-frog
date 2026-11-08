//! Screens of the active-recall workflows for `--screenshot` (EPIC-035): `extract:<ULID>` (a note's page with
//! its sync preview open).

use crate::effect::Effect;
use crate::model::{Model, apply};
use crate::msg::Msg;
use crate::recall_types::RecallMsg;

/// Go to the screen named `name`, if it is one of these.
pub fn setup(model: &mut Model, name: &str) -> Option<Vec<Effect>> {
    let (kind, uid) = name.split_once(':')?;
    let uid = studyfrog_md::ulid::parse_ulid(uid)?;
    match kind {
        "extract" => Some(apply(model, Msg::OpenItem(uid))),
        _ => None,
    }
}

/// Once the screen has loaded: open what the picture should show.
pub fn later(model: &mut Model, name: &str) -> Vec<Effect> {
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
