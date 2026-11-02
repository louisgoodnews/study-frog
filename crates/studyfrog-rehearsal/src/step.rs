//! The run state machine: `Idle -> Running <-> Paused -> Finished | Aborted`.
//! `step` validates first and mutates second, so a refused input leaves the session unchanged.

use studyfrog_core::event::{EventKind, Latency, LatencyKind, RunEvent};
use studyfrog_core::plan::MAX_REQUEUES;
use studyfrog_core::review::Rating;
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_core::uid::Uid;

use crate::input::{Action, Input, StepError, StepOutput};
use crate::pacing::remaining_ms;
use crate::session::{ItemPhase, Session, current_item};

/// Apply one input.
pub fn step(session: &mut Session, input: Input) -> Result<StepOutput, StepError> {
    if input.t_ms < session.last_t_ms {
        return Err(StepError::TimeWentBackwards);
    }
    validate(session, &input.action)?;
    session.last_t_ms = input.t_ms;
    if timed_out(session, input.t_ms) {
        return Ok(time_up(session, input.t_ms));
    }
    Ok(apply(session, input))
}

fn validate(s: &Session, action: &Action) -> Result<(), StepError> {
    match (s.run.state, action) {
        (RunState::Finished | RunState::Aborted, _) => Err(StepError::Ended),
        (RunState::Idle, Action::Start | Action::Abort) => Ok(()),
        (RunState::Idle, _) => Err(StepError::NotStarted),
        (_, Action::Start) => Err(StepError::AlreadyStarted),
        (RunState::Paused, Action::Resume | Action::Abort | Action::Tick) => Ok(()),
        (RunState::Paused, _) => Err(StepError::Paused),
        (_, Action::Resume) => Err(StepError::NotPaused),
        (_, Action::Pause | Action::Abort | Action::Tick) => Ok(()),
        (_, Action::Flip | Action::Skip) => {
            require_phase(s, ItemPhase::Shown, StepError::NotAwaitingReveal)
        }
        (_, Action::SubmitAnswer(_)) => require_answer_mode(s),
        (_, Action::Rate(_)) => require_phase(s, ItemPhase::Revealed, StepError::NotRevealed),
        (_, Action::Mark(_)) if s.phase == ItemPhase::NoItem => Err(StepError::NoItemShown),
        (_, Action::Mark(_)) => Ok(()),
    }
}

fn require_phase(s: &Session, want: ItemPhase, err: StepError) -> Result<(), StepError> {
    if s.phase == want { Ok(()) } else { Err(err) }
}

fn require_answer_mode(s: &Session) -> Result<(), StepError> {
    if s.run.mode != RunMode::Answer {
        return Err(StepError::WrongMode);
    }
    require_phase(s, ItemPhase::Shown, StepError::NotAwaitingReveal)
}

fn event(s: &mut Session, t_ms: u32, item: Option<Uid>, kind: EventKind) -> RunEvent {
    let seq = s.next_seq;
    s.next_seq += 1;
    RunEvent {
        run: s.run.uid,
        seq,
        t_ms,
        item,
        kind,
    }
}

/// Time the current item has been on screen, without paused time.
fn item_latency(s: &Session, t_ms: u32) -> u32 {
    t_ms.saturating_sub(s.shown_at_ms)
        .saturating_sub(s.paused_in_item_ms)
}

fn apply(s: &mut Session, input: Input) -> StepOutput {
    let t = input.t_ms;
    let mut out = StepOutput::default();
    let item = current_item(s);
    match input.action {
        Action::Start => start(s, t, &mut out),
        Action::Flip => reveal(
            s,
            t,
            item,
            LatencyKind::Flip,
            EventKind::ItemFlipped,
            &mut out,
        ),
        Action::SubmitAnswer(text) => reveal(
            s,
            t,
            item,
            LatencyKind::Answer,
            EventKind::AnswerSubmitted(text),
            &mut out,
        ),
        Action::Skip => skip(s, t, item, &mut out),
        Action::Rate(rating) => rate(s, t, item, rating, &mut out),
        Action::Mark(d) => out.events.push(event(s, t, item, EventKind::ItemMarked(d))),
        Action::Tick => {}
        Action::Pause => pause(s, t, &mut out),
        Action::Resume => resume(s, t, &mut out),
        Action::Abort => abort(s, t, &mut out),
    }
    out
}

fn start(s: &mut Session, t: u32, out: &mut StepOutput) {
    s.run.state = RunState::Running;
    s.started_ms = t;
    out.events.push(event(s, t, None, EventKind::RunStarted));
    show_next(s, t, out);
}

/// Show the item at the cursor, or finish the run when the queue is exhausted.
fn show_next(s: &mut Session, t: u32, out: &mut StepOutput) {
    match s.queue.get(s.cursor).copied() {
        Some(item) => {
            s.phase = ItemPhase::Shown;
            s.shown_at_ms = t;
            s.paused_in_item_ms = 0;
            out.events
                .push(event(s, t, Some(item), EventKind::ItemShown));
        }
        None => {
            s.phase = ItemPhase::NoItem;
            s.run.state = RunState::Finished;
            out.events.push(event(s, t, None, EventKind::RunFinished));
        }
    }
}

fn reveal(
    s: &mut Session,
    t: u32,
    item: Option<Uid>,
    kind: LatencyKind,
    what: EventKind,
    out: &mut StepOutput,
) {
    s.phase = ItemPhase::Revealed;
    out.latency = item.map(|uid| Latency {
        item: uid,
        kind,
        ms: item_latency(s, t),
    });
    out.events.push(event(s, t, item, what));
}

fn skip(s: &mut Session, t: u32, item: Option<Uid>, out: &mut StepOutput) {
    out.latency = item.map(|uid| Latency {
        item: uid,
        kind: LatencyKind::Skip,
        ms: item_latency(s, t),
    });
    out.events.push(event(s, t, item, EventKind::ItemSkipped));
    s.cursor += 1;
    show_next(s, t, out);
}

fn rate(s: &mut Session, t: u32, item: Option<Uid>, rating: Rating, out: &mut StepOutput) {
    out.events
        .push(event(s, t, item, EventKind::ItemRated(rating)));
    if let Some(uid) = item.filter(|_| rating == Rating::Again && s.run.plan.requeue_again) {
        requeue(s, uid);
    }
    s.cursor += 1;
    show_next(s, t, out);
}

/// Put `uid` at the end of the queue unless it already came back `MAX_REQUEUES` times.
fn requeue(s: &mut Session, uid: Uid) {
    let count = s.requeues.entry(uid).or_insert(0);
    if *count < MAX_REQUEUES {
        *count += 1;
        s.queue.push(uid);
    }
}

fn pause(s: &mut Session, t: u32, out: &mut StepOutput) {
    s.run.state = RunState::Paused;
    s.paused_at_ms = Some(t);
    out.events.push(event(s, t, None, EventKind::RunPaused));
}

fn resume(s: &mut Session, t: u32, out: &mut StepOutput) {
    let paused = s.paused_at_ms.take().map_or(0, |at| t.saturating_sub(at));
    s.paused_in_item_ms = s.paused_in_item_ms.saturating_add(paused);
    s.paused_total_ms = s.paused_total_ms.saturating_add(paused);
    s.run.state = RunState::Running;
    out.events.push(event(s, t, None, EventKind::RunResumed));
}

fn abort(s: &mut Session, t: u32, out: &mut StepOutput) {
    s.phase = ItemPhase::NoItem;
    s.run.state = RunState::Aborted;
    out.events.push(event(s, t, None, EventKind::RunAborted));
}

/// A countdown whose active time has run out while the run was running.
fn timed_out(s: &Session, t: u32) -> bool {
    s.run.state == RunState::Running && remaining_ms(s, t) == Some(0)
}

/// End a countdown: the item on screen is not answered (no rating, no schedule change) and the run is finished.
fn time_up(s: &mut Session, t: u32) -> StepOutput {
    let mut out = StepOutput::default();
    let open = current_item(s);
    out.events.push(event(s, t, open, EventKind::TimeUp));
    s.phase = ItemPhase::NoItem;
    s.run.state = RunState::Finished;
    out.events.push(event(s, t, None, EventKind::RunFinished));
    out
}
