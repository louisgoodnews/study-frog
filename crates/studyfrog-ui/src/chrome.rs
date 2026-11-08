//! The window chrome: the sidebar, the top-bar menus, the global search and the notification list.
//! Pure: no egui, no I/O.

use studyfrog_store::overview::ItemRow;

use crate::effect::Effect;
use crate::pdf_types::PdfEffect;
use crate::state::{Notice, NoticeKind, State};

pub use crate::notify::MAX_NOTIFICATIONS;
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
    /// About, and closing the vault.
    Profile,
    /// The places visited, to jump to.
    History,
}

/// What the chrome shows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Chrome {
    /// The sidebar is folded away.
    pub sidebar_hidden: bool,
    /// Zen mode chosen by the user (`None`: follow the automatic choice, see `zen::active`).
    pub zen: Option<bool>,
    /// The open menu.
    pub menu: Option<Menu>,
    /// The global search text.
    pub search: String,
    /// Items matching the search.
    pub results: Vec<ItemRow>,
    /// PDFs and pages matching the search.
    pub pdf_results: Vec<studyfrog_store::pdf_search::PdfHit>,
    /// The notification list, oldest first.
    pub notifications: Vec<crate::notify::Notification>,
    /// The next notification id.
    pub next_id: u64,
    /// Which notifications the panel lists.
    pub notify_filter: crate::notify::NotifyFilter,
    /// The vault whose stored list this one is.
    pub notify_vault: Option<String>,
    /// The stored list has been read, so changes are stored too.
    pub notify_loaded: bool,
    /// The user chose Exit: the window is to be closed.
    pub quit: bool,
}

/// Messages of the chrome.
#[derive(Debug, Clone, PartialEq)]
pub enum ChromeMsg {
    /// Show or fold the sidebar.
    ToggleSidebar,
    /// Zen mode on or off (Ctrl+Shift+F, F11, or the top bar's button).
    ToggleZen,
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
    /// The user did something in the notification panel.
    Notify(crate::notify::NotifyMsg),
    /// The stored notifications were read.
    NotificationsLoaded(Result<Vec<crate::notify::Notification>, String>),
    /// The Exit button was pressed, or the question after it was answered.
    Quit(crate::quit_ui::QuitMsg),
    /// Back, Forward or a jump in the history.
    History(crate::history::HistoryMsg),
}

fn with(state: State, chrome: Chrome) -> (State, Vec<Effect>) {
    (State { chrome, ..state }, Vec::new())
}

fn toggle_menu(state: State, menu: Menu) -> (State, Vec<Effect>) {
    let c = state.chrome.clone();
    let opening = c.menu != Some(menu);
    with(
        state,
        Chrome {
            menu: opening.then_some(menu),
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
        ChromeMsg::ToggleZen => {
            let mut state = state;
            state.chrome.zen = Some(!crate::zen::active(&state));
            (state, Vec::new())
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
        ChromeMsg::Notify(m) => crate::notify::update(state, m),
        ChromeMsg::Quit(m) => crate::quit_ui::update(state, m),
        ChromeMsg::History(m) => crate::history::update(state, m),
        ChromeMsg::NotificationsLoaded(r) => crate::notify::loaded(state, r),
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

/// Notifications the user has not read yet.
pub fn unread(c: &Chrome) -> usize {
    crate::notify::unread_count(&c.notifications)
}
