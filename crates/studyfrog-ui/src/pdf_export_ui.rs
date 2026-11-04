//! State updates, transitions, and effects for the PDF export dialog.

use crate::effect::Effect;
use crate::modal::{Modal, Toast, ToastAction};
use crate::notify_types::NotifyTarget;
use crate::pdf_effect::PdfEffect;
use crate::pdf_export_types::{
    PdfExportJob, PdfExportMsg, PdfExportState, default_destination, default_pdf_export_state,
    resolve_destination_path,
};
use crate::pdf_types::PdfView;
use crate::state::{Notice, NoticeKind, Screen, State};

fn request_preview(form: &mut PdfExportState) -> Vec<Effect> {
    form.previewing = true;
    vec![Effect::Pdf(PdfEffect::Export(PdfExportJob::Preview {
        target: form.target.clone(),
        options: form.options.clone(),
    }))]
}

fn handle_submit(
    vault: Option<&crate::state::VaultInfo>,
    form: &mut PdfExportState,
) -> Vec<Effect> {
    let dest_str = form.destination.trim();
    if dest_str.is_empty() {
        form.error = Some("Choose a destination file name.".to_string());
        return Vec::new();
    }
    let dest = resolve_destination_path(vault, dest_str);
    form.busy = true;
    form.error = None;
    vec![Effect::Pdf(PdfEffect::Export(PdfExportJob::Export {
        target: form.target.clone(),
        options: form.options.clone(),
        destination: dest,
    }))]
}

fn handle_exported(
    state: State,
    form: &mut PdfExportState,
    result: Result<studyfrog_service::pdf_export::PdfExportOutcome, String>,
) -> (State, Vec<Effect>) {
    form.busy = false;
    match result {
        Ok(outcome) => {
            let dest_str = outcome.destination.display().to_string();
            let summary = format!(
                "Exported {} pages ({} items) to {}.",
                outcome.pages,
                outcome.item_count,
                outcome.destination.display()
            );
            form.status = Some(summary.clone());
            form.error = None;

            let mut state = state;
            let stem = outcome
                .destination
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("PDF Export")
                .to_string();

            state.pdf = Some(PdfView {
                title: stem,
                path: dest_str.clone(),
                page: 0,
                color: "yellow".to_owned(),
                ..Default::default()
            });

            state.toasts.push(Toast {
                title: "PDF exported".to_string(),
                detail: format!("Saved to {}", outcome.destination.display()),
                action: ToastAction::OpenPdf(dest_str),
                expires_ms: Some(state.wall_ms + 10_000),
            });

            let (state, fx) = crate::notify::push(
                state,
                (NoticeKind::Info, summary),
                Some(NotifyTarget::Screen(Screen::Pdf)),
            );
            (state, fx)
        }
        Err(e) => {
            form.error = Some(format!("Export failed: {e}"));
            let mut state = state;
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Export failed: {e}"),
            });
            (state, Vec::new())
        }
    }
}

/// Handle a message of the PDF export dialog.
pub fn update(state: State, msg: PdfExportMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    match msg {
        PdfExportMsg::Open(target) => {
            let dest = default_destination(&state, &target);
            let mut form = default_pdf_export_state(target, dest);
            let fx = request_preview(&mut form);
            state.modal = Some(Modal::PdfExport(Box::new(form)));
            return (state, fx);
        }
        PdfExportMsg::Close => {
            state.modal = None;
            return (state, Vec::new());
        }
        _ => {}
    }

    if let PdfExportMsg::Exported(result) = msg {
        let Some(Modal::PdfExport(mut form)) = state.modal.take() else {
            return (state, Vec::new());
        };
        let (mut state, fx) = handle_exported(state, &mut form, result);
        state.modal = Some(Modal::PdfExport(form));
        return (state, fx);
    }

    if let PdfExportMsg::Submit = msg {
        let dest_str = match &state.modal {
            Some(Modal::PdfExport(form)) => form.destination.trim().to_string(),
            _ => return (state, Vec::new()),
        };
        if dest_str.is_empty() {
            if let Some(Modal::PdfExport(ref mut form)) = state.modal {
                form.error = Some("Choose a destination file name.".to_string());
            }
            return (state, Vec::new());
        }
        let dest = resolve_destination_path(state.vault.as_ref(), &dest_str);
        let Some(Modal::PdfExport(ref mut form)) = state.modal else {
            return (state, Vec::new());
        };
        form.busy = true;
        form.error = None;
        let fx = vec![Effect::Pdf(PdfEffect::Export(PdfExportJob::Export {
            target: form.target.clone(),
            options: form.options.clone(),
            destination: dest,
        }))];
        return (state, fx);
    }

    let Some(Modal::PdfExport(ref mut form)) = state.modal else {
        return (state, Vec::new());
    };

    match msg {
        PdfExportMsg::SetPageSize(size) => {
            form.options.page_size = size;
            let fx = request_preview(form);
            (state, fx)
        }
        PdfExportMsg::SetMargins(margins) => {
            form.options.margins = margins;
            let fx = request_preview(form);
            (state, fx)
        }
        PdfExportMsg::SetFontSize(size) => {
            form.options.font_size = size;
            let fx = request_preview(form);
            (state, fx)
        }
        PdfExportMsg::SetTitlePage(val) => {
            form.options.title_page = val;
            let fx = request_preview(form);
            (state, fx)
        }
        PdfExportMsg::SetToc(val) => {
            form.options.table_of_contents = val;
            let fx = request_preview(form);
            (state, fx)
        }
        PdfExportMsg::SetPageNumbers(val) => {
            form.options.page_numbers = val;
            let fx = request_preview(form);
            (state, fx)
        }
        PdfExportMsg::SetShowAnswers(val) => {
            form.options.show_answers = val;
            let fx = request_preview(form);
            (state, fx)
        }
        PdfExportMsg::EditDestination(dest) => {
            form.destination = dest;
            form.error = None;
            (state, Vec::new())
        }
        PdfExportMsg::BrowseDestination => (
            state,
            vec![Effect::Pdf(PdfEffect::Export(PdfExportJob::Browse))],
        ),
        PdfExportMsg::DestinationPicked(picked) => {
            if let Some(dest) = picked {
                form.destination = dest;
                form.error = None;
            }
            (state, Vec::new())
        }
        PdfExportMsg::PreviewReady(result) => {
            form.previewing = false;
            match result {
                Ok(p) => {
                    form.preview = Some(p);
                    form.error = None;
                }
                Err(e) => {
                    form.error = Some(e);
                    form.preview = None;
                }
            }
            (state, Vec::new())
        }
        PdfExportMsg::Submit => {
            let vault = state.vault.clone();
            let fx = handle_submit(vault.as_ref(), form);
            (state, fx)
        }
        PdfExportMsg::Exported(_) => (state, Vec::new()),
        PdfExportMsg::Open(_) | PdfExportMsg::Close => (state, Vec::new()),
    }
}
