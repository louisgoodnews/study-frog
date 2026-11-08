//! Running a standard prompt through the connected assistant with an attached file (TASK-0207). Pure.
//!
//! The steps: choose a prompt and a file, read the file here on this computer, show exactly what would be sent,
//! send it only after the user presses Send, show the answer, import the CSV in it through the import preview and,
//! if the preview finds problems, send the problems back the same way (shown first, sent after Send).

use studyfrog_agent::provider::{Message, Role, is_local, may_send};
use studyfrog_service::prompts::{
    PROMPTS, csv_of_reply, problem_of_reply, prompt_with, run_message,
};
use studyfrog_service::xfer_adapters::CSV_LAYOUT;
use studyfrog_service::xfer_csv_map::ai_message;

use crate::agent_types::{ChatPurpose, selected_provider};
use crate::effect::Effect;
use crate::prompts_ui::PromptsEffect;
use crate::state::State;
use crate::transfer_ui::{Tab, TransferMsg};

const SYSTEM: &str = "You are a careful assistant that follows the user's instructions exactly.";

/// A file read for the assistant: its name, its text (cut to the limit) and whether it was cut.
pub type Attachment = (String, String, bool);

/// Where the run stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PromptRun {
    /// The prompt chosen (its id).
    pub prompt: Option<&'static str>,
    /// The file to attach.
    pub path: String,
    /// The file as it will be sent.
    pub file: Option<Attachment>,
    /// The conversation as it would be sent now; waiting for Send.
    pub outgoing: Option<Vec<Message>>,
    /// The conversation so far.
    pub convo: Vec<Message>,
    /// The latest answer.
    pub reply: Option<String>,
    /// Waiting for the file or the assistant.
    pub busy: bool,
    /// What went wrong.
    pub error: Option<String>,
}

/// What the user does in the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptRunMsg {
    /// Use this prompt.
    Choose(&'static str),
    /// The path of the file was edited.
    Path(String),
    /// Choose the file in a dialog.
    Browse,
    /// The dialog answered.
    Browsed(Option<String>),
    /// Read the file and show what would be sent.
    Prepare,
    /// The file was read.
    Read(Result<Attachment, String>),
    /// Send what is shown.
    Send,
    /// Do not send.
    Cancel,
    /// The assistant answered.
    Replied(Result<String, String>),
    /// Import the CSV in the answer through the import preview.
    ImportReply,
    /// The answer's CSV was written to a file: its path.
    Written(Result<String, String>),
    /// Show the import's problems to the assistant, to be sent after Send.
    AskToFix,
}

fn failed(state: State, why: impl Into<String>) -> (State, Vec<Effect>) {
    let mut state = state;
    state.prompt_run.busy = false;
    state.prompt_run.error = Some(why.into());
    (state, Vec::new())
}

fn prepared(state: State, file: Attachment) -> (State, Vec<Effect>) {
    let mut state = state;
    let Some(source) = PROMPTS
        .iter()
        .find(|p| Some(p.id) == state.prompt_run.prompt)
    else {
        return failed(state, "Choose a prompt first.");
    };
    let picks = crate::prompts_ui::pick_refs(&state.prompts_view.picks);
    let content = run_message(&prompt_with(source, &picks), &file.0, &file.1);
    let run = &mut state.prompt_run;
    run.outgoing = Some(vec![Message {
        role: Role::User,
        content,
    }]);
    (run.convo, run.reply, run.file, run.busy, run.error) =
        (Vec::new(), None, Some(file), false, None);
    (state, Vec::new())
}

fn send(state: State) -> (State, Vec<Effect>) {
    let Some(messages) = state.prompt_run.outgoing.clone() else {
        return (state, Vec::new());
    };
    let Some(provider) = selected_provider(&state.agent) else {
        return failed(state, "Choose a connection first (the Connections tab).");
    };
    if !may_send(&provider) {
        let why = format!(
            "Allow sending text to {} on the Connections tab first.",
            provider.name
        );
        return failed(state, why);
    }
    let mut state = state;
    let run = &mut state.prompt_run;
    (run.convo, run.outgoing, run.busy, run.error) = (messages.clone(), None, true, None);
    let effect = Effect::AgentChat {
        provider,
        system: SYSTEM.to_owned(),
        messages,
        purpose: ChatPurpose::Prompt,
    };
    (state, vec![effect])
}

fn replied(state: State, result: Result<String, String>) -> (State, Vec<Effect>) {
    match result {
        Err(e) => failed(state, e),
        Ok(text) => {
            let mut state = state;
            let run = &mut state.prompt_run;
            run.convo.push(Message {
                role: Role::Assistant,
                content: text.clone(),
            });
            (run.reply, run.busy, run.error) = (Some(text), false, None);
            (state, Vec::new())
        }
    }
}

fn import_reply(state: State) -> (State, Vec<Effect>) {
    let reply = state.prompt_run.reply.clone().unwrap_or_default();
    match (csv_of_reply(&reply), problem_of_reply(&reply)) {
        (Some(csv), _) => (state, vec![Effect::Prompts(PromptsEffect::WriteReply(csv))]),
        (None, Some(why)) => {
            let text = format!("The assistant could not do it: {why}");
            failed(state, text)
        }
        (None, None) => failed(state, "The answer holds no CSV block to import."),
    }
}

fn open_import(state: State, path: String) -> (State, Vec<Effect>) {
    let steps = [
        TransferMsg::Open(Tab::Import, None),
        TransferMsg::Path(path),
        TransferMsg::Preview,
    ];
    steps
        .into_iter()
        .fold((state, Vec::new()), |(state, mut all), step| {
            let (state, more) = crate::transfer_ui::update(state, step);
            all.extend(more);
            (state, all)
        })
}

fn ask_to_fix(state: State) -> (State, Vec<Effect>) {
    let problems = state
        .transfer
        .preview
        .as_ref()
        .and_then(|p| p.csv.as_ref())
        .map(|c| c.problems.clone())
        .unwrap_or_default();
    if problems.is_empty() || state.prompt_run.convo.is_empty() {
        return failed(state, "There is nothing to send back.");
    }
    let mut state = state;
    let mut messages = state.prompt_run.convo.clone();
    messages.push(Message {
        role: Role::User,
        content: ai_message(CSV_LAYOUT, &problems),
    });
    state.prompt_run.outgoing = Some(messages);
    state.prompt_run.error = None;
    state.modal = None;
    crate::nav::navigate(state, crate::state::Screen::Agent)
}

/// Handle the run.
pub fn update(state: State, msg: PromptRunMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    match msg {
        PromptRunMsg::Choose(id) => {
            (state.prompt_run.prompt, state.prompt_run.outgoing) = (Some(id), None)
        }
        PromptRunMsg::Path(text) => {
            (state.prompt_run.path, state.prompt_run.outgoing) = (text, None)
        }
        PromptRunMsg::Browse => return (state, vec![Effect::Prompts(PromptsEffect::Browse)]),
        PromptRunMsg::Browsed(Some(path)) => state.prompt_run.path = path,
        PromptRunMsg::Browsed(None) => {}
        PromptRunMsg::Prepare => {
            let path = state.prompt_run.path.trim().to_owned();
            if state.prompt_run.prompt.is_none() || path.is_empty() {
                return failed(state, "Choose a prompt and a file first.");
            }
            (state.prompt_run.busy, state.prompt_run.error) = (true, None);
            return (state, vec![Effect::Prompts(PromptsEffect::Read(path))]);
        }
        PromptRunMsg::Read(Ok(file)) => return prepared(state, file),
        PromptRunMsg::Read(Err(e))
        | PromptRunMsg::Replied(Err(e))
        | PromptRunMsg::Written(Err(e)) => {
            return failed(state, e);
        }
        PromptRunMsg::Send => return send(state),
        PromptRunMsg::Cancel => state.prompt_run.outgoing = None,
        PromptRunMsg::Replied(ok) => return replied(state, ok),
        PromptRunMsg::ImportReply => return import_reply(state),
        PromptRunMsg::Written(Ok(path)) => return open_import(state, path),
        PromptRunMsg::AskToFix => return ask_to_fix(state),
    }
    (state, Vec::new())
}

/// Whether the connection chosen is on this computer, for the words above Send.
pub fn local(state: &State) -> bool {
    selected_provider(&state.agent).is_some_and(|p| is_local(&p))
}
