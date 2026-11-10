//! Persisted names of orders, effects, kinds and state filters.

use crate::plan::{ItemKind, Order, ScheduleEffect, StateFilter};

const ORDERS: [(Order, &str); 7] = [
    (Order::Due, "due"),
    (Order::Sequential, "sequential"),
    (Order::HardestFirst, "hardest_first"),
    (Order::EasiestFirst, "easiest_first"),
    (Order::StalestFirst, "stalest_first"),
    (Order::LowestRetrievability, "lowest_retrievability"),
    (Order::LeastFamiliar, "least_familiar"),
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
