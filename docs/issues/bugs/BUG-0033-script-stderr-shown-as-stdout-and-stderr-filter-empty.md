---
id: BUG-0033
type: bug
title: "Script stderr is shown in the stdout stream and the Stderr / Errors filter stays empty"
status: backlog
estimate: 3
labels: [m12, console, ext]
acceptance_criteria:
  - "Text a script writes to standard error appears under the Stderr / Errors filter and not under Stdout"
  - "Script stderr lines are drawn in the error palette, not in the same green as stdout"
  - "Export log labels each script line as stdout or stderr"
---

## Context
Found while auditing STORY-0151 and STORY-0152 in the running app (2026-11-22). A job `hello.py` that prints one line to stdout and one to stderr, run by hand from the job editor:
- the Console's "All" and "Stdout" views show both lines, both in the same green as stdout;
- the "Stderr / Errors" view says "No entries for selected filter";
- the exported `<vault>/logs/script_runs.log` writes the stderr line as a plain line, with no stream label.
The app log does record "script ... wrote to standard error", so the text is captured but routed to the wrong stream.

Screenshots (`/tmp/claude-0/audit/g5/`): `0151-console-run.png` (both lines green), `0151-filter-stderr.png` (empty filter), `0151-filter-stdout.png` (stderr line under Stdout), `0152-export.png` and `/tmp/claude-0/audit/g5/vault/logs/script_runs.log`.

## Description
Route a script's standard error into the stderr stream of the console (the stream the Stderr / Errors filter reads) and draw it in the error or warning palette. Keep the stream label when the log is exported. Check in the window with a script that writes to both streams.

## Acceptance Criteria
- [ ] Text a script writes to standard error appears under the Stderr / Errors filter and not under Stdout
- [ ] Script stderr lines are drawn in the error palette, not in the same green as stdout
- [ ] Export log labels each script line as stdout or stderr

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `crates/studyfrog-ui/src/console_ui.rs`, `crates/studyfrog-ui/src/log_stream.rs`, `crates/studyfrog-service/src/scripting/exec.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- STORY-0152
- STORY-0151

## Verification (2026-11-22)
Reproduced in the running app (`/tmp/claude-0/verifyB`). Script `/tmp/claude-0/verifyB/home/scripts/err.sh` prints `VERIFY-STDOUT-LINE one` to stdout and `VERIFY-STDERR-LINE two` to stderr; run from Console > Jobs > "Run a script by hand":
- All: both lines shown, both in the same green (115,203,135), checked by pixel histogram (`/tmp/claude-0/verifyB/k05-console-all.png`). Criterion 2 not met.
- Stdout: the stderr line is under Stdout (`/tmp/claude-0/verifyB/k07-filter-stdout.png`). Criterion 1 not met.
- Stderr / Errors: "No entries for selected filter" (`/tmp/claude-0/verifyB/k06-filter-stderr.png`).
- Export log wrote `/tmp/claude-0/verifyB/vault/logs/script_runs.log` with both lines as plain text and no stream label (criterion 3 not met).
Status stays backlog.

Review (2026-11-22): NOT FIXED. The stderr line is under Stdout (`/tmp/claude-0/review-c/shots/10-filter-stdout.png`), the Stderr / Errors filter is empty (`/tmp/claude-0/review-c/shots/11-filter-stderr.png`), and both lines are the same green (`/tmp/claude-0/review-c/shots/09-run-all.png`). Stays backlog.

## Open Questions
- None.
