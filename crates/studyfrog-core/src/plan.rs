//! Run plans: how a rehearsal queue is built and what a run does to the schedule.
//! Plain data; queue building lives in `studyfrog-rehearsal`. See `docs/specs/rehearsal-run.md`.

use crate::uid::Uid;

/// The studiable kinds (stacks are containers, not studied).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    /// A note (its title can be rehearsed).
    Note,
    /// A flashcard.
    Card,
    /// A free-text question.
    Question,
}

/// How the queue is sequenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Order {
    /// Spaced repetition: most overdue first, then new items, within daily budgets.
    Due,
    /// Library order, every filtered item once.
    Sequential,
    /// Highest FSRS difficulty first.
    HardestFirst,
    /// Lowest FSRS difficulty first.
    EasiestFirst,
    /// Least recently reviewed first (never reviewed first of all).
    StalestFirst,
    /// Lowest predicted recall first.
    LowestRetrievability,
}

/// Which learning state an item must be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateFilter {
    /// No restriction.
    Any,
    /// Never reviewed.
    New,
    /// Reviewed at least once.
    Learned,
    /// Reviewed and due now.
    Due,
    /// Reviewed and not yet due.
    NotDue,
}

/// Inclusive FSRS difficulty range (difficulty runs from 1 to 10).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DifficultyRange {
    /// Lowest accepted difficulty.
    pub min: f32,
    /// Highest accepted difficulty.
    pub max: f32,
}

/// Which items may enter the queue. An empty list means no restriction on that axis.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemFilter {
    /// Accepted kinds.
    pub kinds: Vec<ItemKind>,
    /// Accepted stacks.
    pub stacks: Vec<Uid>,
    /// Item must carry at least one of these tags (compared case-insensitively).
    pub tags_any: Vec<String>,
    /// Learning state.
    pub state: StateFilter,
    /// Difficulty range; items without an FSRS state are excluded when set.
    pub difficulty: Option<DifficultyRange>,
}

/// What a run does to the schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScheduleEffect {
    /// Ratings feed the scheduler and are written as reviews.
    UpdateSchedule,
    /// Events and latencies are recorded; the schedule is untouched.
    PracticeOnly,
}

/// Everything needed to build and run a queue.
#[derive(Debug, Clone, PartialEq)]
pub struct RunPlan {
    /// Sequencing.
    pub order: Order,
    /// Admission filter.
    pub filter: ItemFilter,
    /// Effect on the schedule.
    pub effect: ScheduleEffect,
    /// Put the chosen items in a random order (seeded, reproducible). Independent of the order: a
    /// spaced run can be shuffled too, and the shuffle never changes which items are chosen, except
    /// for a sequential run with a limit, where it picks a random sample.
    pub shuffle: bool,
    /// Seed for shuffling and random tie-breaks.
    pub seed: u64,
    /// Cap on the queue length.
    pub limit: Option<u32>,
    /// Rated-Again items return to the end of the queue.
    pub requeue_again: bool,
}

/// How often one item may be requeued within a run.
pub const MAX_REQUEUES: u8 = 3;

/// A filter that admits everything.
pub fn filter_all() -> ItemFilter {
    ItemFilter {
        kinds: Vec::new(),
        stacks: Vec::new(),
        tags_any: Vec::new(),
        state: StateFilter::Any,
        difficulty: None,
    }
}

fn plan(order: Order, effect: ScheduleEffect, requeue_again: bool) -> RunPlan {
    RunPlan {
        order,
        filter: filter_all(),
        effect,
        shuffle: false,
        seed: 0,
        limit: None,
        requeue_again,
    }
}

/// Spaced repetition: due-first, updates the schedule, lapses come back in the same run.
pub fn plan_spaced() -> RunPlan {
    plan(Order::Due, ScheduleEffect::UpdateSchedule, true)
}

/// Run through every item once in library order.
pub fn plan_sequential() -> RunPlan {
    plan(Order::Sequential, ScheduleEffect::PracticeOnly, false)
}

/// Run through every item once in a reproducible random order.
pub fn plan_shuffled(seed: u64) -> RunPlan {
    RunPlan {
        seed,
        shuffle: true,
        ..plan(Order::Sequential, ScheduleEffect::PracticeOnly, false)
    }
}

/// Drill the hardest items first.
pub fn plan_hardest() -> RunPlan {
    plan(Order::HardestFirst, ScheduleEffect::PracticeOnly, true)
}

/// Review what has not been seen for the longest time.
pub fn plan_stalest() -> RunPlan {
    plan(Order::StalestFirst, ScheduleEffect::PracticeOnly, false)
}

const ORDERS: [(Order, &str); 6] = [
    (Order::Due, "due"),
    (Order::Sequential, "sequential"),
    (Order::HardestFirst, "hardest_first"),
    (Order::EasiestFirst, "easiest_first"),
    (Order::StalestFirst, "stalest_first"),
    (Order::LowestRetrievability, "lowest_retrievability"),
];

/// Persisted name of an order.
pub fn order_name(order: Order) -> &'static str {
    ORDERS
        .iter()
        .find(|(o, _)| *o == order)
        .map_or("due", |(_, n)| n)
}

/// Inverse of [`order_name`]. The old name `shuffled` (before shuffling became a separate switch)
/// means a sequential order.
pub fn order_from_name(name: &str) -> Option<Order> {
    ORDERS
        .iter()
        .find(|(_, n)| *n == name)
        .map(|(o, _)| *o)
        .or_else(|| (name == "shuffled").then_some(Order::Sequential))
}

/// Persisted name of an effect.
pub fn effect_name(effect: ScheduleEffect) -> &'static str {
    match effect {
        ScheduleEffect::UpdateSchedule => "update_schedule",
        ScheduleEffect::PracticeOnly => "practice_only",
    }
}

/// Inverse of [`effect_name`].
pub fn effect_from_name(name: &str) -> Option<ScheduleEffect> {
    match name {
        "update_schedule" => Some(ScheduleEffect::UpdateSchedule),
        "practice_only" => Some(ScheduleEffect::PracticeOnly),
        _ => None,
    }
}

/// Persisted name of an item kind.
pub fn item_kind_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Note => "note",
        ItemKind::Card => "card",
        ItemKind::Question => "question",
    }
}

/// Inverse of [`item_kind_name`].
pub fn item_kind_from_name(name: &str) -> Option<ItemKind> {
    [ItemKind::Note, ItemKind::Card, ItemKind::Question]
        .into_iter()
        .find(|k| item_kind_name(*k) == name)
}

const STATES: [(StateFilter, &str); 5] = [
    (StateFilter::Any, "any"),
    (StateFilter::New, "new"),
    (StateFilter::Learned, "learned"),
    (StateFilter::Due, "due"),
    (StateFilter::NotDue, "not_due"),
];

/// Persisted name of a state filter.
pub fn state_filter_name(state: StateFilter) -> &'static str {
    STATES
        .iter()
        .find(|(s, _)| *s == state)
        .map_or("any", |(_, n)| n)
}

/// Inverse of [`state_filter_name`].
pub fn state_filter_from_name(name: &str) -> Option<StateFilter> {
    STATES.iter().find(|(_, n)| *n == name).map(|(s, _)| *s)
}
