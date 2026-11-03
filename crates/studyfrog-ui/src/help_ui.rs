//! Reading help: which page opens for which screen, following links, going back and searching. Pure.

use studyfrog_md::blocks::parse_blocks;
use studyfrog_service::help::{PAGES, PageRef, resolve, search, supplied};

use crate::effect::Effect;
pub use crate::help_types::{HelpEffect, HelpMsg, HelpState, PageEdit};
use crate::state::{Notice, NoticeKind, Screen, Selection, State};

/// The page opened when help has no page yet.
pub const FIRST_PAGE: &str = "getting-started";

/// The supplied page that explains `screen` (the stack page while a stack is open in the library).
pub fn page_for(state: &State) -> &'static str {
    match state.screen {
        Screen::Welcome | Screen::Help => FIRST_PAGE,
        Screen::Home => "home",
        Screen::Library if matches!(state.browser.selection, Selection::Stack(_)) => "stack-page",
        Screen::Library => "library",
        Screen::Rehearse => "rehearse",
        Screen::Settings => "settings",
        Screen::Editor => "item-page",
        Screen::Graph => "graph",
        Screen::Calendar => "calendar",
        Screen::Board => "board",
        Screen::Achievements => "achievements",
        Screen::Reports => "reports",
        Screen::Agent => "assistant",
        Screen::Pdf => "pdf",
    }
}

/// The Markdown of a page, or `None` when it no longer exists.
pub fn text_of<'a>(help: &'a HelpState, page: &PageRef) -> Option<&'a str> {
    match page {
        PageRef::Supplied(id) => supplied(id).map(|p| p.text),
        PageRef::User(file) => help
            .user
            .iter()
            .find(|u| &u.file == file)
            .map(|u| u.text.as_str()),
    }
}

/// The page showing: the chosen one, or the first page.
pub fn current(help: &HelpState) -> PageRef {
    help.page
        .clone()
        .unwrap_or_else(|| PageRef::Supplied(FIRST_PAGE.to_owned()))
}

/// Show `page`, parsing it once; the page left goes on the back list when `remember` is set.
pub(crate) fn show(help: HelpState, page: PageRef, remember: bool) -> HelpState {
    let blocks = text_of(&help, &page).map(parse_blocks).unwrap_or_default();
    let mut back = help.back.clone();
    if let (true, Some(old)) = (remember, help.page.clone()) {
        back.push(old);
    }
    HelpState {
        page: Some(page),
        blocks,
        back,
        ask_delete: false,
        ..help
    }
}

fn with(state: State, help: HelpState) -> (State, Vec<Effect>) {
    (State { help, ..state }, Vec::new())
}

fn load() -> Vec<Effect> {
    vec![Effect::Help(HelpEffect::Load)]
}

/// Go to the help screen at `page`, remembering where the user came from.
pub fn open_at(state: State, page: PageRef) -> (State, Vec<Effect>) {
    let came_from = match state.screen {
        Screen::Help => state.help.came_from,
        other => Some(other),
    };
    let help = show(state.help.clone(), page, state.screen == Screen::Help);
    let help = HelpState { came_from, ..help };
    let effects = if state.vault.is_some() {
        load()
    } else {
        Vec::new()
    };
    let state = State {
        screen: Screen::Help,
        help,
        ..state
    };
    (state, effects)
}

fn link(state: State, target: String) -> (State, Vec<Effect>) {
    match resolve(&target, &state.help.user) {
        Some(page) => open_at(state, page),
        None => {
            let text = format!("No help page is called \"{target}\".");
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text,
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}

fn back(state: State) -> (State, Vec<Effect>) {
    let mut help = state.help.clone();
    match help.back.pop() {
        Some(page) => {
            let help = show(help, page, false);
            with(state, help)
        }
        None => (state, Vec::new()),
    }
}

fn leave(state: State) -> (State, Vec<Effect>) {
    let to = state.help.came_from.unwrap_or(Screen::Welcome);
    let to = if state.vault.is_none() {
        Screen::Welcome
    } else {
        to
    };
    crate::nav::navigate(state, to)
}

fn searched(state: State, text: String) -> (State, Vec<Effect>) {
    let hits = search(&text, &state.help.user);
    let help = HelpState {
        search: text,
        hits,
        ..state.help.clone()
    };
    with(state, help)
}

fn loaded(
    state: State,
    result: Result<Vec<studyfrog_service::help::UserPage>, String>,
) -> (State, Vec<Effect>) {
    match result {
        Ok(user) => {
            let help = HelpState {
                user,
                ..state.help.clone()
            };
            let page = current(&help);
            let hits = search(&help.search, &help.user);
            let help = HelpState {
                hits,
                ..show(help, page, false)
            };
            with(state, help)
        }
        Err(e) => {
            let text = format!("Your help pages could not be read: {e}");
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text,
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}

/// Handle a message of the help screen.
pub fn update(state: State, msg: HelpMsg) -> (State, Vec<Effect>) {
    match msg {
        HelpMsg::ForScreen => {
            let id = page_for(&state).to_owned();
            open_at(state, PageRef::Supplied(id))
        }
        HelpMsg::Open(page) => open_at(state, page),
        HelpMsg::Link(target) => link(state, target),
        HelpMsg::Back => back(state),
        HelpMsg::Leave => leave(state),
        HelpMsg::Search(text) => searched(state, text),
        HelpMsg::Loaded(result) => loaded(state, result),
        HelpMsg::Tutorial(m) => crate::tutorial::update(state, m),
        edit => crate::help_edit::update(state, edit),
    }
}

/// Every supplied page id, in tree order (for tests and the link check).
pub fn page_ids() -> Vec<&'static str> {
    PAGES.iter().map(|p| p.id).collect()
}
