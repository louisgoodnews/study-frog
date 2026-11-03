//! Performing the import and export jobs of the dialog. Thin: the formats and the rules live in the service.

use std::fs;
use std::path::Path;

use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::xfer_adapters::{is_csv_path, read_csv_source, read_source};
use studyfrog_service::xfer_csv::to_csv;
use studyfrog_service::xfer_csv_map::{CsvOptions, problem_text};
use studyfrog_service::xfer_export::{export_bundle, export_json_zip, export_markdown_zip};
use studyfrog_service::xfer_import::apply_import;
use studyfrog_service::xfer_model::Bundle;
use studyfrog_service::xfer_plan::{counts, plan_import};

use crate::exec::Worker;
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::transfer_ui::{CsvInfo, Format, Imported, Preview, TransferJob, TransferMsg};

const SHOWN_LINES: usize = 100;
const EXTENSIONS: [&str; 6] = ["json", "zip", "csv", "tsv", "txt", "apkg"];

fn msg(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

/// A file read: its items, the name of the reader and, for CSV, how it was read.
type Read = (Bundle, String, Option<Box<CsvInfo>>);

/// Read the file; a CSV file with the user's choices, and then also how it was read.
fn read(path: &str, csv: &CsvOptions) -> Result<Read, String> {
    let path = Path::new(path);
    if !is_csv_path(path) {
        return read_source(path).map(|(b, a)| (b, a.to_owned(), None));
    }
    let r = read_csv_source(path, csv)?;
    let info = CsvInfo {
        sheet: r.sheet,
        problems: r.problems,
    };
    Ok((
        r.bundle,
        "CSV or TSV cards".to_owned(),
        Some(Box::new(info)),
    ))
}

fn preview(
    w: &Worker,
    path: &str,
    (target, csv): (Option<&str>, &CsvOptions),
) -> Result<Box<Preview>, String> {
    let session = w.session.as_ref().ok_or("no vault is open")?;
    let (bundle, adapter, csv) = read(path, csv)?;
    let plan = plan_import(session, &bundle, target).map_err(|e| e.to_string())?;
    Ok(Box::new(Preview {
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
    }))
}

fn import(
    w: &mut Worker,
    job: (
        &str,
        Option<&str>,
        studyfrog_service::xfer_plan::Policy,
        bool,
        &CsvOptions,
    ),
    now_ns: i64,
) -> Result<Box<Imported>, String> {
    let (path, target, policy, with_schedule, csv) = job;
    let (bundle, _, _) = read(path, csv)?;
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

/// Write a sample file into `folder`, or the user's Documents folder, or the home folder.
fn save_example(folder: &str, file: &str, text: &str) -> Result<String, String> {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let documents = home.as_ref().map(|h| h.join("Documents"));
    let dir = [Some(Path::new(folder).to_path_buf()), documents, home]
        .into_iter()
        .flatten()
        .find(|d| !d.as_os_str().is_empty() && d.is_dir())
        .ok_or("there is no folder to write to; choose one under Export")?;
    let target = dir.join(file);
    fs::write(&target, text).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(target.display().to_string())
}

/// Perform one job and report.
pub fn run(w: &mut Worker, job: TransferJob, now_ns: i64) -> Msg {
    match job {
        TransferJob::Discover => crate::transfer_found::look(),
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
        TransferJob::SaveExample { folder, file, text } => msg(TransferMsg::ExampleSaved(
            save_example(&folder, &file, &text),
        )),
        TransferJob::Preview(path, target, csv) => msg(TransferMsg::Previewed(preview(
            w,
            &path,
            (target.as_deref(), &csv),
        ))),
        TransferJob::Import {
            path,
            target,
            policy,
            with_schedule,
            csv,
        } => msg(TransferMsg::Imported(import(
            w,
            (&path, target.as_deref(), policy, with_schedule, &csv),
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
