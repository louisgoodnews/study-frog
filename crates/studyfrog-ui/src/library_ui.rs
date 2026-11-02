//! The library list: the rows of the vault, the columns shown, the sort and what the current stack and search let through. Pure.

use studyfrog_core::query::{Cond, Field, MAX_ORDER_KEYS, Op, Order, Query, Val};
use studyfrog_core::query_eval::{Row, run_filtered};
use studyfrog_service::library_rows::LibraryItem;

use crate::effect::Effect;
use crate::state::{Selection, State};

pub use crate::library_columns::{
    COLUMNS, Column, DEFAULT_COLUMNS, cell, column_field, column_label, columns_from_text,
    columns_to_text,
};

/// The list of the library.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryState {
    /// Every item of the vault, as last loaded.
    pub items: Vec<LibraryItem>,
    /// The rows alone, for the query evaluator.
    pub rows: Vec<Row>,
    /// Counts the loads, so a change of the rows is noticed.
    pub generation: u64,
    /// The columns shown.
    pub columns: Vec<Column>,
    /// How the list is sorted (at most three keys).
    pub sort: Vec<Order>,
    /// The indices into `items` that the stack and the search let through, in order.
    pub shown: Vec<usize>,
    /// What `shown` was made from: the generation, the stack, the search and the sort.
    pub shown_for: Option<(u64, Selection, String, Vec<Order>)>,
    /// How many items the stack lets through before the search.
    pub scope_total: usize,
    /// The list must be loaded again.
    pub stale: bool,
    /// The old overview of stacks is shown instead of the list.
    pub show_overview: bool,
}

impl Default for LibraryState {
    fn default() -> Self {
        LibraryState {
            items: Vec::new(),
            rows: Vec::new(),
            generation: 0,
            columns: DEFAULT_COLUMNS.to_vec(),
            sort: Vec::new(),
            shown: Vec::new(),
            shown_for: None,
            scope_total: 0,
            stale: true,
            show_overview: false,
        }
    }
}

/// What happened on the library list.
#[derive(Debug, Clone, PartialEq)]
pub enum LibraryMsg {
    /// The rows were read.
    Loaded(Result<Vec<LibraryItem>, String>),
    /// A heading was clicked: sort by it, or add it to the sort with `true`.
    Sort(Column, bool),
    /// Show or hide a column.
    ToggleColumn(Column),
    /// Show the old overview of stacks, or the list.
    Overview(bool),
}

/// Whether this message changes what the list should show, so the rows are read again.
pub fn wants_reload(msg: &crate::msg::Msg) -> bool {
    use crate::msg::Msg;
    matches!(
        msg,
        Msg::ItemsLoaded(_)
            | Msg::VaultOpened(Ok(_))
            | Msg::Navigate(crate::state::Screen::Library)
            | Msg::Created(Ok(_))
    )
}

fn toggle(columns: &[Column], c: Column) -> Vec<Column> {
    if c == Column::Title {
        return columns.to_vec();
    }
    let mut next: Vec<Column> = columns.to_vec();
    match next.iter().position(|x| *x == c) {
        Some(i) => {
            next.remove(i);
        }
        None => next.push(c),
    }
    COLUMNS
        .iter()
        .map(|(x, _, _)| *x)
        .filter(|x| next.contains(x))
        .collect()
}

fn sorted_by(sort: &[Order], field: Field, add: bool) -> Vec<Order> {
    let current = sort.iter().find(|o| o.field == field).copied();
    let flipped = Order {
        field,
        descending: current.is_some_and(|o| !o.descending),
    };
    if !add {
        return vec![flipped];
    }
    let mut next: Vec<Order> = sort.iter().filter(|o| o.field != field).copied().collect();
    next.push(flipped);
    let start = next.len().saturating_sub(MAX_ORDER_KEYS);
    next.split_off(start)
}

/// Handle a library message.
pub fn update(state: State, msg: LibraryMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    let lib = &mut state.library;
    match msg {
        LibraryMsg::Loaded(Ok(items)) => {
            lib.rows = items.iter().map(|i| i.row.clone()).collect();
            lib.items = items;
            lib.generation += 1;
        }
        LibraryMsg::Loaded(Err(e)) => {
            state.notice = Some(crate::state::Notice {
                kind: crate::state::NoticeKind::Error,
                text: format!("Could not read the library: {e}"),
            });
        }
        LibraryMsg::Sort(c, add) => {
            if let Some(f) = column_field(c) {
                lib.sort = sorted_by(&lib.sort, f, add);
            }
        }
        LibraryMsg::ToggleColumn(Column::Title) => {}
        LibraryMsg::ToggleColumn(c) => {
            lib.columns = toggle(&lib.columns, c);
            let app = studyfrog_core::settings::AppSettings {
                library_columns: Some(columns_to_text(&state.library.columns)),
                ..state.app.clone()
            };
            let effects = vec![Effect::SaveAppSettings(app.clone())];
            state.app = app;
            return (state, effects);
        }
        LibraryMsg::Overview(on) => lib.show_overview = on,
    }
    (state, Vec::new())
}

fn in_scope(row: &Row, selection: &Selection) -> bool {
    match selection {
        Selection::Stack(folder) => row.stacks.iter().step_by(2).any(|s| s == folder),
        Selection::Inbox => row.stacks.is_empty(),
        Selection::Nothing => true,
    }
}

fn query_of(search: &str, sort: &[Order]) -> Query {
    let word = search.trim();
    let cond = (!word.is_empty())
        .then(|| Cond::Cmp(Field::Title, Op::Contains, Val::Text(word.to_owned())));
    Query {
        cond,
        order: sort.to_vec(),
    }
}

/// Ask for the rows when they are stale and the library is on screen, and work out what to show when its inputs changed.
pub fn sync(state: State, reload: bool) -> (State, Vec<Effect>) {
    let mut state = state;
    state.library.stale |= reload;
    let mut effects = Vec::new();
    if state.library.stale && state.vault.is_some() && state.screen == crate::state::Screen::Library
    {
        state.library.stale = false;
        effects.push(Effect::LoadLibrary);
    }
    let key = (
        state.library.generation,
        state.browser.selection.clone(),
        state.browser.search.clone(),
        state.library.sort.clone(),
    );
    if state.library.shown_for.as_ref() != Some(&key) {
        let today = state.wall_ms.div_euclid(86_400_000);
        let q = query_of(&key.2, &key.3);
        let selection = key.1.clone();
        state.library.shown =
            run_filtered(&q, &state.library.rows, today, &|r| in_scope(r, &selection));
        let scope = state
            .library
            .rows
            .iter()
            .filter(|r| in_scope(r, &selection));
        state.library.scope_total = scope.count();
        state.library.shown_for = Some(key);
    }
    (state, effects)
}
