//! The prompts for an AI (TASK-0206): copy one, save one as a Markdown file, or save them all into a folder. Copying
//! needs no connection to any assistant; the clipboard is the view's, the files are the worker's. Pure.

use studyfrog_service::prompts::{PROMPTS, PromptSource, prompt_of};
use studyfrog_service::prompts_meta::{PromptCategory, category_of};

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// What the Prompts tab shows: the category and the search, and the options picked in the prompts that have any.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PromptsView {
    /// Only this category; `None` shows all.
    pub category: Option<PromptCategory>,
    /// Only prompts whose title or summary contains this text.
    pub query: String,
    /// The picked options as (parameter key, option index).
    pub picks: Vec<(String, usize)>,
}

/// The picks as the pairs the prompt functions take.
pub fn pick_refs(picks: &[(String, usize)]) -> Vec<(&str, usize)> {
    picks.iter().map(|(key, at)| (key.as_str(), *at)).collect()
}

/// The prompts the category and the search leave, in the order of the list.
pub fn visible_prompts(view: &PromptsView) -> Vec<&'static PromptSource> {
    let query = view.query.trim().to_lowercase();
    PROMPTS
        .iter()
        .filter(|s| view.category.is_none_or(|c| category_of(s.id) == c))
        .filter(|s| {
            let p = prompt_of(s);
            let category = studyfrog_service::prompts_meta::category_name(category_of(s.id));
            let hay = format!("{} {} {category}", p.title, p.summary).to_lowercase();
            query.is_empty() || hay.contains(&query)
        })
        .collect()
}

/// What the user did on the Prompts tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptsMsg {
    /// Save this prompt (its id) as a Markdown file in a folder to choose.
    Save(&'static str),
    /// Save every prompt into a folder to choose.
    SaveAll,
    /// Show only this category (`None`: all).
    Category(Option<PromptCategory>),
    /// The search box changed.
    Query(String),
    /// An option was picked: the parameter's key and the option's index.
    Pick(&'static str, usize),
    /// A step of running a prompt through the assistant.
    Run(crate::prompt_run::PromptRunMsg),
    /// The files were written: the folder and the count, `None` when the folder dialog was cancelled, or why not.
    Saved(Result<Option<String>, String>),
}

/// What the worker is asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptsEffect {
    /// Ask for a folder and write the prompts with these ids into it, with these picked options.
    Export(Vec<&'static str>, Vec<(String, usize)>),
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
        PromptsMsg::Save(id) => {
            let picks = state.prompts_view.picks.clone();
            (
                state,
                vec![Effect::Prompts(PromptsEffect::Export(vec![id], picks))],
            )
        }
        PromptsMsg::SaveAll => {
            let ids = PROMPTS.iter().map(|p| p.id).collect();
            let picks = state.prompts_view.picks.clone();
            (
                state,
                vec![Effect::Prompts(PromptsEffect::Export(ids, picks))],
            )
        }
        PromptsMsg::Category(category) => {
            let mut state = state;
            state.prompts_view.category = category;
            (state, Vec::new())
        }
        PromptsMsg::Query(query) => {
            let mut state = state;
            state.prompts_view.query = query;
            (state, Vec::new())
        }
        PromptsMsg::Pick(key, at) => {
            let mut state = state;
            let picks = &mut state.prompts_view.picks;
            picks.retain(|(k, _)| k != key);
            picks.push((key.to_owned(), at));
            (state, Vec::new())
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
