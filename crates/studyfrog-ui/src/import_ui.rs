//! Importing cards and making them from a registered card type: update logic. Pure.

use studyfrog_service::import::ImportReport;

use crate::browse::reload;
use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{Browser, CardDraft, Notice, NoticeKind, Selection, State, VaultInfo};

/// What an import or a card type produced.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportOutcome {
    /// The vault after the cards were added.
    pub info: VaultInfo,
    /// How many cards were written, and which were skipped.
    pub report: ImportReport,
}

fn notice(kind: NoticeKind, text: impl Into<String>) -> Option<Notice> {
    Some(Notice {
        kind,
        text: text.into(),
    })
}

fn failed(state: State, text: &str) -> (State, Vec<Effect>) {
    (
        State {
            notice: notice(NoticeKind::Error, text),
            ..state
        },
        Vec::new(),
    )
}

fn stack_of(state: &State) -> Option<String> {
    match &state.browser.selection {
        Selection::Stack(folder) => Some(folder.clone()),
        _ => None,
    }
}

/// "Add cards" with a registered card type: the text of the front field is the source.
pub fn add_from_type(state: State) -> (State, Vec<Effect>) {
    let (Some(folder), Some(type_id)) = (stack_of(&state), state.browser.card_type.clone()) else {
        return failed(state, "Open a stack first.");
    };
    let source = state.browser.draft.front.clone();
    if source.trim().is_empty() {
        return failed(state, "Write the text to make cards from first.");
    }
    let effect = Effect::CreateFromType {
        folder,
        type_id,
        source,
    };
    (
        State {
            notice: None,
            ..state
        },
        vec![effect],
    )
}

fn import_clicked(state: State) -> (State, Vec<Effect>) {
    let Some(folder) = stack_of(&state) else {
        return failed(state, "Open a stack first.");
    };
    let path = state.browser.import_path.trim().to_owned();
    if path.is_empty() {
        return failed(state, "Choose the file to import first.");
    }
    (
        State {
            notice: None,
            ..state
        },
        vec![Effect::ImportFile { folder, path }],
    )
}

fn imported(state: State, result: Result<ImportOutcome, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(ImportOutcome { info, report }) => {
            let mut text = format!("Created {} card(s).", report.created);
            if !report.skipped.is_empty() {
                text.push_str(&format!(
                    " Skipped {}: {}",
                    report.skipped.len(),
                    report.skipped.join("; ")
                ));
            }
            let browser = Browser {
                draft: CardDraft::default(),
                import_path: String::new(),
                ..state.browser.clone()
            };
            let effects = reload(&browser.selection);
            let kind = if report.created == 0 {
                NoticeKind::Error
            } else {
                NoticeKind::Info
            };
            (
                State {
                    vault: Some(info),
                    browser,
                    notice: notice(kind, text),
                    ..state
                },
                effects,
            )
        }
        Err(e) => failed(state, &format!("Could not create the cards: {e}")),
    }
}

/// Handle the import and card-type messages.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    let with = |state: State, f: &dyn Fn(Browser) -> Browser| {
        let browser = f(state.browser.clone());
        (State { browser, ..state }, Vec::new())
    };
    match msg {
        Msg::CardTypeChosen(id) => with(state, &|b| Browser {
            card_type: id.clone(),
            ..b
        }),
        Msg::ImportPathEdited(path) => with(state, &|b| Browser {
            import_path: path.clone(),
            ..b
        }),
        Msg::BrowseImportClicked => (state, vec![Effect::PickImportFile]),
        Msg::ImportFilePicked(Some(path)) => with(state, &|b| Browser {
            import_path: path.clone(),
            ..b
        }),
        Msg::ImportClicked => import_clicked(state),
        Msg::Imported(result) => imported(state, result),
        _ => (state, Vec::new()),
    }
}
