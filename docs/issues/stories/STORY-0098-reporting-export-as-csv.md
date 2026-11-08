---
id: STORY-0098
type: story
title: Reporting export as CSV
status: review
epic: EPIC-032
parents: [EPIC-032]
estimate: 3
labels: [m12, report, xfer]
acceptance_criteria:
  - "The Reports screen provides an Export as CSV action button"
  - "Exported CSV files contain structured tables covering review volumes, retention rates, mastery distributions, and rehearsal run logs"
  - "The export file is written to user-selected locations via native file dialog or saved directly to vault exports"
---

## Context
StudyFrog collects detailed learning telemetry: retention rates, review counts, SRS interval progressions, stack mastery distributions, and rehearsal run performance logs. Currently, these metrics are exclusively viewable within the Reports screen charts. Learners and educators frequently want to export their raw performance numbers into CSV files for longitudinal analysis, spreadsheet modeling, or archival.

## Description
Provide full reporting data export in CSV format:
1. Export Action & UX:
   - Add a prominent "Export CSV" button in the Reports screen toolbar (`crates/studyfrog-ui/src/view/reports.rs`).
   - Trigger a native file save dialog defaulting to `studyfrog-report-<vault>-<date>.csv`.
2. CSV Data Structure:
   - Export standard, well-structured CSV records including:
     - Summary Metrics: Total reviews, cards learned, cards mastered, current retention rate, active streak.
     - Per-Stack Statistics: Stack name, total cards, due count, learned count, mastered count, average interval (days), retention rate.
     - Rehearsal History Log: Timestamp, stack, run duration (seconds), cards reviewed, ratings breakdown (Again, Hard, Good, Easy), pass rate.
   - Format cleanly adhering to RFC 4180 with standard headers and comma delimiters.
3. Feedback & Error Handling:
   - Display a status notice upon successful export with the saved file path.
   - Handle filesystem permissions and write failures gracefully.

## Acceptance Criteria
- [x] The Reports screen provides an Export as CSV action button
- [ ] Exported CSV files contain structured tables covering review volumes, retention rates, mastery distributions, and rehearsal run logs
- [x] The export file is written to user-selected locations via native file dialog or saved directly to vault exports

## Design Notes
- Serialization: Implement report CSV generation in `crates/studyfrog-service/src/report.rs` or `crates/studyfrog-service/src/xfer_csv.rs`.
- Crate touchpoints:
  - `crates/studyfrog-service/src/report.rs`: Add `export_report_csv(summary: &ReportSummary) -> String`.
  - `crates/studyfrog-ui/src/view/reports.rs`: Add Export CSV button and file save effect.
  - `crates/studyfrog-ui/src/exec_transfer.rs`: Handle file writing for report exports.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Built and tested: `studyfrog-service::report_csv` renders one RFC 4180 table (`section,subject,metric,value`) with the review total, retention per gap bucket, answer-time percentiles, the review forecast, the familiarity distribution and the run log (state, rated, skipped, active seconds, answers per minute); the Reports screen has "Export as CSV", which writes `exports/studyfrog-report-<vault>-<date>.csv` in the vault and announces the path; a failed write is reported.

Open: criterion 2 is partly met. The export has no per-stack statistics, no learned or mastered totals and no streak, because the report bundle does not carry them; "mastery distribution" is the familiarity distribution. The file goes to the vault's `exports` folder, there is no save dialog. Not looked at in a real window.

## Dependencies
EPIC-032.

## Open Questions
None.
