//! Update logic of a rehearsal run. The pure engine (`step`) is called right here; the shell only
//! supplies the time (`State::now`) and executes the effects that save progress.
use crate::effect::EffectRehearsal;
use crate::effect::EffectReports;
use crate::msg::MsgRehearsal;

pub(crate) mod choice;
mod status;
mod timing;

pub use status::{awaiting_rating, awaiting_reveal, is_over};

use studyfrog_core::run::{Run, RunState};
use studyfrog_core::settings::AnswerEngine;
use studyfrog_rehearsal::input::{Action, Input};
use studyfrog_rehearsal::learn_graph::LearnGraph;
use studyfrog_rehearsal::session::{ItemPhase, current_item, session_new};
use studyfrog_rehearsal::step::step;
use studyfrog_service::rehearse::step_context;

use crate::effect::Effect;
use crate::msg::Msg;
use crate::run_state::{ItemContent, LOOKAHEAD, RunView, engine_ms, run_view_new};
use crate::setup::{Rehearse, plan_from_form, refresh_preview};
use crate::state::{Notice, NoticeKind, State};
use timing::rating_timing;

pub(crate) fn error(state: State, text: impl Into<String>) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text: text.into(),
    });
    (State { notice, ..state }, Vec::new())
}

/// Handle the run messages.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    match msg {
        Msg::Rehearsal(MsgRehearsal::StartRunClicked) => start_clicked(state),
        Msg::Rehearsal(MsgRehearsal::RunReady(result)) => run_ready(state, result),
        Msg::Rehearsal(MsgRehearsal::RunAction(action)) => act(state, action),
        Msg::Rehearsal(MsgRehearsal::AnswerEdited(text)) => {
            with_run(state, |r| RunView { answer: text, ..r })
        }
        Msg::Rehearsal(MsgRehearsal::ItemLoaded(uid, result)) => item_loaded(state, uid, result),
        Msg::Rehearsal(MsgRehearsal::AnswerScored(uid, result)) => {
            crate::run_results::answer_scored(state, uid, result)
        }
        Msg::Rehearsal(MsgRehearsal::ApplySuggestion { item, expected, to }) => (
            state,
            vec![Effect::Rehearsal(EffectRehearsal::Rerate {
                item,
                expected,
                to,
            })],
        ),
        Msg::Rehearsal(MsgRehearsal::Rerated(uid, result)) => {
            crate::run_results::rerated(state, uid, result)
        }
        Msg::Rehearsal(MsgRehearsal::StepRecorded(Err(e))) => {
            error(state, format!("Could not save your progress: {e}"))
        }
        Msg::Rehearsal(MsgRehearsal::LeaveRun) => leave(state),
        _ => (state, Vec::new()),
    }
}

fn with_run(state: State, f: impl FnOnce(RunView) -> RunView) -> (State, Vec<Effect>) {
    let mut state = state;
    state.rehearse.run = state.rehearse.run.take().map(f);
    (state, Vec::new())
}

fn start_clicked(state: State) -> (State, Vec<Effect>) {
    if state.rehearse.starting || state.rehearse.run.is_some() {
        return (state, Vec::new());
    }
    let form = &state.rehearse.form;
    match plan_from_form(form, 0) {
        Ok(plan) => {
            let effect = Effect::Rehearsal(EffectRehearsal::StartRun {
                plan,
                mode: form.mode,
            });
            let rehearse = Rehearse {
                starting: true,
                ..state.rehearse.clone()
            };
            (
                State {
                    rehearse,
                    notice: None,
                    ..state
                },
                vec![effect],
            )
        }
        Err(reason) => error(state, reason),
    }
}

fn run_ready(state: State, result: Result<Run, String>) -> (State, Vec<Effect>) {
    let mut state = state;
    state.rehearse.starting = false;
    match result {
        Ok(run) => begin_run(state, run, None),
        Err(e) => error(state, format!("Could not start the run: {e}")),
    }
}

/// Show a new run and start it. A custom learn-mode run carries its graph, which decides its queue.
pub(crate) fn begin_run(state: State, run: Run, learn: Option<LearnGraph>) -> (State, Vec<Effect>) {
    let mut state = state;
    state.rehearse.starting = false;
    // A new run takes the page back from a memory game that was left on screen.
    state.rehearse.memory = None;
    let mut session = session_new(run);
    session.learn = learn.map(studyfrog_rehearsal::learn_path::new_path);
    let mut view = run_view_new(session, state.now);
    view.reading =
        studyfrog_rehearsal::reading::reading_params(state.app.reading_wpm, &state.app.reading);
    state.rehearse.run = Some(view);
    state.chrome.zen = None; // a new run follows the Zen setting again
    act(state, Action::Start)
}

/// Feed one action to the engine. Actions the engine refuses (a rating before the answer is
/// revealed, a second flip, ...) are ignored: they are stray key presses, not errors.
pub(crate) fn act(state: State, action: Action) -> (State, Vec<Effect>) {
    let mut state = state;
    let Some(mut run) = state.rehearse.run.take() else {
        return (state, Vec::new());
    };
    let t_ms = engine_ms(state.now, run.started_time, run.session.last_t_ms);
    let rated_item = current_item(&run.session);
    // Read before a rating clears the typed answer: the score is the item's semantic judgement.
    let judged = run.scored.as_ref().map(|s| s.score.value);
    let clears_answer = matches!(action, Action::Rate(_) | Action::Skip)
        || (crate::exam_ui::is_exam(&run) && matches!(action, Action::SubmitAnswer(_)));
    let mut effects = Vec::new();
    let submitted = match &action {
        Action::SubmitAnswer(text) => current_item(&run.session).map(|item| {
            Effect::Answer(crate::run_state::AnswerEffect::Score {
                run: run.session.run.uid,
                item,
                answer: (
                    text.clone(),
                    run.content.get(&item).and_then(|c| c.expected.clone()),
                ),
                words_only: state.app.answer_engine == AnswerEngine::Words,
            })
        }),
        _ => None,
    };
    if let Ok(out) = step(&mut run.session, Input { t_ms, action }) {
        let dwell = out.dwell_ms;
        effects.extend(submitted);
        run.events.extend(out.events.iter().cloned());
        run.latencies.extend(out.latency);
        if clears_answer {
            run.answer.clear();
            run.case_run = Default::default();
            run.scored = None;
        }
        effects.extend(prefetch(&mut run));
        if is_over(&run) {
            effects.push(Effect::Reports(EffectReports::CheckAchievements));
            if crate::exam_ui::is_exam(&run) {
                effects.extend(crate::exam_ui::load_rest(&mut run));
                effects.extend(crate::exam_ui::grade_when_ready(&mut run));
            }
            if run.diary_prompt.is_none() {
                run.diary_prompt = Some(crate::diary_prompt::prompt_new(&state, &run));
            }
        }
        let mut ctx = step_context(&run.session);
        ctx.timing = rated_item.and_then(|item| rating_timing(&run, item, (dwell, judged)));
        effects.push(Effect::Rehearsal(EffectRehearsal::RecordStep { ctx, out }));
    }
    state.rehearse.run = Some(run);
    (state, effects)
}

/// Request the content of the current item and the next few that are not loaded yet.
fn prefetch(run: &mut RunView) -> Vec<Effect> {
    let upcoming: Vec<_> = run
        .session
        .queue
        .iter()
        .skip(run.session.cursor)
        .take(LOOKAHEAD)
        .copied()
        .collect();
    upcoming
        .into_iter()
        .filter(|uid| run.requested.insert(*uid))
        .map(|u| Effect::Rehearsal(EffectRehearsal::LoadItem(u)))
        .collect()
}

fn item_loaded(
    state: State,
    uid: studyfrog_core::uid::Uid,
    result: Result<ItemContent, String>,
) -> (State, Vec<Effect>) {
    let mut state = state;
    let content = match result {
        Ok(c) => c,
        Err(e) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not load an item: {e}"),
            });
            ItemContent {
                title: String::new(),
                prompt: format!("(this item could not be loaded: {e})"),
                ..Default::default()
            }
        }
    };
    if let Some(run) = &mut state.rehearse.run {
        run.blocks
            .insert(uid, crate::run_state::blocks_of(&content));
        choice::note_item(run, uid, &content);
        run.content.insert(uid, content);
    }
    (state, Vec::new())
}

fn leave(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    match &state.rehearse.run {
        Some(run) if is_over(run) => {
            state.rehearse.run = None;
            refresh_preview(state)
        }
        _ => (state, Vec::new()),
    }
}
