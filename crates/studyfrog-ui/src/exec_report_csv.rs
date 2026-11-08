//! Writing the reports as a CSV file (STORY-0098): every report, every stack's numbers and the
//! totals, into the vault's `exports` folder or a folder the user chooses.

use std::path::PathBuf;

use studyfrog_core::time::UnixMs;
use studyfrog_service::reports::build_reports_local;
use studyfrog_service::timezone::local_offset_minutes;

use crate::exec::Worker;

/// Write the reports as a CSV file, into a folder chosen in the folder dialog when `choose` is set.
pub fn report_csv_to(w: &Worker, now_ns: i64, choose: bool) -> Result<String, String> {
    if !choose {
        return report_csv_file(w, now_ns, None);
    }
    match w.folder_picker.and_then(|pick| pick()) {
        Some(dir) => report_csv_file(w, now_ns, Some(dir)),
        None => Err("No folder was chosen, so the report was not written.".to_owned()),
    }
}

/// Write the reports, with every stack's numbers and the totals, as a CSV file into `folder` or
/// else the vault's `exports` folder; the path, or why not.
pub fn report_csv_file(w: &Worker, now_ns: i64, folder: Option<PathBuf>) -> Result<String, String> {
    use studyfrog_core::civil::{date_from_days, format_date};
    use studyfrog_service::report_csv::{ReportCsvData, data_of, report_csv, report_csv_name};
    let session = w.session.as_ref().ok_or("no vault is open")?;
    let now = UnixMs(now_ns / 1_000_000);
    let bundle = build_reports_local(session, now).map_err(|e| e.to_string())?;
    let extras =
        studyfrog_service::report_extras::report_extras(session, now, local_offset_minutes(now))
            .map_err(|e| e.to_string())?;
    let day = format_date(date_from_days(now.0.div_euclid(86_400_000)));
    let vault = session.root.file_name().map_or_else(
        || "vault".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let dir = folder.unwrap_or_else(|| session.root.join("exports"));
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let target = dir.join(report_csv_name(&vault, &day));
    let data = ReportCsvData {
        extras: Some(&extras),
        ..data_of(&bundle)
    };
    std::fs::write(&target, report_csv(&data)).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(target.display().to_string())
}
