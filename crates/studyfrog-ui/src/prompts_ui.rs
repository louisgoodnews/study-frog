//! The prompts for an AI (TASK-0206): copy one, save one as a Markdown file, or save them all into a folder. Copying
//! needs no connection to any assistant; the clipboard is the view's, the files are the worker's. Pure.

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// What the user did on the Prompts tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptsMsg {
    /// Save this prompt (its id) as a Markdown file in a folder to choose.
    Save(&'static str),
    /// Save every prompt into a folder to choose.
    SaveAll,
    /// A step of running a prompt through the assistant.
    Run(crate::prompt_run::PromptRunMsg),
    /// The files were written: the folder and the count, `None` when the folder dialog was cancelled, or why not.
    Saved(Result<Option<String>, String>),
}

/// What the worker is asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptsEffect {
    /// Ask for a folder and write the prompts with these ids into it.
    Export(Vec<&'static str>),
    /// Ask for a file to attach.
    Browse,
    /// Read this file to attach it.
    Read(String),
    /// Write the CSV of an answer to a file to import.
    WriteReply(String),
}

fn say(state: State, kind: NoticeKind, text: String) -> (State, Vec<Effect>) {
    (
        State {
            notice: Some(Notice { kind, text }),
            ..state
        },
        Vec::new(),
    )
}

/// Handle the Prompts tab.
pub fn update(state: State, msg: PromptsMsg) -> (State, Vec<Effect>) {
    match msg {
        PromptsMsg::Save(id) => (
            state,
            vec![Effect::Prompts(PromptsEffect::Export(vec![id]))],
        ),
        PromptsMsg::SaveAll => {
            let ids = studyfrog_service::prompts::PROMPTS
                .iter()
                .map(|p| p.id)
                .collect();
            (state, vec![Effect::Prompts(PromptsEffect::Export(ids))])
        }
        PromptsMsg::Run(m) => crate::prompt_run::update(state, m),
        PromptsMsg::Saved(Ok(Some(text))) => say(state, NoticeKind::Info, text),
        PromptsMsg::Saved(Ok(None)) => (state, Vec::new()),
        PromptsMsg::Saved(Err(e)) => say(
            state,
            NoticeKind::Error,
            format!("Could not save the prompts: {e}"),
        ),
    }
}
