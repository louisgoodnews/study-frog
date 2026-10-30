//! Update logic of the stack browser: selection, search and the create forms. Pure.

use crate::effect::Effect;
use crate::msg::{DraftField, Msg};
use crate::state::{Browser, CardDraft, ItemRow, Notice, NoticeKind, Selection, State};

fn error(text: impl Into<String>) -> Option<Notice> {
    Some(Notice {
        kind: NoticeKind::Error,
        text: text.into(),
    })
}

/// Effects that refresh the items of `selection` (none when nothing is selected).
pub fn reload(selection: &Selection) -> Vec<Effect> {
    match selection {
        Selection::Nothing => Vec::new(),
        other => vec![Effect::LoadItems(other.clone())],
    }
}

/// Items whose title contains `query`, ignoring case (Unicode simple lowercase) and surrounding spaces.
pub fn filter_items<'a>(items: &'a [ItemRow], query: &str) -> Vec<&'a ItemRow> {
    let needle: String = query.trim().chars().flat_map(char::to_lowercase).collect();
    items
        .iter()
        .filter(|i| {
            needle.is_empty()
                || i.title
                    .chars()
                    .flat_map(char::to_lowercase)
                    .collect::<String>()
                    .contains(&needle)
        })
        .collect()
}

/// Handle the browser's messages; anything else is returned unchanged.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    match msg {
        Msg::Select(sel) => select(state, sel),
        Msg::SearchEdited(text) => with_browser(state, |b| Browser { search: text, ..b }),
        Msg::ItemsLoaded(result) => items_loaded(state, result),
        Msg::NewStackEdited(text) => with_browser(state, |b| Browser {
            new_stack: text,
            ..b
        }),
        Msg::CreateStackClicked => create_stack(state),
        Msg::DraftEdited(field, text) => with_browser(state, |b| edit_draft(b, field, text)),
        Msg::AddCardClicked => add_card(state),
        Msg::Created(result) => created(state, result),
        _ => (state, Vec::new()),
    }
}

fn with_browser(state: State, f: impl FnOnce(Browser) -> Browser) -> (State, Vec<Effect>) {
    let browser = f(state.browser.clone());
    (State { browser, ..state }, Vec::new())
}

fn edit_draft(b: Browser, field: DraftField, text: String) -> Browser {
    let d = &b.draft;
    let draft = match field {
        DraftField::Title => CardDraft {
            title: text,
            ..d.clone()
        },
        DraftField::Front => CardDraft {
            front: text,
            ..d.clone()
        },
        DraftField::Back => CardDraft {
            back: text,
            ..d.clone()
        },
    };
    Browser { draft, ..b }
}

fn select(state: State, sel: Selection) -> (State, Vec<Effect>) {
    let browser = Browser {
        selection: sel.clone(),
        items: Vec::new(),
        search: String::new(),
        ..state.browser.clone()
    };
    (State { browser, ..state }, reload(&sel))
}

fn items_loaded(state: State, result: Result<Vec<ItemRow>, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(items) => with_browser(state, |b| Browser { items, ..b }),
        Err(e) => (
            State {
                notice: error(format!("Could not list the items: {e}")),
                ..state
            },
            Vec::new(),
        ),
    }
}

fn create_stack(state: State) -> (State, Vec<Effect>) {
    let title = state.browser.new_stack.trim().to_owned();
    if title.is_empty() {
        return (
            State {
                notice: error("Enter a title for the new stack."),
                ..state
            },
            Vec::new(),
        );
    }
    (
        State {
            notice: None,
            ..state
        },
        vec![Effect::CreateStack(title)],
    )
}

fn add_card(state: State) -> (State, Vec<Effect>) {
    let Selection::Stack(folder) = &state.browser.selection else {
        return (
            State {
                notice: error("Open a stack first."),
                ..state
            },
            Vec::new(),
        );
    };
    let d = &state.browser.draft;
    if d.front.trim().is_empty() || d.back.trim().is_empty() {
        return (
            State {
                notice: error("A card needs a front and a back."),
                ..state
            },
            Vec::new(),
        );
    }
    let effect = Effect::CreateCard {
        folder: folder.clone(),
        title: d.title.clone(),
        front: d.front.clone(),
        back: d.back.clone(),
    };
    (
        State {
            notice: None,
            ..state
        },
        vec![effect],
    )
}

fn created(state: State, result: Result<crate::state::VaultInfo, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(info) => {
            let browser = Browser {
                new_stack: String::new(),
                draft: CardDraft::default(),
                ..state.browser.clone()
            };
            let reload = reload(&browser.selection);
            let notice = Some(Notice {
                kind: NoticeKind::Info,
                text: "Created.".to_owned(),
            });
            (
                State {
                    vault: Some(info),
                    browser,
                    notice,
                    ..state
                },
                reload,
            )
        }
        Err(e) => (
            State {
                notice: error(format!("Could not create it: {e}")),
                ..state
            },
            Vec::new(),
        ),
    }
}
