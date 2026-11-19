//! Worker execution for Universal Import Hub jobs (`PreviewHub`, `ImportHub`, `ImportPdf`, `CreateCardType`).

use std::path::Path;

use studyfrog_core::plan::ItemKind;
use studyfrog_ext::registry::find_card_type;
use studyfrog_ext::types::{Faces, ImportedCard};
use studyfrog_service::create::{IdSource, fresh_id_source};
use studyfrog_service::import::{ImportReport, MAX_IMPORT};
use studyfrog_service::session::ServiceError;
use studyfrog_service::track::track_cards;
use studyfrog_service::xfer_csv_map::problem_text;
use studyfrog_service::xfer_import::{ImportTarget, Policy, apply_import_to, plan_import_to};
use studyfrog_service::xfer_model::{Bundle, item};
use studyfrog_service::xfer_pdf::import_pdf_to;
use studyfrog_service::xfer_plan::{ImportPlan, counts};

use crate::exec::Worker;
use crate::import_hub::{HubJob, HubMsg, HubTarget};
use crate::import_ui::ImportOutcome;
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::transfer_ui::{CsvInfo, Imported, Preview, TransferMsg};

const SHOWN_LINES: usize = 100;

fn msg(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

fn ids(now_ns: i64) -> Result<IdSource, ServiceError> {
    fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
}

fn to_service_target(target: &HubTarget) -> ImportTarget {
    match target {
        HubTarget::Existing(None) => ImportTarget::FromSource,
        HubTarget::Existing(Some(f)) => ImportTarget::ExistingStack(f.clone()),
        HubTarget::NewStack(n) => ImportTarget::NewStack(n.clone()),
        HubTarget::UnstackedNotes => ImportTarget::VaultRootNotes,
    }
}

pub(crate) fn build_preview(
    plan: ImportPlan,
    bundle: &Bundle,
    adapter: String,
    csv: Option<Box<CsvInfo>>,
) -> Box<Preview> {
    Box::new(Preview {
        adapter,
        total: plan.lines.len(),
        counts: counts(&plan),
        new_stacks: plan.new_stacks,
        warnings: plan
            .warnings
            .iter()
            .filter(|w| {
                !csv.as_ref()
                    .is_some_and(|c| c.problems.iter().any(|p| problem_text(p) == **w))
            })
            .cloned()
            .collect(),
        scheduling: bundle.scheduling && bundle.items.iter().any(|i| !i.reviews.is_empty()),
        files: bundle.files.len(),
        lines: plan.lines.into_iter().take(SHOWN_LINES).collect(),
        csv,
    })
}

fn preview_hub(
    w: &Worker,
    path: &str,
    target: &HubTarget,
    csv: &studyfrog_service::xfer_csv_map::CsvOptions,
) -> Result<Box<crate::transfer_types::Preview>, String> {
    let session = w.session.as_ref().ok_or("no vault is open")?;
    let (bundle, adapter, csv) = crate::exec_transfer::read_any(w, path, csv)?;
    let plan =
        plan_import_to(session, &bundle, &to_service_target(target)).map_err(|e| e.to_string())?;
    Ok(build_preview(plan, &bundle, adapter, csv))
}

/// How an import treats duplicates and whether it keeps review history.
struct ImportOpts {
    policy: Policy,
    with_schedule: bool,
}

fn import_hub(
    w: &mut Worker,
    path: &str,
    target: &HubTarget,
    opts: ImportOpts,
    csv: &studyfrog_service::xfer_csv_map::CsvOptions,
    now_ns: i64,
) -> Result<Box<Imported>, String> {
    let (bundle, _, _) = crate::exec_transfer::read_any(w, path, csv)?;
    let session = w.session.as_mut().ok_or("no vault is open")?;
    let id = ids(now_ns).map_err(|e| e.to_string())?;
    let done = apply_import_to(
        session,
        &bundle,
        &to_service_target(target),
        (opts.policy, opts.with_schedule),
        id,
    )
    .map_err(|e| e.to_string())?;
    Ok(Box::new(Imported {
        created: done.created,
        replaced: done.replaced,
        skipped: done.skipped,
        notes: done.notes,
        info: done.info,
    }))
}

fn import_pdf(w: &mut Worker, path: &str, target: &HubTarget, now_ns: i64) -> Msg {
    let Some(session) = w.session.as_mut() else {
        return msg(TransferMsg::Hub(Box::new(HubMsg::PdfImported(Err(
            "no vault is open".to_owned(),
        )))));
    };
    let res = ids(now_ns)
        .and_then(|id| {
            import_pdf_to(
                session,
                Path::new(path),
                &to_service_target(target),
                id,
                now_ns,
            )
        })
        .map(|(info, rel)| (rel, info))
        .map_err(|e| e.to_string());
    msg(TransferMsg::Hub(Box::new(HubMsg::PdfImported(res))))
}

fn create_card_type(
    w: &mut Worker,
    target: &HubTarget,
    type_id: &str,
    source: &str,
    now_ns: i64,
) -> Msg {
    let reply = |r| msg(TransferMsg::Hub(Box::new(HubMsg::CardTypeCreated(r))));
    let Some(card_type) = find_card_type(&w.registry, type_id).copied() else {
        return reply(Err(format!("card type \"{type_id}\" is not available")));
    };
    let faces = match (card_type.expand)(source) {
        Ok(f) => f,
        Err(e) => return reply(Err(format!("{}: {e}", card_type.name))),
    };
    if faces.len() > MAX_IMPORT {
        return reply(Err(format!(
            "that would create {} cards; one import is limited to {MAX_IMPORT}",
            faces.len()
        )));
    }
    let Some(session) = w.session.as_mut() else {
        return reply(Err("no vault is open".to_owned()));
    };
    let svc_target = to_service_target(target);
    let items = faces
        .into_iter()
        .map(|Faces { front, back }| item(ItemKind::Card, "", &front, &back))
        .collect();
    let bundle = Bundle {
        items,
        ..Bundle::default()
    };
    let res = ids(now_ns)
        .and_then(|id| {
            let plan = plan_import_to(session, &bundle, &svc_target)?;
            let done =
                apply_import_to(session, &bundle, &svc_target, (Policy::KeepBoth, false), id)?;
            let folder = plan
                .lines
                .first()
                .map_or_else(String::new, |l| l.folder.clone());
            let titles: Vec<String> = plan.lines.iter().map(|l| l.title.clone()).collect();
            if !folder.is_empty() {
                track_cards(session, &done.info, (&folder, &titles), id);
            }
            let report = ImportReport {
                created: done.created,
                titles,
                skipped: Vec::new(),
            };
            Ok(ImportOutcome {
                info: done.info,
                report,
            })
        })
        .map_err(|e| e.to_string());
    reply(res)
}

/// Convert [`ImportedCard`] rows from an extension file importer into a [`Bundle`].
pub(crate) fn cards_bundle(cards: Vec<ImportedCard>) -> Bundle {
    let items = cards
        .into_iter()
        .map(|c| {
            let mut it = item(ItemKind::Card, "", &c.front, &c.back);
            it.title = c.title;
            it.tags = c.tags;
            it
        })
        .collect();
    Bundle {
        items,
        ..Bundle::default()
    }
}

/// Execute one [`HubJob`] and return the resulting [`Msg`].
pub fn run(w: &mut Worker, job: HubJob, now_ns: i64) -> Msg {
    match job {
        HubJob::PreviewHub(path, target, csv) => {
            msg(TransferMsg::Previewed(preview_hub(w, &path, &target, &csv)))
        }
        HubJob::ImportHub(path, target, policy, with_schedule, csv) => {
            msg(TransferMsg::Imported(import_hub(
                w,
                &path,
                &target,
                ImportOpts {
                    policy,
                    with_schedule,
                },
                &csv,
                now_ns,
            )))
        }
        HubJob::ImportPdf(path, target) => import_pdf(w, &path, &target, now_ns),
        HubJob::CreateCardType(target, type_id, source) => {
            create_card_type(w, &target, &type_id, &source, now_ns)
        }
    }
}
