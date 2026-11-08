//! What the worker writes to the application log besides notices (STORY-0122): every effect that
//! failed, import and export outcomes with their diagnostics and warnings, and effects that took
//! long. Pure: the lines are chosen here and written by `exec_log`.

use std::fmt::Write as _;

use studyfrog_service::logging::LogLevel;

use crate::effect::Effect;
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::pdf_types::PdfMsg;
use crate::transfer_ui::TransferMsg;

/// An effect slower than this is logged as a warning.
pub const SLOW_MS: u128 = 2_000;
/// Diagnostics kept per outcome, so one large import cannot flood the log.
const MOST_LINES: usize = 20;

/// A writer that keeps the first `left` bytes and then refuses, so naming an effect never formats
/// all of a large one.
struct Capped {
    text: String,
    left: usize,
}

impl std::fmt::Write for Capped {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let take: String = s.chars().take(self.left).collect();
        self.left -= take.chars().count();
        self.text.push_str(&take);
        if self.left == 0 {
            Err(std::fmt::Error)
        } else {
            Ok(())
        }
    }
}

/// The effect's name: its variant, and the inner variants of the grouped ones (`Pdf.Render`).
pub fn effect_label(effect: &Effect) -> String {
    let mut c = Capped {
        text: String::new(),
        left: 48,
    };
    let _ = write!(c, "{effect:?}");
    let end = c
        .text
        .find(|ch: char| !(ch.is_alphanumeric() || ch == '(' || ch == '_'))
        .unwrap_or(c.text.len());
    let parts = c.text[..end].split('(');
    let names: Vec<&str> = parts
        .take_while(|p| p.starts_with(|ch: char| ch.is_ascii_uppercase()))
        .collect();
    names.join(".")
}

/// The error a message carries, for the messages that answer an effect with a `Result`.
fn problem_of(msg: &Msg) -> Option<&str> {
    let e = match msg {
        Msg::Imported(Err(e))
        | Msg::VaultOpened(Err(e))
        | Msg::VaultPolled(Err(e))
        | Msg::AppSettingsSaved(Err(e))
        | Msg::ItemsLoaded(Err(e))
        | Msg::Created(Err(e))
        | Msg::RunReady(Err(e))
        | Msg::ItemLoaded(_, Err(e))
        | Msg::Rerated(_, Err(e))
        | Msg::StepRecorded(Err(e))
        | Msg::DocumentLoaded(_, Err(e))
        | Msg::Saved(Err(e))
        | Msg::GraphLoaded(Err(e))
        | Msg::ReportsBuilt(Err(e))
        | Msg::AchievementsChecked(Err(e))
        | Msg::Pdf(
            PdfMsg::Opened(_, Err(e))
            | PdfMsg::Rendered(_, _, Err(e))
            | PdfMsg::TextLoaded(_, _, Err(e)),
        ) => e,
        Msg::Library(LibraryMsg::Transfer(
            TransferMsg::Exported(Err(e))
            | TransferMsg::ExampleSaved(Err(e))
            | TransferMsg::Previewed(Err(e))
            | TransferMsg::Imported(Err(e)),
        )) => e,
        _ => return None,
    };
    Some(e.as_str())
}

/// Import and export outcomes, with their diagnostics.
fn transfer_lines(msg: &Msg, out: &mut Vec<(LogLevel, String)>) {
    let mut push = |level, text: String| {
        if out.len() < MOST_LINES {
            out.push((level, text));
        }
    };
    match msg {
        Msg::Imported(Ok(o)) => {
            let r = &o.report;
            push(
                LogLevel::Info,
                format!(
                    "import: {} card(s) written, {} skipped",
                    r.created,
                    r.skipped.len()
                ),
            );
            r.skipped
                .iter()
                .for_each(|s| push(LogLevel::Warn, format!("import skipped: {s}")));
        }
        Msg::Library(LibraryMsg::Transfer(t)) => match t {
            TransferMsg::Exported(Ok(path)) => {
                push(LogLevel::Info, format!("export written to {path}"))
            }
            TransferMsg::Previewed(Ok(p)) => {
                p.warnings
                    .iter()
                    .for_each(|w| push(LogLevel::Warn, format!("import preview: {w}")));
            }
            TransferMsg::Imported(Ok(i)) => {
                let counts = (i.created, i.replaced, i.skipped);
                push(
                    LogLevel::Info,
                    format!(
                        "import: {} new, {} replaced, {} skipped",
                        counts.0, counts.1, counts.2
                    ),
                );
                i.notes
                    .iter()
                    .for_each(|n| push(LogLevel::Info, format!("import: {n}")));
            }
            _ => {}
        },
        _ => {}
    }
}

/// The log lines for one effect: what it answered and how long it took.
pub fn worker_events(label: &str, msgs: &[Msg], took_ms: u128) -> Vec<(LogLevel, String)> {
    let mut out = Vec::new();
    for msg in msgs {
        if let Some(e) = problem_of(msg) {
            out.push((LogLevel::Warn, format!("{label} failed: {e}")));
        }
        transfer_lines(msg, &mut out);
    }
    if took_ms >= SLOW_MS {
        out.push((LogLevel::Warn, format!("{label} took {took_ms} ms")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transfer(m: TransferMsg) -> Msg {
        Msg::Library(LibraryMsg::Transfer(m))
    }

    #[test]
    fn an_effect_is_named_by_its_variants_without_its_data() {
        let big = Effect::OpenVault("x".repeat(10_000));
        assert_eq!(effect_label(&big), "OpenVault");
        let csv = Effect::Library(crate::library_bulk::LibraryEffect::Transfer(
            crate::transfer_types::TransferJob::ReportCsv(true),
        ));
        assert_eq!(effect_label(&csv), "Library.Transfer.ReportCsv");
    }

    #[test]
    fn failures_exports_and_slow_effects_become_lines_and_quiet_ones_none() {
        let failed = [Msg::VaultOpened(Err("no such folder".into()))];
        let lines = worker_events("OpenVault", &failed, 5);
        assert_eq!(
            lines,
            [(
                LogLevel::Warn,
                "OpenVault failed: no such folder".to_owned()
            )]
        );
        let written = [transfer(TransferMsg::Exported(Ok(
            "/v/exports/a.csv".into()
        )))];
        let lines = worker_events("Library", &written, 5);
        assert_eq!(
            lines[0],
            (
                LogLevel::Info,
                "export written to /v/exports/a.csv".to_owned()
            )
        );
        let slow = worker_events("Pdf.Render", &[], SLOW_MS + 1);
        assert_eq!(slow[0].0, LogLevel::Warn);
        assert!(slow[0].1.contains("took"));
        assert!(worker_events("LoadLibrary", &[Msg::Tick], 3).is_empty());
    }

    #[test]
    fn a_refused_import_is_one_warning_with_the_reason() {
        let refused = transfer(TransferMsg::Imported(Err("bad file".into())));
        let lines = worker_events("Library", &[refused], 1);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].1.contains("bad file"));
    }
}
