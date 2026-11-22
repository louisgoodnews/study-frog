---
id: STORY-0152
type: story
title: "Dedicated internal stdout and stderr execution logging for user scripts"
status: review
epic: EPIC-049
parents: [EPIC-049]
estimate: 3
labels: [m12, ext, console]
acceptance_criteria:
  - "The application captures stdout and stderr from user scripts into dedicated, distinct internal streams"
  - "The script execution UI displays stdout and stderr with distinct visual styling (e.g. standard text vs. error highlights) and timestamps"
  - "Internal execution logs can be filtered, cleared, and exported for troubleshooting user-authored scripts"
---

## Context
When executing automated scripts, differentiating between standard programmatic output (query responses, progress notifications) and diagnostic error output (syntax errors, uncaught exceptions, stack traces) is essential for rapid debugging. Currently, console log entries interleave or aggregate execution text into generic notification notices. StudyFrog requires dedicated stdout and stderr logging channels with visual separation and structured diagnostics.

## Description
Implement dedicated stdout and stderr capture and display:
1. Capture child process or DSL runtime `stdout` and `stderr` streams independently into structured ring buffers.
2. In the console UI, render stdout in clear monospace body text and stderr in highlighted error/warning palette tones.
3. Provide stream filter toggles (`All`, `stdout only`, `stderr only`) and search filtering.
4. Support clearing active streams and exporting execution logs to `<vault>/logs/script_runs.log` for inspection.

## Acceptance Criteria
- [ ] The application captures stdout and stderr from user scripts into dedicated, distinct internal streams
- [ ] The script execution UI displays stdout and stderr with distinct visual styling (e.g. standard text vs. error highlights) and timestamps
- [ ] Internal execution logs can be filtered, cleared, and exported for troubleshooting user-authored scripts

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `crates/studyfrog-app/src/script_runner.rs`, `crates/studyfrog-service/src/scripting/`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-049 (Expanded scripting environment, DSL, job scheduling, and execution logging).
- STORY-0149 (Rename ScriptRunner to avoid trademark conflicts).

## Window check (2026-11-22)

Checked in the running window: entries show their time, the stream filters and the output search work. Fixed in this check: at 960 by 640 the search field was covered by "Export log" (the buttons now flow in a row), and the success marker drew as an empty box (now "+"). Not checked: "Export log" writing the file.

## Verification (2026-11-22)
Checked in the running app (`/tmp/claude-0/verifyB`). Script `/tmp/claude-0/verifyB/home/scripts/err.sh` prints one line to stdout and one to stderr. Run from Console > Jobs > "Run a script by hand" (`/tmp/claude-0/verifyB/k03-jobs.png`):
- Criterion 1 (distinct internal streams): NOT MET. The stderr line is in the stdout stream (`/tmp/claude-0/verifyB/k07-filter-stdout.png`). The Stderr / Errors filter says "No entries for selected filter" (`/tmp/claude-0/verifyB/k06-filter-stderr.png`). BUG-0033.
- Criterion 2 (distinct styling, timestamps): NOT MET. The stderr line is drawn in the same green (115,203,135) as stdout (`/tmp/claude-0/verifyB/k05-console-all.png`, pixel check). Timestamps are present (elapsed seconds). BUG-0033.
- Criterion 3 (filter, clear, export): NOT MET. Filters and output search work for stdout. The Stderr filter is empty for script stderr. Export log wrote `/tmp/claude-0/verifyB/vault/logs/script_runs.log` with both lines unlabelled. BUG-0033.
Nothing ticked, status stays review.

## Open Questions
- Maximum buffer size for script output streams before rolling truncation (default: 10,000 lines or 2 MB).

## Implementation (2026-11-22, audit)

Audit (2026-11-22): criterion 1 NOT MET: script stderr is shown in the stdout stream; the Stderr / Errors filter shows "No entries for selected filter" for a job that writes to stderr (/tmp/claude-0/audit/g5/0151-filter-stderr.png; the line appears under Stdout in /tmp/claude-0/audit/g5/0151-filter-stdout.png). BUG-0033.
Criterion 2 NOT MET; unticked by this audit, which contradicts the earlier window check: the stderr line is drawn in the same green as the stdout line (/tmp/claude-0/audit/g5/0151-console-run.png; the pixel colours of both lines match). Timestamps are present (elapsed seconds). FrogScript errors are red (/tmp/claude-0/audit/g5/0150-studio-error.png), so only script stderr is affected. BUG-0033.
Criterion 3 NOT MET: the All, Stdout and Events filters, the output search and Clear Output work (/tmp/claude-0/audit/g5/0152-cleared.png). Export log wrote /tmp/claude-0/audit/g5/vault/logs/script_runs.log, and the status bar confirmed the path. But the Stderr filter is empty for script stderr and the export does not label streams (BUG-0033).

Review (2026-11-22): criterion 1 NOT MET: the stderr line of `err.sh` appears in the Stdout view, and the Stderr / Errors view says "No entries for selected filter" (`/tmp/claude-0/review-c/shots/10-filter-stdout.png`, `/tmp/claude-0/review-c/shots/11-filter-stderr.png`). Criterion 2 NOT MET: both lines are the same green (`/tmp/claude-0/review-c/shots/09-run-all.png`). Criterion 3 NOT MET: the Stderr filter is empty for script stderr; All, Stdout and output search work (`09-run-all.png`, `10-filter-stdout.png`). Export not re-run. BUG-0033 still open. Status stays review.
