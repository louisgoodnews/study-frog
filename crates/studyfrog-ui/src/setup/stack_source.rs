//! Which stack the rehearse setup's own actions (Case study, Memory game, Mock exam) work on: the
//! stack ticked in the What step, or else the stack the library has open. Pure: no egui, no I/O.

use crate::state::{Selection, State};

/// The stack or inbox an action draws its material from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackSource {
    /// A stack, by its vault-relative folder.
    Stack(String),
    /// The inbox (the library has it open and no stack is ticked).
    Inbox,
    /// Nothing is ticked and nothing is open.
    Nothing,
}

/// The source for an action on the rehearse setup. The first ticked stack in the vault's tree order
/// wins; with none ticked, the library's selection decides.
pub fn stack_source(state: &State) -> StackSource {
    ticked_stack(state).unwrap_or_else(|| match &state.browser.selection {
        Selection::Stack(folder) => StackSource::Stack(folder.clone()),
        Selection::Inbox => StackSource::Inbox,
        Selection::Nothing => StackSource::Nothing,
    })
}

fn ticked_stack(state: &State) -> Option<StackSource> {
    let vault = state.vault.as_ref()?;
    vault
        .overview
        .stacks
        .iter()
        .find(|s| state.rehearse.form.stacks.contains(&s.uid))
        .map(|s| StackSource::Stack(s.folder.clone()))
}
