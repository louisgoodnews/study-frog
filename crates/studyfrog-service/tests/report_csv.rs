#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The reports as CSV: the table, its quoting and the file name.

use studyfrog_core::pacing::Pacing;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_report::latency::LatencyStats;
use studyfrog_report::retention::RetentionBucket;
use studyfrog_service::report_csv::{ReportCsvData, report_csv, report_csv_name};
use studyfrog_service::run_history::RunSummary;

fn run(state: &str, used_ms: u32) -> RunSummary {
    RunSummary {
        uid: Uid(1),
        mode: RunMode::Flip,
        pacing: Pacing::Free,
        state: state.into(),
        started_ms: 1_800_000_000_000,
        rated: 30,
        skipped: 2,
        used_ms,
        exam: None,
    }
}

fn lines(csv: &str) -> Vec<&str> {
    csv.split("\r\n").filter(|l| !l.is_empty()).collect()
}

#[test]
fn the_table_has_a_header_and_one_row_per_number_for_every_section() {
    let buckets = [
        RetentionBucket {
            from_days: 1,
            to_days: 1,
            reviews: 10,
            recalled: 9,
        },
        RetentionBucket {
            from_days: 8,
            to_days: u32::MAX,
            reviews: 0,
            recalled: 0,
        },
    ];
    let stats = LatencyStats {
        n: 5,
        p50: 1200,
        p90: 3000,
        p95: 4000,
        max: 9000,
    };
    let runs = [run("finished", 600_000)];
    let csv = report_csv(&ReportCsvData {
        reviews: 42,
        retention: &buckets,
        latency: Some(&stats),
        forecast: &[3.0, 7.46],
        spread: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        unscored: 11,
        runs: &runs,
        extras: None,
    });
    let rows = lines(&csv);
    assert_eq!(rows[0], "section,subject,metric,value");
    assert!(csv.ends_with("\r\n"));
    for want in [
        "summary,all,reviews,42",
        "retention,1 days,recalled,9",
        "retention,1 days,retention_rate,0.9000",
        "retention,8+ days,reviews,0",
        "latency,all items,p95_ms,4000",
        "forecast,day 1,expected_reviews,7.5",
        "familiarity,10-19 %,items,1",
        "familiarity,no history,items,11",
        "runs,2027-01-15T08:00:00.000Z Flip Free,rated,30",
        "runs,2027-01-15T08:00:00.000Z Flip Free,active_seconds,600",
        "runs,2027-01-15T08:00:00.000Z Flip Free,answers_per_minute,3.0",
    ] {
        assert!(rows.contains(&want), "missing {want:?} in {rows:#?}");
    }
    assert!(
        !rows
            .iter()
            .any(|r| r.starts_with("retention,8+ days,retention_rate")),
        "an empty bucket has no rate"
    );
    assert!(
        rows.iter().all(|r| r.matches(',').count() >= 3),
        "four cells in every row"
    );
}

#[test]
fn cells_with_commas_quotes_or_line_breaks_are_quoted_and_a_run_without_time_has_no_speed() {
    let runs = [run("ended, \"early\"", 0)];
    let csv = report_csv(&ReportCsvData {
        reviews: 0,
        retention: &[],
        latency: None,
        forecast: &[],
        spread: [0; 10],
        unscored: 0,
        runs: &runs,
        extras: None,
    });
    let rows = lines(&csv);
    assert!(
        rows.contains(&r#"runs,2027-01-15T08:00:00.000Z Flip Free,state,"ended, ""early""""#),
        "{rows:#?}"
    );
    assert!(!csv.contains("answers_per_minute"));
    assert!(!csv.contains("latency"));
}

#[test]
fn the_file_name_has_the_vault_and_the_day_and_nothing_a_file_system_dislikes() {
    assert_eq!(
        report_csv_name("My notes/2026", "2026-11-08"),
        "studyfrog-report-My-notes-2026-2026-11-08.csv"
    );
}
