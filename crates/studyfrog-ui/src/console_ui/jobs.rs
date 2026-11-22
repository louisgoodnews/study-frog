//! The scheduled and event-bound jobs in the app (STORY-0151): the job list from `scripts.toml`, each job's next run,
//! its enabled state and events, its history, a save that is checked before it is written, and runs by hand (the
//! Console runs them, so a manual run is the same as a typed `script run`). Pure: the file is read and written by
//! effects.

use studyfrog_service::scripting::config::Access;
use studyfrog_service::scripting::jobs::JobRow;

use crate::console_ui::ConsoleEffect;
use crate::effect::Effect;
use crate::modal::Modal;
use crate::state::State;

/// The jobs as the editor holds them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobsEditor {
    /// The jobs, in file order.
    pub rows: Vec<JobRow>,
    /// Why some entries of the file could not be used.
    pub errors: Vec<String>,
    /// The interpreter for `.py` scripts.
    pub python: String,
    /// The script files that can be run by hand.
    pub scripts: Vec<String>,
    /// The job whose details are shown.
    pub selected: Option<usize>,
    /// The selected job's recent runs, from the app log.
    pub history: Vec<String>,
    /// The script chosen for a run by hand.
    pub adhoc_script: String,
    /// The arguments typed for a run by hand.
    pub adhoc_args: String,
    /// The name typed for a new job.
    pub new_name: String,
    /// The script chosen for a new job.
    pub new_script: String,
    /// Changes not yet written.
    pub dirty: bool,
    /// The last thing that happened, for the dialog to say.
    pub notice: Option<String>,
}

/// What was read from the file and the scripts folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    /// The jobs, with their next runs.
    pub rows: Vec<JobRow>,
    /// Why some entries could not be used.
    pub errors: Vec<String>,
    /// The interpreter for `.py` scripts.
    pub python: String,
    /// The script files that can be run by hand.
    pub scripts: Vec<String>,
}

/// What the job editor asks the executor to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobsEffect {
    /// Read the jobs and the scripts.
    Load,
    /// Check and write these jobs.
    Save(Vec<JobRow>, String),
    /// Read the recent runs of this job.
    History(String),
}

/// What happened in the job editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobsMsg {
    /// The editor was opened (from the Console).
    Open,
    /// The jobs were read (or not).
    Loaded(Result<Loaded, String>),
    /// A job was chosen in the list.
    Select(usize),
    /// A job was switched on or off.
    Enabled(usize, bool),
    /// A job's schedule was edited (empty: none).
    Schedule(usize, String),
    /// A lifecycle event was ticked or unticked for a job.
    EventOn(usize, String, bool),
    /// "Save" was pressed.
    Save,
    /// The jobs were written (or not).
    Saved(Result<(), String>),
    /// The history of a job was read.
    HistoryLoaded(Vec<String>),
    /// The script chosen for a run by hand.
    AdhocScript(String),
    /// The arguments typed for a run by hand.
    AdhocArgs(String),
    /// The name typed for a new job.
    NewName(String),
    /// The script chosen for a new job.
    NewScript(String),
    /// "Add job" was pressed: a new job with this name and script, switched on, with no schedule or events yet.
    AddJob,
}

/// Open the editor: the dialog shows at once and the jobs are read.
pub fn open(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    state.modal = Some(Modal::ScriptJobs(Box::new(JobsEditor {
        rows: Vec::new(),
        errors: Vec::new(),
        python: "python3".into(),
        scripts: Vec::new(),
        selected: None,
        history: Vec::new(),
        adhoc_script: String::new(),
        adhoc_args: String::new(),
        new_name: String::new(),
        new_script: String::new(),
        dirty: false,
        notice: Some("Reading the jobs…".into()),
    })));
    (
        state,
        vec![Effect::Console(ConsoleEffect::Jobs(JobsEffect::Load))],
    )
}

/// Apply a message to the job editor in the open dialog.
pub fn update(state: State, m: JobsMsg) -> (State, Vec<Effect>) {
    if m == JobsMsg::Open {
        return open(state);
    }
    let mut state = state;
    let asked = match state.modal.as_mut() {
        Some(Modal::ScriptJobs(e)) => apply(e, m),
        _ => Vec::new(),
    };
    let effects = asked
        .into_iter()
        .map(|a| Effect::Console(ConsoleEffect::Jobs(a)))
        .collect();
    (state, effects)
}

fn apply(e: &mut JobsEditor, m: JobsMsg) -> Vec<JobsEffect> {
    match m {
        JobsMsg::Open => {}
        JobsMsg::Loaded(Ok(l)) => {
            e.rows = l.rows;
            e.errors = l.errors;
            e.python = l.python;
            e.scripts = l.scripts;
            e.adhoc_script = e.scripts.first().cloned().unwrap_or_default();
            e.dirty = false;
            e.notice = None;
            if let Some(i) = e.selected.filter(|i| *i < e.rows.len()) {
                return vec![JobsEffect::History(e.rows[i].name.clone())];
            }
        }
        JobsMsg::Loaded(Err(why)) => e.notice = Some(format!("Could not read the jobs: {why}")),
        JobsMsg::Select(i) => {
            if i < e.rows.len() {
                e.selected = Some(i);
                return vec![JobsEffect::History(e.rows[i].name.clone())];
            }
        }
        JobsMsg::Enabled(i, on) => {
            if let Some(r) = e.rows.get_mut(i) {
                r.enabled = on;
                e.dirty = true;
            }
        }
        JobsMsg::Schedule(i, text) => {
            if let Some(r) = e.rows.get_mut(i) {
                r.schedule = text;
                e.dirty = true;
            }
        }
        JobsMsg::EventOn(i, event, on) => {
            if let Some(r) = e.rows.get_mut(i) {
                r.on.retain(|x| x != &event);
                if on {
                    r.on.push(event);
                }
                e.dirty = true;
            }
        }
        JobsMsg::Save => {
            return vec![JobsEffect::Save(e.rows.clone(), e.python.clone())];
        }
        JobsMsg::Saved(Ok(())) => {
            e.dirty = false;
            e.notice = Some("Saved. The new settings apply from the next run.".into());
            return vec![JobsEffect::Load];
        }
        JobsMsg::Saved(Err(why)) => e.notice = Some(format!("Not saved: {why}")),
        JobsMsg::HistoryLoaded(lines) => e.history = lines,
        JobsMsg::AdhocScript(s) => e.adhoc_script = s,
        JobsMsg::AdhocArgs(a) => e.adhoc_args = a,
        JobsMsg::NewName(n) => e.new_name = n,
        JobsMsg::NewScript(s) => e.new_script = s,
        JobsMsg::AddJob => {
            let name = e.new_name.trim().to_owned();
            if name.is_empty() || e.new_script.is_empty() {
                e.notice = Some("A new job needs a name and a script.".into());
            } else if e.rows.iter().any(|r| r.name == name) {
                e.notice = Some(format!("A job called {name} already exists."));
            } else {
                e.rows.push(JobRow {
                    name,
                    script: e.new_script.clone(),
                    args: Vec::new(),
                    schedule: String::new(),
                    on: Vec::new(),
                    timeout_secs: 60,
                    access: Access::Read,
                    enabled: true,
                    next_run: None,
                });
                e.selected = Some(e.rows.len() - 1);
                e.new_name.clear();
                e.dirty = true;
                e.notice = Some("Added. Set its schedule or events, then save.".into());
            }
        }
    }
    Vec::new()
}
