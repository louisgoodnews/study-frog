//! Update logic of a rehearsal run. The pure engine (`step`) is called right here; the shell only
//! supplies the time (`State::now`) and executes the effects that save progress.

use studyfrog_core::run::{Run, RunState};
use studyfrog_core::settings::AnswerEngine;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::{Action, Input};
use studyfrog_rehearsal::reading::{expected_ms, item_records, word_count};
use studyfrog_rehearsal::session::{ItemPhase, current_item, session_new};
use studyfrog_rehearsal::step::step;
use studyfrog_service::rehearse::{RatingTiming, step_context};

use crate::effect::Effect;
use crate::msg::Msg;
use crate::run_state::{ItemContent, LOOKAHEAD, RunView, engine_ms, run_view_new};
use crate::setup::{Rehearse, plan_from_form, refresh_preview};
use crate::state::{Notice, NoticeKind, State};

fn error(state: State, text: impl Into<String>) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text: text.into(),
    });
    (State { notice, ..state }, Vec::new())
}

/// Handle the run messages.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    match msg {
        Msg::StartRunClicked => start_clicked(state),
        Msg::RunReady(result) => run_ready(state, result),
        Msg::RunAction(action) => act(state, action),
        Msg::AnswerEdited(text) => with_run(state, |r| RunView { answer: text, ..r }),
        Msg::ItemLoaded(uid, result) => item_loaded(state, uid, result),
        Msg::AnswerScored(uid, result) => crate::run_results::answer_scored(state, uid, result),
        Msg::ApplySuggestion { item, expected, to } => {
            (state, vec![Effect::Rerate { item, expected, to }])
        }
        Msg::Rerated(uid, result) => crate::run_results::rerated(state, uid, result),
        Msg::StepRecorded(Err(e)) => error(state, format!("Could not save your progress: {e}")),
        Msg::LeaveRun => leave(state),
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
            let effect = Effect::StartRun {
                plan,
                mode: form.mode,
            };
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
        Ok(run) => {
            let mut view = run_view_new(session_new(run), state.now);
            view.reading = studyfrog_rehearsal::reading::reading_params(
                state.app.reading_wpm,
                &state.app.reading,
            );
            state.rehearse.run = Some(view);
            act(state, Action::Start)
        }
        Err(e) => error(state, format!("Could not start the run: {e}")),
    }
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
    let clears_answer = matches!(action, Action::Rate(_) | Action::Skip);
    let mut effects = Vec::new();
    let submitted = match &action {
        Action::SubmitAnswer(text) => current_item(&run.session).map(|item| Effect::ScoreAnswer {
            run: run.session.run.uid,
            item,
            answer: text.clone(),
            words_only: state.app.answer_engine == AnswerEngine::Words,
        }),
        _ => None,
    };
    if let Ok(out) = step(&mut run.session, Input { t_ms, action }) {
        effects.extend(submitted);
        run.events.extend(out.events.iter().cloned());
        run.latencies.extend(out.latency);
        if clears_answer {
            run.answer.clear();
            run.scored = None;
        }
        effects.extend(prefetch(&mut run));
        if is_over(&run) {
            effects.push(Effect::CheckAchievements);
        }
        let mut ctx = step_context(&run.session);
        ctx.timing = rated_item.and_then(|item| rating_timing(&run, item));
        effects.push(Effect::RecordStep { ctx, out });
    }
    state.rehearse.run = Some(run);
    (state, effects)
}

/// How long the answer of `item` was on screen before it was rated and how long its words should take to read.
fn rating_timing(run: &RunView, item: Uid) -> Option<RatingTiming> {
    let record = item_records(&run.events)
        .into_iter()
        .find(|r| r.item == item)?;
    let answer_ms = record.rated.and(record.read_ms)?;
    let words = word_count(&run.content.get(&item)?.answer);
    (words > 0).then(|| RatingTiming {
        answer_ms,
        expected_ms: expected_ms(words, &run.reading),
    })
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
        .map(Effect::LoadItem)
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
                answer: String::new(),
                previous: None,
                source: None,
            }
        }
    };
    if let Some(run) = &mut state.rehearse.run {
        run.blocks
            .insert(uid, crate::run_state::blocks_of(&content));
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

/// True once the run has finished or been aborted.
pub fn is_over(run: &RunView) -> bool {
    matches!(
        run.session.run.state,
        RunState::Finished | RunState::Aborted
    )
}

/// True while the current item is on screen and can be answered or skipped.
pub fn awaiting_reveal(run: &RunView) -> bool {
    run.session.run.state == RunState::Running && run.session.phase == ItemPhase::Shown
}

/// True while the answer is revealed and a rating is expected.
pub fn awaiting_rating(run: &RunView) -> bool {
    run.session.run.state == RunState::Running && run.session.phase == ItemPhase::Revealed
}
