//! The search bar of the library: filter fields (Standard) or a query (Query), and the way between them. Pure.

use studyfrog_core::item_key::is_key;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::Priority;
use studyfrog_core::query::{Cond, Field, Op, Order, Query, QueryError, Val};
use studyfrog_core::query_complete::{apply, complete};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::query_print::print_query;
use studyfrog_core::status::Status;

use crate::state::{Notice, NoticeKind, State};

/// Which way the bar is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    /// Fields for the common filters.
    #[default]
    Standard,
    /// A query typed out.
    Query,
}

/// The bar. The title word of Standard mode lives in the browser (`browser.search`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchState {
    /// The mode in use.
    pub mode: SearchMode,
    /// Only this kind.
    pub kind: Option<ItemKind>,
    /// Only this status.
    pub status: Option<Status>,
    /// Only this priority.
    pub priority: Option<Priority>,
    /// The text of Query mode.
    pub query: String,
}

/// What happened on the bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMsg {
    /// Use Standard (`false`) or Query (`true`), carrying what is set over.
    Mode(bool),
    /// Filter by kind.
    Kind(Option<ItemKind>),
    /// Filter by status.
    Status(Option<Status>),
    /// Filter by priority.
    Priority(Option<Priority>),
    /// The query text was edited.
    Query(String),
    /// Take the n-th suggestion for the query text.
    Complete(usize),
}

/// The query the fields stand for.
pub fn standard_query(word: &str, s: &SearchState, sort: &[Order]) -> Query {
    let word = word.trim();
    let mut conds = Vec::new();
    // A word that is a key (`FLASHCARD_12`) finds the item with that key.
    let field = if is_key(word) {
        (Field::Key, Op::Eq)
    } else {
        (Field::Title, Op::Contains)
    };
    if !word.is_empty() {
        conds.push(Cond::Cmp(field.0, field.1, Val::Text(word.to_owned())));
    }
    let picks = [
        s.kind.map(|k| (Field::Type, Val::Kind(k))),
        s.status.map(|v| (Field::Status, Val::Status(v))),
        s.priority.map(|p| (Field::Priority, Val::Priority(p))),
    ];
    for (field, val) in picks.into_iter().flatten() {
        conds.push(Cond::Cmp(field, Op::Eq, val));
    }
    let cond = match conds.len() {
        0 => None,
        1 => conds.pop(),
        _ => Some(Cond::And(conds)),
    };
    Query {
        cond,
        order: sort.to_vec(),
    }
}

/// The word and fields a query stands for, when it is nothing but those (and an order).
fn fields_of(q: &Query) -> Option<(String, SearchState)> {
    let list = match &q.cond {
        None => Vec::new(),
        Some(Cond::And(v)) => v.clone(),
        Some(c) => vec![c.clone()],
    };
    let mut word = String::new();
    let mut s = SearchState::default();
    for c in list {
        match c {
            Cond::Cmp(Field::Title, Op::Contains, Val::Text(t)) if word.is_empty() => word = t,
            Cond::Cmp(Field::Key, Op::Eq, Val::Text(t)) if word.is_empty() && is_key(&t) => {
                word = t
            }
            Cond::Cmp(Field::Type, Op::Eq, Val::Kind(k)) if s.kind.is_none() => s.kind = Some(k),
            Cond::Cmp(Field::Status, Op::Eq, Val::Status(v)) if s.status.is_none() => {
                s.status = Some(v);
            }
            Cond::Cmp(Field::Priority, Op::Eq, Val::Priority(p)) if s.priority.is_none() => {
                s.priority = Some(p);
            }
            _ => return None,
        }
    }
    Some((word, s))
}

/// The query now in effect, or why the text is not one.
pub fn effective(state: &State) -> Result<Query, QueryError> {
    let (lib, s) = (&state.library, &state.library.search);
    match s.mode {
        SearchMode::Standard => Ok(standard_query(&state.browser.search, s, &lib.sort)),
        SearchMode::Query => parse_query(&s.query).map(|mut q| {
            if q.order.is_empty() {
                q.order = lib.sort.clone();
            }
            q
        }),
    }
}

fn to_query_mode(mut state: State) -> State {
    let q = standard_query(
        &state.browser.search,
        &state.library.search,
        &state.library.sort,
    );
    state.library.search.query = print_query(&q);
    state.library.search.mode = SearchMode::Query;
    state
}

fn to_standard_mode(mut state: State) -> State {
    let parsed = parse_query(&state.library.search.query)
        .ok()
        .and_then(|q| fields_of(&q).map(|f| (f, q.order)));
    match parsed {
        Some(((word, fields), order)) => {
            state.browser.search = word;
            state.library.search = SearchState {
                query: state.library.search.query.clone(),
                ..fields
            };
            if !order.is_empty() {
                state.library.sort = order;
            }
        }
        None => {
            state.notice = Some(Notice {
                kind: NoticeKind::Info,
                text: "That query is more than the fields can show, so they keep their old values."
                    .to_owned(),
            });
        }
    }
    state.library.search.mode = SearchMode::Standard;
    state
}

/// Handle a message of the bar.
pub fn update(state: State, msg: SearchMsg) -> State {
    let mut state = state;
    match msg {
        SearchMsg::Mode(true) => return to_query_mode(state),
        SearchMsg::Mode(false) => return to_standard_mode(state),
        SearchMsg::Kind(k) => state.library.search.kind = k,
        SearchMsg::Status(v) => state.library.search.status = v,
        SearchMsg::Priority(p) => state.library.search.priority = p,
        SearchMsg::Query(text) => state.library.search.query = text,
        SearchMsg::Complete(n) => {
            let text = state.library.search.query.clone();
            if let Some(c) = complete(&text).get(n) {
                state.library.search.query = apply(&text, c);
            }
        }
    }
    state
}
