//! The library list: the rows of the vault, the columns shown, the sort and what the current stack and search let through. Pure.

use studyfrog_core::query::Order;
use studyfrog_core::query_eval::{Row, run_filtered};
use studyfrog_service::library_rows::LibraryItem;

use crate::effect::Effect;
use crate::state::{Selection, State};

use crate::library_sort::{sorted_by, toggle};

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
    /// The search bar.
    pub search: crate::library_search::SearchState,
    /// Why the query text is not a query, if so.
    pub query_error: Option<studyfrog_core::query::QueryError>,
    /// The rows ticked.
    pub selected: std::collections::BTreeSet<studyfrog_core::uid::Uid>,
    /// The saved filters of the vault.
    pub filters: Vec<studyfrog_config::filters::SavedFilter>,
    /// Why the filters file is not used.
    pub filters_problem: Option<String>,
    /// The name typed for a filter to save.
    pub filter_name: String,
    /// How to take back the last bulk edit.
    pub undo: Option<studyfrog_service::bulk::Snapshot>,
    /// The question before deleting is shown.
    pub confirm_delete: bool,
    /// The tag typed in the bulk bar.
    pub tag_input: String,
    /// The page of the open stack.
    pub stack_view: crate::stack_view_ui::StackViewState,
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
            search: Default::default(),
            query_error: None,
            selected: Default::default(),
            filters: Vec::new(),
            filters_problem: None,
            filter_name: String::new(),
            undo: None,
            confirm_delete: false,
            tag_input: String::new(),
            stack_view: Default::default(),
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
    /// The search bar.
    Search(crate::library_search::SearchMsg),
    /// The selection, the bulk bar and the saved filters.
    Bulk(crate::library_bulk::BulkMsg),
    /// The page of a stack.
    Stack(crate::stack_view_ui::StackViewMsg),
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

/// Handle a library message.
pub fn update(state: State, msg: LibraryMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    let lib = &mut state.library;
    match msg {
        LibraryMsg::Loaded(Ok(items)) => {
            lib.rows = items.iter().map(|i| i.row.clone()).collect();
            lib.items = items;
            lib.generation += 1;
            crate::library_bulk::prune_selection(&mut state);
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
        LibraryMsg::Search(m) => return (crate::library_search::update(state, m), Vec::new()),
        LibraryMsg::Bulk(m) => return crate::library_bulk::update(state, m),
        LibraryMsg::Stack(m) => return crate::stack_view_ui::update(state, m),
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

/// What the bar asks for, as one comparable text.
fn search_key(state: &State) -> String {
    let s = &state.library.search;
    match s.mode {
        crate::library_search::SearchMode::Query => format!("q:{}", s.query),
        crate::library_search::SearchMode::Standard => format!(
            "s:{}|{:?}|{:?}|{:?}",
            state.browser.search.trim(),
            s.kind,
            s.status,
            s.priority
        ),
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
        effects.push(Effect::Library(
            crate::library_bulk::LibraryEffect::LoadFilters,
        ));
    }
    let key = (
        state.library.generation,
        state.browser.selection.clone(),
        search_key(&state),
        state.library.sort.clone(),
    );
    if state.library.shown_for.as_ref() != Some(&key) {
        let today = state.wall_ms.div_euclid(86_400_000);
        let selection = key.1.clone();
        match crate::library_search::effective(&state) {
            Ok(q) => {
                state.library.query_error = None;
                state.library.shown =
                    run_filtered(&q, &state.library.rows, today, &|r| in_scope(r, &selection));
            }
            Err(e) => state.library.query_error = Some(e),
        }
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
