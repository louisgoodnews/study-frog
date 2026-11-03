//! Admission filters. An empty list means no restriction on that axis.

use studyfrog_core::plan::{ItemFilter, StateFilter};
use studyfrog_core::time::UnixMs;

use crate::queue::candidate::{Candidate, is_due, is_reviewed};

/// Case-insensitive (Unicode simple lowercase) equality without allocating.
fn same_tag(a: &str, b: &str) -> bool {
    a.chars()
        .flat_map(char::to_lowercase)
        .eq(b.chars().flat_map(char::to_lowercase))
}

fn state_ok(state: StateFilter, c: &Candidate, now: UnixMs) -> bool {
    match state {
        StateFilter::Any => true,
        StateFilter::New => !is_reviewed(c),
        StateFilter::Learned => is_reviewed(c),
        StateFilter::Due => is_due(c, now),
        StateFilter::NotDue => is_reviewed(c) && !is_due(c, now),
    }
}

fn tags_ok(wanted: &[String], c: &Candidate) -> bool {
    wanted.is_empty() || c.tags.iter().any(|t| wanted.iter().any(|w| same_tag(t, w)))
}

fn difficulty_ok(filter: &ItemFilter, c: &Candidate) -> bool {
    filter.difficulty.is_none_or(|r| {
        c.memory
            .is_some_and(|m| (r.min..=r.max).contains(&m.difficulty))
    })
}

/// Items with no history count as unknown, which is less familiar than any bound.
fn familiarity_ok(filter: &ItemFilter, c: &Candidate) -> bool {
    filter
        .familiarity_below
        .is_none_or(|bound| c.familiarity.is_none_or(|f| f < bound))
}

/// Whether `c` passes every predicate of `filter`.
pub fn admits(filter: &ItemFilter, c: &Candidate, now: UnixMs) -> bool {
    (filter.kinds.is_empty() || filter.kinds.contains(&c.kind))
        && (filter.stacks.is_empty()
            || filter.stacks.contains(&c.stack)
            || c.also_in.iter().any(|s| filter.stacks.contains(s)))
        && tags_ok(&filter.tags_any, c)
        && state_ok(filter.state, c, now)
        && difficulty_ok(filter, c)
        && familiarity_ok(filter, c)
}
