//! The window chrome: the sidebar, the top-bar menus, the global search and the notification list.
//! Pure: no egui, no I/O.

use studyfrog_store::overview::ItemRow;

use crate::effect::Effect;
use crate::nav::navigate;
use crate::state::{Notice, NoticeKind, Screen, State};

/// Most notices kept in the notification list.
pub const MAX_NOTIFICATIONS: usize = 30;
/// Most search results shown.
pub const MAX_RESULTS: usize = 10;

/// The menu opened from the top bar (at most one at a time).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Menu {
    /// Switch to another vault.
    Vaults,
    /// Make something new.
    Create,
    /// Recent notices.
    Notifications,
    /// Shortcuts and where things are.
    Help,
    /// About, and closing the vault.
    Profile,
}

/// Something the Create menu can make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Creatable {
    /// A stack of cards.
    Stack,
    /// A card in the open stack.
    Card,
    /// A planned study session.
    Session,
    /// A card on the board.
    BoardCard,
    /// Cards from a file.
    Import,
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
    /// Clear the search.
    SearchClear,
    /// An entry of the Create menu was chosen.
    Create(Creatable),
}

fn with(state: State, chrome: Chrome) -> (State, Vec<Effect>) {
    (State { chrome, ..state }, Vec::new())
}

fn hint(state: State, text: &str) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Info,
        text: text.to_owned(),
    });
    (State { notice, ..state }, Vec::new())
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
    let results = if query.is_empty() {
        Vec::new()
    } else {
        state.chrome.results.clone()
    };
    let effects = if query.is_empty() || state.vault.is_none() {
        Vec::new()
    } else {
        vec![Effect::SearchItems(query)]
    };
    let c = Chrome {
        search: text,
        results,
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

/// Go where the Create menu entry leads, telling the user what to do next.
fn create(state: State, what: Creatable) -> (State, Vec<Effect>) {
    let state = State {
        chrome: Chrome {
            menu: None,
            ..state.chrome.clone()
        },
        ..state
    };
    match what {
        Creatable::Stack => navigate(state, Screen::Library),
        Creatable::Card | Creatable::Import => {
            let (state, effects) = navigate(state, Screen::Library);
            let stack_open = matches!(state.browser.selection, crate::state::Selection::Stack(_));
            if stack_open {
                return (state, effects);
            }
            let text =
                "Open a stack in the library first: cards are added to the stack you have open.";
            let (state, _) = hint(state, text);
            (state, effects)
        }
        Creatable::Session => {
            let planner = crate::planner::Planner {
                pending_new: true,
                ..state.planner.clone()
            };
            navigate(State { planner, ..state }, Screen::Calendar)
        }
        Creatable::BoardCard => navigate(state, Screen::Board),
    }
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
        ChromeMsg::SearchClear => {
            let c = Chrome {
                search: String::new(),
                results: Vec::new(),
                ..state.chrome.clone()
            };
            with(state, c)
        }
        ChromeMsg::Create(what) => create(state, what),
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
