//! Fetching a [`Plan`] into a folder, only after the user has confirmed it (TASK-0298). A file already in the folder
//! with the expected digest is kept; an interrupted file resumes on the next call; a file with the wrong digest is
//! removed by the downloader and reported.

use std::fs;
use std::path::Path;

use crate::model_download::{DownloadProgress, download_resumable, verify_file_sha256};
use crate::model_plan::Plan;

/// Fetches every file of `plan` into `dir`. `confirm` is asked first, with the whole plan (its title, source and sizes);
/// when it answers no, nothing is contacted and nothing is written. `on_progress` receives the file name and progress.
pub fn fetch_confirmed<C, F>(
    plan: &Plan,
    dir: &Path,
    confirm: C,
    mut on_progress: F,
) -> Result<(), String>
where
    C: FnOnce(&Plan) -> bool,
    F: FnMut(&str, DownloadProgress),
{
    if !confirm(plan) {
        return Err("the download was not confirmed".to_string());
    }
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    for file in &plan.files {
        let dest = dir.join(&file.name);
        if verify_file_sha256(&dest, &file.sha256).unwrap_or(false) {
            continue;
        }
        download_resumable(&file.url, &dest, Some(&file.sha256), |p| {
            on_progress(&file.name, p)
        })
        .map_err(|e| format!("{}: {e}", file.name))?;
    }
    Ok(())
}
