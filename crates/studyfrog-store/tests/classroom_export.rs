#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0288: a stored classroom session exports to a readable Markdown transcript.

mod common;

use common::classroom::sample;
use studyfrog_core::uid::Uid;
use studyfrog_store::classroom_store::{export_session_markdown, save_session};
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::schema::MIGRATIONS;

#[test]
fn export_renders_transcript_assignments_submissions_and_grades() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    save_session(&conn, &sample(0x51, "Cell lecture", 1_000)).unwrap();

    let md = export_session_markdown(&conn, Uid(0x51)).unwrap().unwrap();
    assert!(md.starts_with("# Cell lecture\n"));
    assert!(md.contains("Created 1970-01-01 00:00 UTC"));
    assert!(md.contains("- You: Cleo"));
    assert!(md.contains("**Ms Rowan** (1970-01-01 00:00 UTC): Welcome to the lecture."));
    assert!(md.contains("**Cleo** (1970-01-01 00:00 UTC): What is a cell?"));
    assert!(md.contains("### Warm-up\n"));
    assert!(md.contains("For: Cleo"));
    assert!(md.contains("Groups: Group 1 (Ada, Ben); Group 2 (Dev, Cleo)"));
    assert!(md.contains("### Warm-up: Cleo"));
    assert!(md.contains("Grade: 8/10. Good structure."));
    assert!(md.contains("### Project: Group 1"));
    assert!(md.contains("Not graded yet."));
}
