#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Console's output controls (STORY-0152): the search rule, the time on each entry, and the export of the
//! transcript to the vault's log file.

mod common;

use common::{NOW_NS, sample_vault};
use studyfrog_ui::console_ui::lines::{export_text, matches};
use studyfrog_ui::console_ui::{ConsoleMsg, ConsoleState, Entry, EntryKind, apply_console};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_with_host};
use studyfrog_ui::host::Host;
use studyfrog_ui::msg::Msg;

fn entry(text: &str, at: f64) -> Entry {
    Entry {
        at,
        kind: EntryKind::Stdout,
        text: text.into(),
    }
}

#[test]
fn an_empty_search_shows_everything_and_a_text_is_matched_without_case() {
    let e = entry("Checked 3 Cards", 1.0);
    assert!(matches(&e, ""));
    assert!(matches(&e, "  "));
    assert!(matches(&e, "cards"));
    assert!(!matches(&e, "decks"));
}

#[test]
fn each_line_of_the_transcript_carries_its_time() {
    let mut c = ConsoleState::default();
    apply_console(&mut c, ConsoleMsg::Line("started health".into()), 12.5);
    apply_console(&mut c, ConsoleMsg::Search("health".into()), 0.0);
    assert_eq!(c.search, "health");
    assert_eq!(c.entries[0].at, 12.5);
    let text = export_text(&c);
    assert!(text.contains("12.5s"), "{text}");
    assert!(text.contains("started health"), "{text}");
}

#[test]
fn the_export_writes_the_transcript_to_the_vault_log() {
    let vault = sample_vault("console-export");
    let mut w = worker_with_host(None, None, Host::default());
    execute(
        &mut w,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    let out = execute(
        &mut w,
        Effect::Console(studyfrog_ui::console_ui::ConsoleEffect::Export(
            "[     1.0s] Stdout: hello\n".into(),
        )),
        NOW_NS,
    );
    let path = match &out[..] {
        [Msg::Console(ConsoleMsg::Exported(Ok(path)))] => path.clone(),
        other => panic!("not exported: {other:?}"),
    };
    assert!(path.ends_with("logs/script_runs.log"), "{path}");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "[     1.0s] Stdout: hello\n"
    );
}

#[test]
fn an_export_without_a_vault_says_so() {
    let mut w = worker_with_host(None, None, Host::default());
    let out = execute(
        &mut w,
        Effect::Console(studyfrog_ui::console_ui::ConsoleEffect::Export("x".into())),
        NOW_NS,
    );
    assert!(
        matches!(&out[..], [Msg::Console(ConsoleMsg::Exported(Err(_)))]),
        "{out:?}"
    );
}
