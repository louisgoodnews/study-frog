//! Where the user has been: screens, stacks and items, with Back, Forward and a list to jump to any of them, like a
//! browser's history. A place is noted after every update that changed where the user is, so no screen has to
//! remember to report. Pure; kept for the session (a new vault starts a new history).

use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{Screen, Selection, State};

/// Most places kept; the oldest go first.
pub const MAX: usize = 50;

/// A place the user can return to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// A screen (not one of the two below).
    Screen(Screen),
    /// A stack's page, by folder.
    Stack(String),
    /// An item open in the editor.
    Item(Uid),
}

/// The visited places, oldest first, and which one the user is at.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct History {
    /// The places.
    pub entries: Vec<Place>,
    /// The index of the current place.
    pub at: usize,
    /// The place a Back, Forward or jump is going to: arriving there is not a new visit.
    pub skip: Option<Place>,
    /// The vault these places belong to.
    pub vault: Option<String>,
    /// Titles of the items visited, newest last.
    pub titles: Vec<(Uid, String)>,
}

/// What the user did with the history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryMsg {
    /// Go to the place before the current one.
    Back,
    /// Go to the place after the current one.
    Forward,
    /// Go to the place at this index.
    Go(usize),
}

/// The name of a screen as the sidebar and the history show it.
pub fn screen_title(screen: Screen) -> &'static str {
    match screen {
        Screen::Welcome => "Vault chooser",
        Screen::Home => "Home",
        Screen::Folders => "Folders",
        Screen::Agent => "Assistant",
        Screen::Library => "Library",
        Screen::Rehearse => "Rehearse",
        Screen::Settings => "Settings",
        Screen::Editor => "Item",
        Screen::Graph => "Graph",
        Screen::Calendar => "Calendar",
        Screen::Board => "Board",
        Screen::Achievements => "Achievements",
        Screen::Reports => "Reports",
        Screen::Diary => "Diary",
        Screen::Prompts => "Prompts",
        Screen::Pdf => "PDF",
        Screen::Help => "Help",
    }
}

/// Where the user is now, if it is somewhere that can be returned to.
pub fn place_of(state: &State) -> Option<Place> {
    match state.screen {
        Screen::Welcome | Screen::Pdf => None,
        Screen::Editor => state.editor.as_ref().map(|e| Place::Item(e.uid)),
        Screen::Library => match &state.browser.selection {
            Selection::Stack(folder) => Some(Place::Stack(folder.clone())),
            _ => Some(Place::Screen(Screen::Library)),
        },
        other => Some(Place::Screen(other)),
    }
}

/// The words for a place in the list.
pub fn label(state: &State, place: &Place) -> String {
    match place {
        Place::Screen(s) => screen_title(*s).to_owned(),
        Place::Stack(folder) => {
            let title = state
                .vault
                .as_ref()
                .and_then(|v| v.overview.stacks.iter().find(|s| &s.folder == folder))
                .map_or(folder.as_str(), |s| s.title.as_str());
            format!("Stack: {title}")
        }
        Place::Item(uid) => {
            let known = state.history.titles.iter().rev().find(|(u, _)| u == uid);
            format!("Item: {}", known.map_or("(untitled)", |(_, t)| t.as_str()))
        }
    }
}

/// Whether there is a place before the current one.
pub fn can_back(h: &History) -> bool {
    h.at > 0 && h.at < h.entries.len()
}

/// Whether there is a place after the current one.
pub fn can_forward(h: &History) -> bool {
    h.at + 1 < h.entries.len()
}

fn note_title(state: &mut State, place: &Place) {
    let (Place::Item(uid), Some(e)) = (place, state.editor.as_ref()) else {
        return;
    };
    let title = e.original.title.clone();
    let titles = &mut state.history.titles;
    titles.retain(|(u, _)| u != uid);
    titles.push((*uid, title));
    let extra = titles.len().saturating_sub(MAX);
    titles.drain(..extra);
}

/// Note the place the user is at now, if it differs from where they were before the update.
pub fn record(state: State, before: Option<Place>) -> State {
    let mut state = state;
    let root = state.vault.as_ref().map(|v| v.root.clone());
    if root != state.history.vault {
        state.history = History {
            vault: root,
            ..History::default()
        };
    }
    let now = place_of(&state);
    let Some(now) = now.filter(|n| Some(n) != before.as_ref()) else {
        return state;
    };
    note_title(&mut state, &now);
    let h = &mut state.history;
    if h.skip.as_ref() == Some(&now) {
        h.skip = None;
    } else if h.entries.get(h.at) != Some(&now) {
        h.entries.truncate(h.at + 1);
        h.entries.push(now);
        let extra = h.entries.len().saturating_sub(MAX);
        h.entries.drain(..extra);
        h.at = h.entries.len() - 1;
    }
    state
}

fn go(state: State, index: usize) -> (State, Vec<Effect>) {
    let Some(place) = state.history.entries.get(index).cloned() else {
        return (state, Vec::new());
    };
    if place_of(&state).as_ref() == Some(&place) {
        let mut state = state;
        state.history.at = index;
        return (state, Vec::new());
    }
    let then = Msg::Chrome(crate::chrome::ChromeMsg::History(HistoryMsg::Go(index)));
    let (state, waiting) = crate::editor_save::flush_before(state, then);
    if let Some(effects) = waiting {
        return (state, effects);
    }
    let mut state = state;
    state.history.at = index;
    state.history.skip = Some(place.clone());
    match place {
        Place::Screen(screen) => crate::nav::navigate(state, screen),
        Place::Stack(folder) => {
            let (mut state, effects) = crate::nav::navigate(state, Screen::Library);
            state.replay = Some(Box::new(Msg::Select(Selection::Stack(folder))));
            (state, effects)
        }
        Place::Item(uid) => {
            state.replay = Some(Box::new(Msg::OpenItem(uid)));
            (state, Vec::new())
        }
    }
}

/// Handle Back, Forward and a jump.
pub fn update(state: State, msg: HistoryMsg) -> (State, Vec<Effect>) {
    let at = state.history.at;
    match msg {
        HistoryMsg::Back if can_back(&state.history) => go(state, at - 1),
        HistoryMsg::Forward if can_forward(&state.history) => go(state, at + 1),
        HistoryMsg::Go(i) => go(state, i),
        _ => (state, Vec::new()),
    }
}
