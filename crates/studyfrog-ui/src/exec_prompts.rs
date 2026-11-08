//! Writing prompt files. Thin: the texts are the service's.

use std::fs;
use std::path::Path;

use studyfrog_service::prompts::{PROMPTS, clip, file_name, file_text, prompt_with};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::prompt_run::{Attachment, PromptRunMsg};
use crate::prompts_ui::{PromptsEffect, PromptsMsg, pick_refs};

fn write(w: &Worker, ids: &[&str], picks: &[(String, usize)]) -> Result<Option<String>, String> {
    let Some(folder) = w.folder_picker.and_then(|pick| pick()) else {
        return Ok(None);
    };
    let mut written = 0;
    for source in PROMPTS.iter().filter(|s| ids.contains(&s.id)) {
        let prompt = prompt_with(source, &pick_refs(picks));
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

/// The kinds of file offered for attaching: text, source and data files.
const EXTENSIONS: [&str; 17] = [
    "txt", "md", "py", "r", "rs", "php", "java", "toml", "html", "xml", "json", "csv", "tsv",
    "cpp", "c", "h", "hpp",
];

/// Read a text file for the assistant, cut to the limit. Nothing leaves this computer here.
fn read_attachment(path: &str) -> Result<Attachment, String> {
    let name = Path::new(path)
        .file_name()
        .map_or_else(|| path.to_owned(), |n| n.to_string_lossy().into_owned());
    let bytes = fs::read(path).map_err(|e| format!("{name}: {e}"))?;
    if bytes.contains(&0) {
        return Err(format!(
            "{name} is not a text file. For a PDF, copy its text into a .txt file first."
        ));
    }
    let (text, cut) = clip(&String::from_utf8_lossy(&bytes));
    if text.trim().is_empty() {
        return Err(format!("{name} is empty."));
    }
    Ok((name, text, cut))
}

/// The CSV of an answer, written where the import dialog can read it.
fn write_reply(csv: &str) -> Result<String, String> {
    let target =
        std::env::temp_dir().join(format!("studyfrog-ai-answer-{}.csv", std::process::id()));
    fs::write(&target, csv).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(target.display().to_string())
}

fn run_msg(m: PromptRunMsg) -> Msg {
    Msg::Prompts(PromptsMsg::Run(m))
}

/// Perform one job and report.
pub fn run(w: &Worker, effect: PromptsEffect) -> Msg {
    match effect {
        PromptsEffect::Export(ids, picks) => {
            Msg::Prompts(PromptsMsg::Saved(write(w, &ids, &picks)))
        }
        PromptsEffect::Browse => {
            let picked = w.file_picker.and_then(|pick| pick(&EXTENSIONS));
            run_msg(PromptRunMsg::Browsed(
                picked.map(|p| p.display().to_string()),
            ))
        }
        PromptsEffect::Read(path) => run_msg(PromptRunMsg::Read(read_attachment(&path))),
        PromptsEffect::WriteReply(csv) => run_msg(PromptRunMsg::Written(write_reply(&csv))),
    }
}
