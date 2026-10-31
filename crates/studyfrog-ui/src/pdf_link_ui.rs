//! Links of a PDF page: pages are followed at once, addresses only after the user says yes. Pure.

use studyfrog_pdf::links::LinkTarget;

use crate::effect::Effect;
use crate::pdf_link_types::LinkMsg;
use crate::pdf_types::PdfView;
use crate::pdf_ui::{fail, go_to, with};
use crate::state::State;

/// Kinds of address that may be offered for opening.
const SCHEMES: [&str; 3] = ["http://", "https://", "mailto:"];

/// True when the address starts with a kind that may be opened.
pub fn openable(uri: &str) -> bool {
    let lower = uri.trim().to_lowercase();
    SCHEMES.iter().any(|s| lower.starts_with(s)) && !uri.chars().any(char::is_control)
}

fn pending(state: State, link: Option<String>) -> (State, Vec<Effect>) {
    let view = state.pdf.clone().map(|v| PdfView {
        pending_link: link,
        ..v
    });
    (State { pdf: view, ..state }, Vec::new())
}

/// Handle a click on a link, or the answer to the question it raised.
pub fn update(state: State, msg: LinkMsg) -> (State, Vec<Effect>) {
    match msg {
        LinkMsg::Click(LinkTarget::Page(p)) => go_to(state, p),
        LinkMsg::Click(LinkTarget::Uri(uri)) if openable(&uri) => {
            pending(state, Some(uri.trim().to_owned()))
        }
        LinkMsg::Click(LinkTarget::Uri(uri)) => {
            let shown: String = uri.chars().take(60).collect();
            let (state, effects) =
                fail(state, format!("A link of this kind is not opened: {shown}"));
            (with_pending_cleared(state), effects)
        }
        LinkMsg::Approve | LinkMsg::Dismiss => pending(state, None),
    }
}

fn with_pending_cleared(state: State) -> State {
    match state.pdf.clone() {
        Some(v) => with(
            state,
            PdfView {
                pending_link: None,
                ..v
            },
        ),
        None => state,
    }
}
