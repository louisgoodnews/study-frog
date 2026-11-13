---
id: STORY-0098
type: story
title: "Reporting export as CSV"
status: done
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
- [x] Exported CSV files contain structured tables covering review volumes, retention rates, mastery distributions, and rehearsal run logs
- [x] The export file is written to user-selected locations via native file dialog or saved directly to vault exports

## Design Notes
- Serialization: Implement report CSV generation in `crates/studyfrog-service/src/report.rs` or `crates/studyfrog-service/src/xfer_csv.rs`.
- Crate touchpoints:
  - `crates/studyfrog-service/src/report.rs`: Add `export_report_csv(summary: &ReportSummary) -> String`.
  - `crates/studyfrog-ui/src/view/reports.rs`: Add Export CSV button and file save effect.
  - `crates/studyfrog-ui/src/exec_transfer.rs`: Handle file writing for report exports.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Built and tested: `studyfrog-service::report_csv` renders one RFC 4180 table (`section,subject,metric,value`) with the review total, retention per gap bucket, answer-time percentiles, the review forecast, the familiarity distribution and the run log (state, rated, skipped, active seconds, answers per minute).

Added on 2026-11-08: the totals of learned items (reviewed at least once), mastered items (scheduled 21 days or more ahead) and the active streak of days with a review, and per stack (`stack` section, subject `Title (folder)`): items, due now, learned, mastered, reviews, average interval in days and retention (share not rated Again). Measured by `studyfrog-store::report_stacks` and `studyfrog-service::report_extras` like the achievements measure them (items of the stack's own folder). The Reports screen has "Export as CSV" (into `exports/` of the vault) and "Save CSV to..." (the existing folder dialog; cancelling writes nothing and says so). Help page updated.

Verified: `crates/studyfrog-service/tests/report_csv_stacks.rs` (a real vault with reviews recorded through the service: counts, due before and a year later, streak 3 then 0, average interval, retention, quoting of a stack title with a comma), `report_csv.rs`, `crates/studyfrog-ui/tests/report_csv_ui.rs` (both buttons, the file in `exports/` and in the chosen folder with stack and total rows, the cancelled dialog), and a screenshot of the Reports screen with both buttons in the real binary. "Mastery distribution" is the familiarity distribution plus the mastered counts.

## Dependencies
EPIC-032.

## Open Questions
None.
