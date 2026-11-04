//! Post-rehearsal diary prompt data and logic. Pure data and free functions.
//! See Section 5 of `docs/specs/diary.md`.

use std::collections::BTreeMap;

use studyfrog_core::civil::Date;
use studyfrog_core::diary::RatingValue;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::pacing::run_active_ms;
use studyfrog_rehearsal::reading::item_records;

use crate::diary_types::{DiaryEditDraft, DiaryEffect};
use crate::effect::Effect;
use crate::run_state::RunView;
use crate::state::{Screen, State};
use crate::view::date_picker::today_of;

/// Post-rehearsal prompt state displayed on the run summary screen.
#[derive(Debug, Clone, PartialEq)]
pub struct RunDiaryPrompt {
    /// Civil date of the entry.
    pub date: Date,
    /// Pre-filled study minutes.
    pub study_minutes: u32,
    /// Associated rehearsal run ULID.
    pub run_uid: Uid,
    /// Associated stack references.
    pub stacks: Vec<String>,
    /// Items reviewed in this run.
    pub items: Vec<Uid>,
    /// Selected ratings by scale ID.
    pub ratings: BTreeMap<String, RatingValue>,
    /// One-line quick reflection or takeaway.
    pub note: String,
    /// Whether the user skipped this prompt.
    pub skipped: bool,
    /// Whether saving has been initiated.
    pub saved: bool,
}

/// Create a new prompt pre-filled from the state and the finished run.
pub fn prompt_new(state: &State, run: &RunView) -> RunDiaryPrompt {
    RunDiaryPrompt {
        date: today_of(state),
        study_minutes: run_study_minutes(run),
        run_uid: run.session.run.uid,
        stacks: run_stacks(state, run),
        items: reviewed_items(run),
        ratings: BTreeMap::new(),
        note: String::new(),
        skipped: false,
        saved: false,
    }
}

/// Pre-filled duration in minutes from the rehearsal clock.
pub fn run_study_minutes(run: &RunView) -> u32 {
    let active_ms = run_active_ms(&run.events);
    let total_ms = run.events.last().map_or(0, |e| e.t_ms);
    let ms = if active_ms > 0 { active_ms } else { total_ms };
    let mins = ((f64::from(ms)) / 60_000.0).round() as u32;
    if mins == 0 && ms > 0 { 1 } else { mins }
}

/// Stacks rehearsed in this run, formatted as titles or ULIDs.
pub fn run_stacks(state: &State, run: &RunView) -> Vec<String> {
    let mut uids = run.session.run.plan.filter.stacks.clone();
    if uids.is_empty() {
        uids.clone_from(&state.rehearse.form.stacks);
    }
    uids.into_iter()
        .map(|uid| {
            state
                .vault
                .as_ref()
                .and_then(|v| v.overview.stacks.iter().find(|s| s.uid == uid))
                .map_or_else(|| format_ulid(uid), |s| s.title.clone())
        })
        .collect()
}

/// Items reviewed in this run.
pub fn reviewed_items(run: &RunView) -> Vec<Uid> {
    let mut items: Vec<Uid> = item_records(&run.events)
        .into_iter()
        .map(|r| r.item)
        .collect();
    if items.is_empty() {
        items.clone_from(&run.session.queue);
    }
    items
}

/// Convert prompt data into a draft for the diary pipeline.
pub fn prompt_draft(prompt: &RunDiaryPrompt) -> DiaryEditDraft {
    DiaryEditDraft {
        uid: None,
        path: None,
        date: prompt.date,
        title: String::new(),
        study_minutes: Some(prompt.study_minutes),
        tags: String::new(),
        ratings: prompt.ratings.clone(),
        stacks: prompt.stacks.join(", "),
        items: prompt.items.clone(),
        runs: vec![prompt.run_uid],
        exams: Vec::new(),
        content: prompt.note.clone(),
        is_dirty: true,
    }
}

fn ensure_prompt(state: &mut State) -> Option<&mut RunDiaryPrompt> {
    if let Some(run) = &state.rehearse.run
        && run.diary_prompt.is_none()
    {
        let prompt = prompt_new(state, run);
        state.rehearse.run.as_mut()?.diary_prompt = Some(prompt);
    }
    state.rehearse.run.as_mut()?.diary_prompt.as_mut()
}

/// Update a rating in the active prompt.
pub fn update_rating(
    mut state: State,
    scale: String,
    val: Option<RatingValue>,
) -> (State, Vec<Effect>) {
    if let Some(p) = ensure_prompt(&mut state) {
        match val {
            Some(v) => {
                p.ratings.insert(scale, v);
            }
            None => {
                p.ratings.remove(&scale);
            }
        }
    }
    (state, Vec::new())
}

/// Update the reflection note text in the active prompt.
pub fn update_note(mut state: State, note: String) -> (State, Vec<Effect>) {
    if let Some(p) = ensure_prompt(&mut state) {
        p.note = note;
    }
    (state, Vec::new())
}

/// Skip and dismiss the prompt.
pub fn update_skip(mut state: State) -> (State, Vec<Effect>) {
    if let Some(p) = ensure_prompt(&mut state) {
        p.skipped = true;
    }
    (state, Vec::new())
}

/// Submit the prompt as a diary entry.
pub fn update_submit(mut state: State) -> (State, Vec<Effect>) {
    if let Some(p) = ensure_prompt(&mut state) {
        p.saved = true;
        let draft = prompt_draft(p);
        (
            state,
            vec![Effect::Diary(DiaryEffect::SaveEntry(Box::new(draft)))],
        )
    } else {
        (state, Vec::new())
    }
}

/// Expand the prompt into the full diary editor.
pub fn update_expand(mut state: State) -> (State, Vec<Effect>) {
    if let Some(mut run) = state.rehearse.run.take() {
        let p = run
            .diary_prompt
            .take()
            .unwrap_or_else(|| prompt_new(&state, &run));
        let draft = prompt_draft(&p);
        state.diary.active_entry = Some(draft);
        state.screen = Screen::Diary;
    }
    (state, Vec::new())
}
