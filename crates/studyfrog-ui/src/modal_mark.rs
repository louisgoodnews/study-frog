//! Handling modal messages for stack marking and unmarking (ADR 0011, TASK-0217).

use studyfrog_service::bulk::Snapshot;

use crate::effect::Effect;
use crate::modal_types::{Modal, ModalMsg, Toast, ToastAction};
use crate::stack_mark_ui::StackMarkEffect;
use crate::state::{Notice, NoticeKind, Selection, State};

fn toast_undo(title: String, snap: Snapshot) -> Toast {
    Toast {
        title,
        detail: "Click to undo".to_owned(),
        action: ToastAction::UndoMark(snap),
        expires_ms: None,
    }
}

fn toast_info(title: String) -> Toast {
    Toast {
        title,
        detail: String::new(),
        action: ToastAction::Dismiss,
        expires_ms: None,
    }
}

/// Handle stack mark modal messages.
pub fn update(state: State, msg: ModalMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    match msg {
        ModalMsg::AskUnmark(folder) => (
            state,
            vec![Effect::StackMark(StackMarkEffect::PlanUnmark(folder))],
        ),
        ModalMsg::UnmarkPlanLoaded(Ok(plan)) => {
            state.modal = Some(Modal::ConfirmUnmark(plan));
            (state, Vec::new())
        }
        ModalMsg::UnmarkPlanLoaded(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
        ModalMsg::ConfirmUnmark(folder) => {
            state.modal = None;
            (
                state,
                vec![Effect::StackMark(StackMarkEffect::Unmark(folder))],
            )
        }
        ModalMsg::Unmarked(Ok((info, snap, plan))) => {
            state.modal = None;
            state.vault = Some(info);
            if let Selection::Stack(f) = &state.browser.selection
                && f == &plan.folder
            {
                state.browser.selection = plan
                    .target_folder
                    .clone()
                    .map_or(Selection::Nothing, Selection::Stack);
            }
            state.toasts.push(toast_undo(
                format!("Made \"{}\" a normal folder", plan.title),
                snap,
            ));
            (state, Vec::new())
        }
        ModalMsg::Unmarked(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
        ModalMsg::MarkAsStack(folder) => (
            state,
            vec![Effect::StackMark(StackMarkEffect::Mark(folder))],
        ),
        ModalMsg::Marked(Ok((info, snap, created))) => {
            state.vault = Some(info);
            state.toasts.push(toast_undo(
                format!("Made \"{}\" a stack", created.title),
                snap,
            ));
            (state, Vec::new())
        }
        ModalMsg::Marked(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
        ModalMsg::UndoMark(snap) => (state, vec![Effect::StackMark(StackMarkEffect::Undo(snap))]),
        ModalMsg::MarkUndone(Ok(info)) => {
            state.vault = Some(info);
            state
                .toasts
                .push(toast_info("Reverted folder change".to_owned()));
            (state, Vec::new())
        }
        ModalMsg::MarkUndone(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
        ModalMsg::CreatedFolder(Ok((info, name))) => {
            state.modal = None;
            state.vault = Some(info);
            state
                .toasts
                .push(toast_info(format!("Created folder \"{name}\"")));
            (state, Vec::new())
        }
        ModalMsg::CreatedFolder(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
        ModalMsg::SetIsStack(is_stack) => {
            if let Some(Modal::Create(f)) = &state.modal {
                state.modal = Some(Modal::Create(crate::modal_types::CreateForm {
                    is_stack,
                    ..f.clone()
                }));
            }
            (state, Vec::new())
        }
        _ => (state, Vec::new()),
    }
}
