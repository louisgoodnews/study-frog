//! Performing the import and export jobs of the dialog. Thin: the formats and the rules live in the service.

use std::fs;
use std::path::Path;

use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::xfer_adapters::read_source;
use studyfrog_service::xfer_csv::to_csv;
use studyfrog_service::xfer_export::{export_bundle, export_json_zip, export_markdown_zip};
use studyfrog_service::xfer_import::apply_import;
use studyfrog_service::xfer_plan::{counts, plan_import};

use crate::exec::Worker;
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::transfer_ui::{Format, Imported, Preview, TransferJob, TransferMsg};

const SHOWN_LINES: usize = 100;
const EXTENSIONS: [&str; 6] = ["json", "zip", "csv", "tsv", "txt", "apkg"];

fn msg(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

fn preview(w: &Worker, path: &str, target: Option<&str>) -> Result<Box<Preview>, String> {
    let session = w.session.as_ref().ok_or("no vault is open")?;
    let (bundle, adapter) = read_source(Path::new(path))?;
    let plan = plan_import(session, &bundle, target).map_err(|e| e.to_string())?;
    Ok(Box::new(Preview {
        adapter: adapter.to_owned(),
        total: plan.lines.len(),
        counts: counts(&plan),
        new_stacks: plan.new_stacks,
        warnings: plan.warnings.clone(),
        scheduling: bundle.scheduling && bundle.items.iter().any(|i| !i.reviews.is_empty()),
        files: bundle.files.len(),
        lines: plan.lines.into_iter().take(SHOWN_LINES).collect(),
    }))
}

fn import(
    w: &mut Worker,
    job: (
        &str,
        Option<&str>,
        studyfrog_service::xfer_plan::Policy,
        bool,
    ),
    now_ns: i64,
) -> Result<Box<Imported>, String> {
    let (path, target, policy, with_schedule) = job;
    let (bundle, _) = read_source(Path::new(path))?;
    let session = w.session.as_mut().ok_or("no vault is open")?;
    let ids = fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
        .map_err(|e| e.to_string())?;
    let done = apply_import(session, &bundle, target, (policy, with_schedule), ids)
        .map_err(|e| e.to_string())?;
    Ok(Box::new(Imported {
        created: done.created,
        replaced: done.replaced,
        skipped: done.skipped,
        notes: done.notes,
        info: done.info,
    }))
}

fn export(
    w: &Worker,
    job: (&str, Option<&str>, Format, bool),
    now_ns: i64,
) -> Result<String, String> {
    let (folder, scope, format, scheduling) = job;
    let session = w.session.as_ref().ok_or("no vault is open")?;
    if folder.is_empty() || !Path::new(folder).is_dir() {
        return Err("choose a folder to write to".to_owned());
    }
    let day = format_date(date_from_days(now_ns / 1_000_000 / 86_400_000));
    let name = scope.map_or("library".to_owned(), |s| s.replace('/', "-"));
    let exported = format!("{day}T00:00:00Z");
    let (ext, bytes) = match format {
        Format::Json => (
            "zip",
            export_json_zip(
                &export_bundle(session, scope, scheduling).map_err(|e| e.to_string())?,
                &exported,
            ),
        ),
        Format::Csv => (
            "csv",
            export_bundle(session, scope, false).map(|b| to_csv(&b).into_bytes()),
        ),
        Format::Markdown => ("zip", export_markdown_zip(session, scope)),
    };
    let target = Path::new(folder).join(format!("studyfrog-{name}-{day}.{ext}"));
    fs::write(&target, bytes.map_err(|e| e.to_string())?)
        .map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(target.display().to_string())
}

/// Perform one job and report.
pub fn run(w: &mut Worker, job: TransferJob, now_ns: i64) -> Msg {
    match job {
        TransferJob::Browse(folder) => {
            let picked = if folder {
                w.folder_picker.and_then(|p| p())
            } else {
                w.file_picker.and_then(|p| p(&EXTENSIONS))
            };
            msg(TransferMsg::Browsed(
                folder,
                picked.map(|p| p.display().to_string()),
            ))
        }
        TransferJob::Preview(path, target) => {
            msg(TransferMsg::Previewed(preview(w, &path, target.as_deref())))
        }
        TransferJob::Import {
            path,
            target,
            policy,
            with_schedule,
        } => msg(TransferMsg::Imported(import(
            w,
            (&path, target.as_deref(), policy, with_schedule),
            now_ns,
        ))),
        TransferJob::Export {
            folder,
            scope,
            format,
            scheduling,
        } => msg(TransferMsg::Exported(export(
            w,
            (&folder, scope.as_deref(), format, scheduling),
            now_ns,
        ))),
    }
}
