//! Writing the user's own help pages: new, copied from a supplied page, edited, renamed and deleted. Pure.

use studyfrog_service::help::{PageRef, UserPage, title_of};
use studyfrog_service::help_user::{free_name, name_problem};

use crate::effect::Effect;
use crate::help_types::{HelpEffect, HelpMsg, HelpState, PageEdit};
use crate::help_ui::{current, show, text_of};
use crate::state::{Notice, NoticeKind, State};

fn with(state: State, help: HelpState) -> (State, Vec<Effect>) {
    (State { help, ..state }, Vec::new())
}

fn start(state: State, edit: PageEdit) -> (State, Vec<Effect>) {
    if state.vault.is_none() {
        let text = "Open a vault first: your own help pages are kept in it.".to_owned();
        let notice = Some(Notice {
            kind: NoticeKind::Error,
            text,
        });
        return (State { notice, ..state }, Vec::new());
    }
    let help = HelpState {
        edit: Some(edit),
        ask_delete: false,
        ..state.help.clone()
    };
    with(state, help)
}

fn new_page(state: State) -> (State, Vec<Effect>) {
    let name = free_name("My page", &state.help.user);
    let text = format!("# {name}\n\n");
    let edit = PageEdit {
        name,
        text,
        ..PageEdit::default()
    };
    start(state, edit)
}

fn copy_page(state: State) -> (State, Vec<Effect>) {
    let page = current(&state.help);
    let Some(text) = text_of(&state.help, &page).map(str::to_owned) else {
        return (state, Vec::new());
    };
    let title = title_of(&text, "Page").to_owned();
    let name = free_name(&format!("{title} (my copy)"), &state.help.user);
    let edit = PageEdit {
        name,
        text,
        ..PageEdit::default()
    };
    start(state, edit)
}

fn edit_page(state: State) -> (State, Vec<Effect>) {
    let PageRef::User(file) = current(&state.help) else {
        return (state, Vec::new()); // supplied pages are read-only
    };
    let Some(page) = state.help.user.iter().find(|u| u.file == file).cloned() else {
        return (state, Vec::new());
    };
    let edit = PageEdit {
        original: Some(page.file.clone()),
        name: page.file,
        text: page.text,
        ..PageEdit::default()
    };
    start(state, edit)
}

fn change(state: State, f: impl FnOnce(&mut PageEdit)) -> (State, Vec<Effect>) {
    let mut help = state.help.clone();
    if let Some(edit) = help.edit.as_mut() {
        edit.problem = None;
        f(edit);
    }
    with(state, help)
}

fn save(state: State) -> (State, Vec<Effect>) {
    let Some(edit) = state.help.edit.clone().filter(|e| !e.saving) else {
        return (state, Vec::new());
    };
    if let Some(why) = name_problem(&edit.name) {
        return change(state, |e| e.problem = Some(why.to_owned()));
    }
    let effect = Effect::Help(HelpEffect::Save {
        old: edit.original.clone(),
        name: edit.name.trim().to_owned(),
        text: edit.text.clone(),
    });
    let mut help = state.help.clone();
    help.edit = Some(PageEdit {
        saving: true,
        ..edit
    });
    (State { help, ..state }, vec![effect])
}

fn saved(state: State, result: Result<Vec<UserPage>, String>) -> (State, Vec<Effect>) {
    let Some(edit) = state.help.edit.clone() else {
        return (state, Vec::new());
    };
    match result {
        Ok(user) => {
            let page = PageRef::User(edit.name.trim().to_owned());
            let help = HelpState {
                user,
                edit: None,
                ..state.help.clone()
            };
            let notice = Some(Notice {
                kind: NoticeKind::Info,
                text: format!("Saved \"{}\".", edit.name.trim()),
            });
            let help = show(help, page, edit.original.is_none());
            (
                State {
                    help,
                    notice,
                    ..state
                },
                Vec::new(),
            )
        }
        Err(e) => change(state, |ed| {
            ed.saving = false;
            ed.problem = Some(e);
        }),
    }
}

fn delete(state: State) -> (State, Vec<Effect>) {
    match current(&state.help) {
        PageRef::User(file) => (state, vec![Effect::Help(HelpEffect::Delete(file))]),
        PageRef::Supplied(_) => (state, Vec::new()),
    }
}

fn deleted(state: State, result: Result<Vec<UserPage>, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(user) => {
            let first = PageRef::Supplied(crate::help_ui::FIRST_PAGE.to_owned());
            let help = HelpState {
                user,
                back: Vec::new(),
                ..state.help.clone()
            };
            let notice = Some(Notice {
                kind: NoticeKind::Info,
                text: "The page was deleted.".to_owned(),
            });
            let help = show(help, first, false);
            (
                State {
                    help,
                    notice,
                    ..state
                },
                Vec::new(),
            )
        }
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("The page could not be deleted: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}

/// Handle the messages about writing pages.
pub fn update(state: State, msg: HelpMsg) -> (State, Vec<Effect>) {
    match msg {
        HelpMsg::New => new_page(state),
        HelpMsg::Copy => copy_page(state),
        HelpMsg::Edit => edit_page(state),
        HelpMsg::EditName(name) => change(state, |e| e.name = name),
        HelpMsg::EditText(text) => change(state, |e| e.text = text),
        HelpMsg::Save => save(state),
        HelpMsg::Cancel => {
            let help = HelpState {
                edit: None,
                ..state.help.clone()
            };
            with(state, help)
        }
        HelpMsg::Saved(result) => saved(state, result),
        HelpMsg::AskDelete(on) => {
            let help = HelpState {
                ask_delete: on,
                ..state.help.clone()
            };
            with(state, help)
        }
        HelpMsg::Delete => delete(state),
        HelpMsg::Deleted(result) => deleted(state, result),
        _ => (state, Vec::new()),
    }
}
