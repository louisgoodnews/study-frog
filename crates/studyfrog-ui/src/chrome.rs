//! The window chrome: the sidebar, the top-bar menus, the global search and the notification list.
//! Pure: no egui, no I/O.

use studyfrog_store::overview::ItemRow;

use crate::effect::Effect;
use crate::pdf_types::PdfEffect;
use crate::state::{Notice, NoticeKind, State};

/// Most notices kept in the notification list.
pub const MAX_NOTIFICATIONS: usize = 30;
/// Most search results shown.
pub const MAX_RESULTS: usize = 10;
/// Most characters of the vault name shown in the top bar before it is cut with an ellipsis.
pub const MAX_VAULT_NAME: usize = 24;

/// The vault's folder name for the top bar, cut to [`MAX_VAULT_NAME`] characters with an ellipsis. The path is
/// not part of it (the bar's tooltip carries the full path).
pub fn vault_label(root: &str) -> String {
    let name = root
        .rsplit(['/', '\\'])
        .find(|part| !part.is_empty())
        .unwrap_or(root);
    if name.chars().count() <= MAX_VAULT_NAME {
        return name.to_owned();
    }
    let cut: String = name.chars().take(MAX_VAULT_NAME - 1).collect();
    format!("{}…", cut.trim_end())
}

/// The menu opened from the top bar (at most one at a time).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Menu {
    /// Switch to another vault.
    Vaults,
    /// Recent notices.
    Notifications,
    /// Shortcuts and where things are.
    Help,
    /// About, and closing the vault.
    Profile,
}

/// What the chrome shows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Chrome {
    /// The sidebar is folded away.
    pub sidebar_hidden: bool,
    /// The open menu.
    pub menu: Option<Menu>,
    /// The global search text.
    pub search: String,
    /// Items matching the search.
    pub results: Vec<ItemRow>,
    /// PDFs and pages matching the search.
    pub pdf_results: Vec<studyfrog_store::pdf_search::PdfHit>,
    /// Recent notices, oldest first.
    pub notifications: Vec<Notice>,
    /// How many of them the user has seen.
    pub seen: usize,
}

/// Messages of the chrome.
#[derive(Debug, Clone, PartialEq)]
pub enum ChromeMsg {
    /// Show or fold the sidebar.
    ToggleSidebar,
    /// Open the menu, or close it when it is the one open.
    ToggleMenu(Menu),
    /// Close any menu.
    CloseMenu,
    /// The search text changed.
    SearchEdited(String),
    /// The search finished: the query it was for, and what it found.
    SearchResults(String, Result<Vec<ItemRow>, String>),
    /// The PDF part of a search finished.
    PdfResults(
        String,
        Result<Vec<studyfrog_store::pdf_search::PdfHit>, String>,
    ),
    /// Clear the search.
    SearchClear,
}

fn with(state: State, chrome: Chrome) -> (State, Vec<Effect>) {
    (State { chrome, ..state }, Vec::new())
}

fn toggle_menu(state: State, menu: Menu) -> (State, Vec<Effect>) {
    let c = state.chrome.clone();
    let opening = c.menu != Some(menu);
    let seen = if opening && menu == Menu::Notifications {
        c.notifications.len()
    } else {
        c.seen
    };
    with(
        state,
        Chrome {
            menu: opening.then_some(menu),
            seen,
            ..c
        },
    )
}

fn search_edited(state: State, text: String) -> (State, Vec<Effect>) {
    let query = text.trim().to_owned();
    let (results, pdf_results) = if query.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        (
            state.chrome.results.clone(),
            state.chrome.pdf_results.clone(),
        )
    };
    let effects = if query.is_empty() || state.vault.is_none() {
        Vec::new()
    } else {
        vec![
            Effect::SearchItems(query.clone()),
            Effect::Pdf(PdfEffect::Search(query)),
        ]
    };
    let c = Chrome {
        search: text,
        results,
        pdf_results,
        ..state.chrome.clone()
    };
    (State { chrome: c, ..state }, effects)
}

fn search_results(
    state: State,
    query: String,
    result: Result<Vec<ItemRow>, String>,
) -> (State, Vec<Effect>) {
    if query != state.chrome.search.trim() {
        return (state, Vec::new()); // an answer to something typed since
    }
    match result {
        Ok(results) => with(
            state.clone(),
            Chrome {
                results,
                ..state.chrome
            },
        ),
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Search failed: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}

fn pdf_results(
    state: State,
    query: String,
    result: Result<Vec<studyfrog_store::pdf_search::PdfHit>, String>,
) -> (State, Vec<Effect>) {
    if query != state.chrome.search.trim() {
        return (state, Vec::new());
    }
    let pdf_results = result.unwrap_or_default();
    let c = Chrome {
        pdf_results,
        ..state.chrome.clone()
    };
    with(state, c)
}

/// Handle the chrome's messages.
pub fn update(state: State, msg: ChromeMsg) -> (State, Vec<Effect>) {
    match msg {
        ChromeMsg::ToggleSidebar => {
            let c = Chrome {
                sidebar_hidden: !state.chrome.sidebar_hidden,
                ..state.chrome.clone()
            };
            with(state, c)
        }
        ChromeMsg::ToggleMenu(menu) => toggle_menu(state, menu),
        ChromeMsg::CloseMenu => {
            let c = Chrome {
                menu: None,
                ..state.chrome.clone()
            };
            with(state, c)
        }
        ChromeMsg::SearchEdited(text) => search_edited(state, text),
        ChromeMsg::SearchResults(query, result) => search_results(state, query, result),
        ChromeMsg::PdfResults(query, result) => pdf_results(state, query, result),
        ChromeMsg::SearchClear => {
            let c = Chrome {
                search: String::new(),
                results: Vec::new(),
                pdf_results: Vec::new(),
                ..state.chrome.clone()
            };
            with(state, c)
        }
    }
}

/// Keep every new notice in the list the bell shows. Call after each update with the notice from
/// before it.
pub fn track_notice(before: &Option<Notice>, mut state: State) -> State {
    let Some(n) = state.notice.clone().filter(|n| Some(n) != before.as_ref()) else {
        return state;
    };
    let c = &mut state.chrome;
    c.notifications.push(n);
    if c.notifications.len() > MAX_NOTIFICATIONS {
        c.notifications.remove(0);
        c.seen = c.seen.saturating_sub(1);
    }
    state
}

/// Notices the user has not looked at yet.
pub fn unread(c: &Chrome) -> usize {
    c.notifications.len().saturating_sub(c.seen)
}
