//! Writing prompt files. Thin: the texts are the service's.

use std::fs;

use studyfrog_service::prompts::{PROMPTS, file_name, file_text, prompt_of};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::prompts_ui::{PromptsEffect, PromptsMsg};

fn write(w: &Worker, ids: &[&str]) -> Result<Option<String>, String> {
    let Some(folder) = w.folder_picker.and_then(|pick| pick()) else {
        return Ok(None);
    };
    let mut written = 0;
    for source in PROMPTS.iter().filter(|s| ids.contains(&s.id)) {
        let prompt = prompt_of(source);
        let target = folder.join(file_name(&prompt));
        fs::write(&target, file_text(&prompt)).map_err(|e| format!("{}: {e}", target.display()))?;
        written += 1;
    }
    let noun = if written == 1 { "prompt" } else { "prompts" };
    Ok(Some(format!(
        "Saved {written} {noun} to {}.",
        folder.display()
    )))
}

/// Perform one job and report.
pub fn run(w: &Worker, effect: PromptsEffect) -> Msg {
    let PromptsEffect::Export(ids) = effect;
    Msg::Prompts(PromptsMsg::Saved(write(w, &ids)))
}
