//! Following a wikilink in the editor: open the item it names, or say why it cannot.
use super::*;

pub(super) fn link_resolved(
    state: State,
    target: &str,
    result: Result<Option<Uid>, String>,
) -> (State, Vec<Effect>) {
    match result {
        Ok(Some(uid)) => open_item(state, uid),
        Ok(None) => notice(
            state,
            NoticeKind::Error,
            format!("No item is titled \"{target}\"."),
        ),
        Err(e) => notice(
            state,
            NoticeKind::Error,
            format!("Could not follow the link: {e}"),
        ),
    }
}
