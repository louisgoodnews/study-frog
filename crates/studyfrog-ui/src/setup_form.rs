//! The run setup form: its data, the validated conversion to a [`RunPlan`], presets and edits.
//! Pure: no egui, no I/O.

use studyfrog_core::plan::{
    ItemFilter, ItemKind, Order, RunPlan, ScheduleEffect, StateFilter, plan_hardest,
    plan_sequential, plan_shuffled, plan_spaced, plan_stalest,
};
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;

/// A one-click starting point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Spaced repetition: what is due, plus new items.
    Spaced,
    /// Every item once, in library order.
    EverythingInOrder,
    /// Every item once, shuffled.
    Shuffled,
    /// The hardest items first.
    Hardest,
    /// What has not been seen for the longest time.
    Stalest,
}

/// One change to the form.
#[derive(Debug, Clone, PartialEq)]
pub enum SetupEdit {
    /// Reset every field to a preset.
    Preset(Preset),
    /// Flip cards or type answers.
    Mode(RunMode),
    /// How the queue is ordered.
    Order(Order),
    /// Whether ratings change the schedule.
    Effect(ScheduleEffect),
    /// Repeat items rated Again.
    Requeue(bool),
    /// Put the chosen items in a random order.
    Shuffle(bool),
    /// Maximum number of items (empty: no limit).
    Limit(String),
    /// Include or exclude a kind.
    Kind(ItemKind, bool),
    /// Include or exclude a stack.
    Stack(Uid, bool),
    /// Include or exclude several stacks at once (a stack and the stacks inside it).
    Stacks(Vec<Uid>, bool),
    /// Comma-separated tags (any of).
    Tags(String),
    /// Learning-state filter.
    State(StateFilter),
    /// Turn the difficulty filter on or off.
    DifficultyOn(bool),
    /// Lowest difficulty (1 to 10).
    DiffMin(String),
    /// Highest difficulty (1 to 10).
    DiffMax(String),
    /// Turn the familiarity filter on or off.
    FamOn(bool),
    /// Only items less familiar than this many percent.
    FamBelow(String),
    /// Change the pacing.
    Pacing(crate::setup_pacing::PacingEdit),
}

/// The fields as the user edits them (text stays text until it is validated).
#[derive(Debug, Clone, PartialEq)]
pub struct SetupForm {
    /// Presentation mode.
    pub mode: RunMode,
    /// Queue order.
    pub order: Order,
    /// Effect on the schedule.
    pub effect: ScheduleEffect,
    /// Repeat items rated Again.
    pub requeue_again: bool,
    /// Shuffle the chosen items.
    pub shuffle: bool,
    /// Limit as typed.
    pub limit: String,
    /// Accepted kinds (empty: all).
    pub kinds: Vec<ItemKind>,
    /// Accepted stacks (empty: all).
    pub stacks: Vec<Uid>,
    /// Tags as typed, comma separated.
    pub tags: String,
    /// Learning state.
    pub state: StateFilter,
    /// Difficulty filter switched on.
    pub difficulty_on: bool,
    /// Lowest difficulty as typed.
    pub diff_min: String,
    /// Highest difficulty as typed.
    pub diff_max: String,
    /// Familiarity filter switched on.
    pub fam_on: bool,
    /// The familiarity bound as typed, in percent.
    pub fam_below: String,
    /// How the clock figures in the run.
    pub pacing: crate::setup_pacing::PacingForm,
}

fn plan_to_form(plan: &RunPlan, mode: RunMode) -> SetupForm {
    SetupForm {
        mode,
        order: plan.order,
        effect: plan.effect,
        requeue_again: plan.requeue_again,
        shuffle: plan.shuffle,
        limit: plan.limit.map(|l| l.to_string()).unwrap_or_default(),
        kinds: plan.filter.kinds.clone(),
        stacks: plan.filter.stacks.clone(),
        tags: plan.filter.tags_any.join(", "),
        state: plan.filter.state,
        difficulty_on: plan.filter.difficulty.is_some(),
        diff_min: "1".to_owned(),
        diff_max: "10".to_owned(),
        fam_on: plan.filter.familiarity_below.is_some(),
        pacing: crate::setup_pacing::form_of(plan.pacing),
        fam_below: plan
            .filter
            .familiarity_below
            .map_or_else(|| "60".to_owned(), |f| format!("{}", (f * 100.0).round())),
    }
}

/// The form for a preset; practice presets are practice-only, as documented in the rehearsal spec.
pub fn form_for_preset(preset: Preset, mode: RunMode) -> SetupForm {
    let plan = match preset {
        Preset::Spaced => plan_spaced(),
        Preset::EverythingInOrder => plan_sequential(),
        Preset::Shuffled => plan_shuffled(0),
        Preset::Hardest => plan_hardest(),
        Preset::Stalest => plan_stalest(),
    };
    plan_to_form(&plan, mode)
}

impl Default for SetupForm {
    fn default() -> Self {
        form_for_preset(Preset::Spaced, RunMode::Flip)
    }
}

pub(crate) fn number<T: std::str::FromStr>(text: &str, what: &str) -> Result<T, String> {
    text.trim()
        .parse()
        .map_err(|_| format!("{what} must be a number"))
}

fn limit(f: &SetupForm) -> Result<Option<u32>, String> {
    if f.limit.trim().is_empty() {
        return Ok(None);
    }
    match number::<u32>(&f.limit, "the limit")? {
        0 => Err("the limit must be at least 1".to_owned()),
        n => Ok(Some(n)),
    }
}

/// Validate the form and build the plan. `seed` drives shuffling.
pub fn plan_from_form(f: &SetupForm, seed: u64) -> Result<RunPlan, String> {
    let tags_any: Vec<String> = f
        .tags
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect();
    let filter = ItemFilter {
        kinds: f.kinds.clone(),
        stacks: f.stacks.clone(),
        tags_any,
        state: f.state,
        difficulty: crate::setup_fam::difficulty(f)?,
        familiarity_below: crate::setup_fam::familiarity_bound(f)?,
    };
    let plan = RunPlan {
        order: f.order,
        filter,
        effect: f.effect,
        shuffle: f.shuffle,
        seed,
        limit: limit(f)?,
        requeue_again: f.requeue_again,
        pacing: crate::setup_pacing::pacing_from(&f.pacing)?,
    };
    match f.mode {
        RunMode::Exam => studyfrog_core::exam::exam_plan(plan),
        _ => Ok(plan),
    }
}

fn toggle<T: PartialEq>(mut list: Vec<T>, item: T, on: bool) -> Vec<T> {
    list.retain(|x| *x != item);
    if on {
        list.push(item);
    }
    list
}

/// Apply one edit to the form.
pub fn apply_edit(f: SetupForm, edit: SetupEdit) -> SetupForm {
    match edit {
        // A preset sets how to rehearse; what to rehearse was chosen before and stays.
        SetupEdit::Preset(p) => SetupForm {
            stacks: f.stacks.clone(),
            kinds: f.kinds.clone(),
            tags: f.tags.clone(),
            ..form_for_preset(p, f.mode)
        },
        SetupEdit::Mode(mode) => SetupForm { mode, ..f },
        SetupEdit::Order(order) => SetupForm { order, ..f },
        SetupEdit::Effect(effect) => SetupForm { effect, ..f },
        SetupEdit::Requeue(requeue_again) => SetupForm { requeue_again, ..f },
        SetupEdit::Shuffle(shuffle) => SetupForm { shuffle, ..f },
        SetupEdit::Limit(limit) => SetupForm { limit, ..f },
        SetupEdit::Kind(kind, on) => SetupForm {
            kinds: toggle(f.kinds.clone(), kind, on),
            ..f
        },
        SetupEdit::Stack(uid, on) => SetupForm {
            stacks: toggle(f.stacks.clone(), uid, on),
            ..f
        },
        SetupEdit::Stacks(uids, on) => SetupForm {
            stacks: uids
                .into_iter()
                .fold(f.stacks.clone(), |l, u| toggle(l, u, on)),
            ..f
        },
        SetupEdit::Tags(tags) => SetupForm { tags, ..f },
        SetupEdit::State(state) => SetupForm { state, ..f },
        SetupEdit::DifficultyOn(difficulty_on) => SetupForm { difficulty_on, ..f },
        SetupEdit::DiffMin(diff_min) => SetupForm { diff_min, ..f },
        SetupEdit::DiffMax(diff_max) => SetupForm { diff_max, ..f },
        SetupEdit::FamOn(fam_on) => SetupForm { fam_on, ..f },
        SetupEdit::FamBelow(fam_below) => SetupForm { fam_below, ..f },
        SetupEdit::Pacing(e) => SetupForm {
            pacing: crate::setup_pacing::edited(f.pacing.clone(), e),
            ..f
        },
    }
}
