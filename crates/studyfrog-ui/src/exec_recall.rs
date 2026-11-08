//! Performing the active-recall effects (EPIC-035) on the worker.

use studyfrog_service::create::fresh_id_source;
use studyfrog_service::inline_sync::{apply_inline_sync, plan_inline_sync};
use studyfrog_service::session::{VaultSession, describe_service_error};

use crate::msg::Msg;
use crate::recall_types::{RecallEffect, RecallMsg};

fn text(e: &studyfrog_service::session::ServiceError) -> String {
    describe_service_error(e)
}

/// Perform one effect and report what happened.
pub fn run(s: &mut VaultSession, effect: RecallEffect, now_ns: i64) -> Msg {
    let now_ms = u64::try_from(now_ns / 1_000_000).unwrap_or(0);
    let m = match effect {
        RecallEffect::PlanExtract(note, body) => RecallMsg::ExtractPlanned(
            plan_inline_sync(s, note, Some(&body))
                .map(Box::new)
                .map_err(|e| text(&e)),
        ),
        RecallEffect::ApplyExtract(plan) => RecallMsg::ExtractSynced(
            fresh_id_source(now_ms)
                .and_then(|ids| apply_inline_sync(s, &plan, ids))
                .map(Box::new)
                .map_err(|e| text(&e)),
        ),
    };
    Msg::Recall(m)
}
