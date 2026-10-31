//! Making a card or a question from a PDF passage on the worker.

use studyfrog_service::create::fresh_id_source;
use studyfrog_service::create_more::NewKind;
use studyfrog_service::pdf_notes::{ItemRequest, item_from_pdf};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::pdf_draft_types::{DraftKind, DraftMsg, ItemJob};
use crate::pdf_types::PdfMsg;

/// Make the item.
pub fn make(w: &mut Worker, job: &ItemJob, now_ns: i64) -> Msg {
    let result = match &mut w.session {
        Some(s) => fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
            .and_then(|ids| {
                let req = ItemRequest {
                    kind: if job.kind == DraftKind::Card {
                        NewKind::Card
                    } else {
                        NewKind::Question
                    },
                    pdf: &job.path,
                    page: job.page,
                    front: &job.front,
                    back: &job.back,
                    folder: &job.folder,
                };
                item_from_pdf(s, &req, ids)
            })
            .map(|(info, made)| Box::new((info, made.path)))
            .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Pdf(PdfMsg::Draft(DraftMsg::Made(result)))
}
