#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The CSV export's per-stack statistics and totals (STORY-0098), measured on a real vault with
//! reviews recorded through the service.

use std::fs;
use std::path::PathBuf;

use studyfrog_agent::exam::{ExamResult, Verdict};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::agent_vault::record_exam;
use studyfrog_service::report_csv::{data_of, report_csv};
use studyfrog_service::report_extras::report_extras;
use studyfrog_service::reports::build_reports;
use studyfrog_service::session::{VaultSession, open_vault};

const DAY: i64 = 86_400_000;
const T0: i64 = 1_800_000_000_000;

fn write(dir: &std::path::Path, rel: &str, n: u128, kind: &str, title: &str, body: &str) {
    let text = format!(
        "---\nid: {}\nkind: {kind}\ntitle: {title}\n---\n{body}",
        format_ulid(Uid(n))
    );
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn vault(label: &str) -> (PathBuf, VaultSession) {
    let dir = std::env::temp_dir().join(format!("sf-report-stacks-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    write(&dir, "bio/_stack.md", 1, "stack", "Biology, basics", "");
    write(&dir, "bio/c1.md", 2, "card", "Cell", "front?\n???\nback\n");
    write(&dir, "bio/c2.md", 3, "card", "Gene", "front?\n???\nback\n");
    write(&dir, "bio/n.md", 4, "note", "Notes", "text\n");
    write(&dir, "art/_stack.md", 5, "stack", "Art", "");
    write(&dir, "art/c.md", 6, "card", "Hue", "front?\n???\nback\n");
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (dir, s)
}

fn answer(s: &mut VaultSession, item: u128, correct: bool, at: i64) {
    let verdict = if correct {
        Verdict::Correct
    } else {
        Verdict::Incorrect
    };
    let r = ExamResult {
        uid: format_ulid(Uid(item)),
        verdict,
    };
    record_exam(s, &[r], UnixMs(at)).unwrap();
}

#[test]
fn every_stack_gets_its_items_due_learned_mastered_interval_and_retention() {
    let (dir, mut s) = vault("stacks");
    // Card 2 on three days in a row (one miss), card 3 once; nothing in Art.
    answer(&mut s, 2, true, T0);
    answer(&mut s, 2, false, T0 + DAY);
    answer(&mut s, 2, true, T0 + 2 * DAY);
    answer(&mut s, 3, true, T0 + 2 * DAY);
    let now = UnixMs(T0 + 2 * DAY + 3_600_000);
    let x = report_extras(&s, now, 0).unwrap();
    assert_eq!(x.totals.learned, 2);
    assert_eq!(x.totals.mastered, 0, "nothing is scheduled weeks ahead yet");
    assert_eq!(x.totals.streak_days, 3);
    let art = &x.stacks[0];
    assert_eq!((art.folder.as_str(), art.title.as_str()), ("art", "Art"));
    assert_eq!((art.items, art.learned, art.reviews), (1, 0, 0));
    assert_eq!(art.due, 0, "a card never reviewed has no due date yet");
    assert_eq!(art.average_interval_days, None);
    let bio = &x.stacks[1];
    assert_eq!(
        (bio.items, bio.learned, bio.reviews, bio.recalled),
        (3, 2, 4, 3)
    );
    assert_eq!(bio.due, 0, "both cards were just reviewed");
    assert!(bio.average_interval_days.is_some_and(|d| d > 0.0));
    let later = report_extras(&s, UnixMs(T0 + 400 * DAY), 0).unwrap();
    assert_eq!(later.stacks[1].due, 2, "a year later both are due");
    assert_eq!(later.totals.streak_days, 0, "the streak has ended");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_csv_carries_the_totals_and_one_block_per_stack() {
    let (dir, mut s) = vault("csv");
    answer(&mut s, 2, true, T0);
    answer(&mut s, 2, false, T0 + DAY);
    let now = UnixMs(T0 + DAY + 60_000);
    let bundle = build_reports(&s, now, 0).unwrap();
    let extras = report_extras(&s, now, 0).unwrap();
    let csv = report_csv(&studyfrog_service::report_csv::ReportCsvData {
        extras: Some(&extras),
        ..data_of(&bundle)
    });
    let rows: Vec<&str> = csv.split("\r\n").collect();
    for want in [
        "summary,all,learned,1",
        "summary,all,mastered,0",
        "summary,all,streak_days,2",
        "stack,Art (art),items,1",
        "stack,Art (art),learned,0",
        "stack,\"Biology, basics (bio)\",items,3",
        "stack,\"Biology, basics (bio)\",reviews,2",
        "stack,\"Biology, basics (bio)\",retention_rate,0.5000",
    ] {
        assert!(rows.contains(&want), "missing {want:?} in {rows:#?}");
    }
    assert!(
        rows.iter()
            .any(|r| r.starts_with("stack,\"Biology, basics (bio)\",average_interval_days,"))
    );
    let _ = fs::remove_dir_all(&dir);
}
