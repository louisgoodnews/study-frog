//! The modal dialogs (create an item, open a vault) and the toasts that can open them. Pure.

use studyfrog_service::create_more::NewKind;

use crate::browse::reload;
use crate::effect::Effect;
use crate::modal_form::{due_of, tags_of, toggled};
use crate::state::{Notice, NoticeKind, Screen, Selection, State};

pub use crate::modal_types::{
    CreateForm, CreateOutcome, Field, Modal, ModalMsg, Toast, ToastAction, problem, stack_labels,
};

fn default_parent(state: &State) -> Option<String> {
    match &state.browser.selection {
        Selection::Stack(f) => Some(f.clone()),
        _ => state
            .vault
            .as_ref()
            .and_then(|v| v.overview.stacks.first().map(|s| s.folder.clone())),
    }
}

fn form_new(state: &State, kind: Option<NewKind>) -> CreateForm {
    let has_stacks = state
        .vault
        .as_ref()
        .is_some_and(|v| !v.overview.stacks.is_empty());
    CreateForm {
        kind: kind.unwrap_or(if has_stacks {
            NewKind::Card
        } else {
            NewKind::Stack
        }),
        title: String::new(),
        text: String::new(),
        answer: String::new(),
        tags: String::new(),
        parent: default_parent(state),
        another: false,
        error: None,
        busy: false,
        due: String::new(),
        reminders: vec![1440],
        priority: studyfrog_core::priority::Priority::Medium,
        difficulty: studyfrog_core::event::Difficulty::Difficult,
    }
}

fn with_form(state: State, f: impl FnOnce(CreateForm) -> CreateForm) -> (State, Vec<Effect>) {
    let modal = match &state.modal {
        Some(Modal::Create(form)) => Some(Modal::Create(f(form.clone()))),
        other => other.clone(),
    };
    (State { modal, ..state }, Vec::new())
}

fn submit(state: State) -> (State, Vec<Effect>) {
    let Some(Modal::Create(form)) = state.modal.clone() else {
        return (state, Vec::new());
    };
    if form.busy {
        return (state, Vec::new());
    }
    if let Some(why) = problem(&form) {
        return with_form(state, |f| CreateForm {
            error: Some(why.to_owned()),
            ..f
        });
    }
    let effect = Effect::CreateItem {
        kind: form.kind,
        title: form.title.trim().to_owned(),
        text: form.text.clone(),
        answer: form.answer.clone(),
        tags: tags_of(&form.tags),
        parent: form.parent.clone(),
        due: due_of(&form),
        choices: if form.kind == NewKind::Stack {
            (None, None)
        } else {
            (Some(form.priority), Some(form.difficulty))
        },
    };
    let (state, _) = with_form(state, |f| CreateForm {
        busy: true,
        error: None,
        ..f
    });
    (state, vec![effect])
}

fn created(state: State, result: Result<Box<CreateOutcome>, String>) -> (State, Vec<Effect>) {
    let Some(Modal::Create(form)) = state.modal.clone() else {
        return (state, Vec::new());
    };
    match result {
        Err(e) => with_form(state, |f| CreateForm {
            busy: false,
            error: Some(e),
            ..f
        }),
        Ok(outcome) => {
            let CreateOutcome {
                info,
                created,
                warnings,
            } = *outcome;
            let notice = Some(match warnings.first() {
                Some(w) => Notice {
                    kind: NoticeKind::Error,
                    text: format!("Created \"{}\", but {w}", created.title),
                },
                None => Notice {
                    kind: NoticeKind::Info,
                    text: format!("Created \"{}\".", created.title),
                },
            });
            let new_stack = (form.kind == NewKind::Stack && !form.another)
                .then(|| created.path.trim_end_matches("/_stack.md").to_owned());
            let mut state = State {
                vault: Some(info),
                notice,
                ..state
            };
            state.modal = if form.another {
                let kept = CreateForm {
                    title: String::new(),
                    text: String::new(),
                    answer: String::new(),
                    busy: false,
                    error: None,
                    ..form
                };
                Some(Modal::Create(kept))
            } else {
                None
            };
            if let Some(folder) = new_stack {
                state.browser.selection = Selection::Stack(folder);
                state.screen = Screen::Library;
            }
            let mut effects = reload(&state.browser.selection);
            effects.extend([Effect::LoadBoard, Effect::LoadSessions]);
            (state, effects)
        }
    }
}

fn click_toast(state: State, i: usize) -> (State, Vec<Effect>) {
    let Some(toast) = state.toasts.get(i).cloned() else {
        return (state, Vec::new());
    };
    let mut state = state;
    state.toasts.remove(i);
    match toast.action {
        ToastAction::OpenVaultDialog => state.modal = Some(Modal::Vault),
        ToastAction::OpenCalendar => return crate::nav::navigate(state, Screen::Calendar),
    }
    (state, Vec::new())
}

/// Handle the modal and toast messages.
pub fn update(state: State, msg: ModalMsg) -> (State, Vec<Effect>) {
    match msg {
        ModalMsg::OpenCreate => {
            let modal = Some(Modal::Create(form_new(&state, None)));
            (State { modal, ..state }, Vec::new())
        }
        ModalMsg::OpenCreateStack => {
            let modal = Some(Modal::Create(form_new(&state, Some(NewKind::Stack))));
            (State { modal, ..state }, Vec::new())
        }
        ModalMsg::OpenVault => (
            State {
                modal: Some(Modal::Vault),
                ..state
            },
            Vec::new(),
        ),
        ModalMsg::OpenImport => (
            State {
                modal: Some(Modal::Import),
                ..state
            },
            Vec::new(),
        ),
        ModalMsg::Close => (
            State {
                modal: None,
                ..state
            },
            Vec::new(),
        ),
        ModalMsg::SetKind(kind) => with_form(state, |f| CreateForm {
            kind,
            error: None,
            ..f
        }),
        ModalMsg::Edit(field, text) => with_form(state, |f| match field {
            Field::Title => CreateForm {
                title: text,
                error: None,
                ..f
            },
            Field::Text => CreateForm {
                text,
                error: None,
                ..f
            },
            Field::Answer => CreateForm {
                answer: text,
                error: None,
                ..f
            },
            Field::Tags => CreateForm { tags: text, ..f },
            Field::Due => CreateForm {
                due: text,
                error: None,
                ..f
            },
        }),
        ModalMsg::SetParent(parent) => with_form(state, |f| CreateForm {
            parent,
            error: None,
            ..f
        }),
        ModalMsg::SetAnother(another) => with_form(state, |f| CreateForm { another, ..f }),
        ModalMsg::SetPriority(priority) => with_form(state, |f| CreateForm { priority, ..f }),
        ModalMsg::SetDifficulty(difficulty) => with_form(state, |f| CreateForm { difficulty, ..f }),
        ModalMsg::ToggleReminder(m) => with_form(state, |f| CreateForm {
            reminders: toggled(&f.reminders, m),
            ..f
        }),
        ModalMsg::Submit => submit(state),
        ModalMsg::Created(result) => created(state, result),
        ModalMsg::ToastClicked(i) => click_toast(state, i),
        ModalMsg::ToastDismissed(i) => {
            let mut state = state;
            if i < state.toasts.len() {
                state.toasts.remove(i);
            }
            (state, Vec::new())
        }
    }
}
