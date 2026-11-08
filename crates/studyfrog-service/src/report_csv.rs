//! The reports as one CSV table (RFC 4180): `section,subject,metric,value`, one row per number, so a
//! spreadsheet can pivot it. Pure.

use studyfrog_core::time::UnixMs;
use studyfrog_md::rfc3339::format_rfc3339;
use studyfrog_report::latency::LatencyStats;
use studyfrog_report::retention::{RetentionBucket, retention_rate};

use crate::report_extras::ReportExtras;
use crate::reports::ReportBundle;
use crate::run_history::{RunSummary, answers_per_minute};

/// What is exported, borrowed from the reports.
#[derive(Debug, Clone, Copy)]
pub struct ReportCsvData<'a> {
    /// Reviews in the history.
    pub reviews: usize,
    /// Retention per gap bucket.
    pub retention: &'a [RetentionBucket],
    /// Time to reveal an answer, all items.
    pub latency: Option<&'a LatencyStats>,
    /// Expected reviews on each of the next days, starting today.
    pub forecast: &'a [f32],
    /// Items per tenth of the familiarity scale.
    pub spread: [u32; 10],
    /// Items with no history yet.
    pub unscored: usize,
    /// The latest runs.
    pub runs: &'a [RunSummary],
    /// Every stack's numbers and the vault's totals, when they were measured.
    pub extras: Option<&'a ReportExtras>,
}

/// The part of the reports that is exported.
pub fn data_of(b: &ReportBundle) -> ReportCsvData<'_> {
    ReportCsvData {
        reviews: b.reviews,
        retention: &b.retention,
        latency: b.latency.as_ref(),
        forecast: &b.forecast,
        spread: b.familiarity.spread,
        unscored: b.familiarity.unscored,
        runs: &b.runs,
        extras: None,
    }
}

/// A cell, in double quotes when it holds a comma, a quote or a line break.
fn cell(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_owned()
    }
}

fn row(section: &str, subject: &str, metric: &str, value: impl ToString) -> String {
    let fields = [section, subject, metric, &value.to_string()];
    fields.map(cell).join(",")
}

fn gap_label(b: &RetentionBucket) -> String {
    match (b.from_days, b.to_days) {
        (f, t) if f == t => format!("{f} days"),
        (f, u32::MAX) => format!("{f}+ days"),
        (f, t) => format!("{f}-{t} days"),
    }
}

fn retention_rows(buckets: &[RetentionBucket]) -> Vec<String> {
    let mut rows = Vec::new();
    for b in buckets {
        let subject = gap_label(b);
        rows.push(row("retention", &subject, "reviews", b.reviews));
        rows.push(row("retention", &subject, "recalled", b.recalled));
        if let Some(rate) = retention_rate(b) {
            rows.push(row(
                "retention",
                &subject,
                "retention_rate",
                format!("{rate:.4}"),
            ));
        }
    }
    rows
}

fn latency_rows(stats: Option<&LatencyStats>) -> Vec<String> {
    let Some(s) = stats else { return Vec::new() };
    let named = [
        ("samples", s.n as u32),
        ("p50_ms", s.p50),
        ("p90_ms", s.p90),
    ];
    let tail = [("p95_ms", s.p95), ("max_ms", s.max)];
    named
        .iter()
        .chain(&tail)
        .map(|(metric, v)| row("latency", "all items", metric, v))
        .collect()
}

fn run_rows(runs: &[RunSummary]) -> Vec<String> {
    let mut rows = Vec::new();
    for r in runs {
        let when = format_rfc3339(UnixMs(r.started_ms)).unwrap_or_default();
        let subject = format!("{when} {:?} {:?}", r.mode, r.pacing);
        rows.push(row("runs", &subject, "state", &r.state));
        rows.push(row("runs", &subject, "rated", r.rated));
        rows.push(row("runs", &subject, "skipped", r.skipped));
        rows.push(row("runs", &subject, "active_seconds", r.used_ms / 1000));
        if let Some(per_minute) = answers_per_minute(r.rated, r.used_ms) {
            rows.push(row(
                "runs",
                &subject,
                "answers_per_minute",
                format!("{per_minute:.1}"),
            ));
        }
    }
    rows
}

fn extra_rows(x: &ReportExtras) -> Vec<String> {
    let t = &x.totals;
    let mut rows = vec![
        row("summary", "all", "learned", t.learned),
        row("summary", "all", "mastered", t.mastered),
        row("summary", "all", "streak_days", t.streak_days),
    ];
    for s in &x.stacks {
        let subject = format!("{} ({})", s.title, s.folder);
        let counts = [
            ("items", s.items),
            ("due", s.due),
            ("learned", s.learned),
            ("mastered", s.mastered),
            ("reviews", s.reviews),
        ];
        rows.extend(counts.map(|(metric, v)| row("stack", &subject, metric, v)));
        if let Some(days) = s.average_interval_days {
            rows.push(row(
                "stack",
                &subject,
                "average_interval_days",
                format!("{days:.1}"),
            ));
        }
        if s.reviews > 0 {
            let rate = f64::from(s.recalled) / f64::from(s.reviews);
            rows.push(row(
                "stack",
                &subject,
                "retention_rate",
                format!("{rate:.4}"),
            ));
        }
    }
    rows
}

/// The table, header first, with `\r\n` between rows as RFC 4180 asks.
pub fn report_csv(d: &ReportCsvData<'_>) -> String {
    let mut rows = vec![
        "section,subject,metric,value".to_owned(),
        row("summary", "all", "reviews", d.reviews),
    ];
    rows.extend(d.extras.map(extra_rows).unwrap_or_default());
    rows.extend(retention_rows(d.retention));
    rows.extend(latency_rows(d.latency));
    for (day, expected) in d.forecast.iter().enumerate() {
        let subject = format!("day {day}");
        rows.push(row(
            "forecast",
            &subject,
            "expected_reviews",
            format!("{expected:.1}"),
        ));
    }
    for (tenth, items) in d.spread.iter().enumerate() {
        let subject = format!("{}-{} %", tenth * 10, tenth * 10 + 9);
        rows.push(row("familiarity", &subject, "items", items));
    }
    rows.push(row("familiarity", "no history", "items", d.unscored));
    rows.extend(run_rows(d.runs));
    rows.join("\r\n") + "\r\n"
}

/// The file name of an export: `studyfrog-report-<vault>-<date>.csv`, with the vault name cleaned.
pub fn report_csv_name(vault: &str, day: &str) -> String {
    let clean: String = vault
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!("studyfrog-report-{clean}-{day}.csv")
}
