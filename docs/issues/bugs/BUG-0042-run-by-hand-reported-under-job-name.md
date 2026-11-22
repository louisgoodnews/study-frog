---
id: BUG-0042
type: bug
title: "A script run by hand from the job editor is reported under the name of the job that uses that file"
status: backlog
estimate: 1
labels: [m12, console]
acceptance_criteria:
  - "Running a file by hand reports the file's name, e.g. 'hello.py', in the started and finished lines"
  - "A job's own runs still report the job's name"
---

## Context
Found while auditing STORY-0151 in the running app (2026-11-22). The job editor's "Run a script by hand" with `hello.py` and the arguments "xray yankee" logged "› script run hello.py xray yankee", then "started daily-hello; its output follows when it ends" and "+ script daily-hello finished". The job `daily-hello` uses `hello.py`, so the run was reported under the job's name, although the job's own schedule did not start it. The output and arguments are right.

Screenshot: `/tmp/claude-0/audit/g5/0151-console-run.png`.

## Description
Report a by-hand run under the file's name (or the job's name only when the run was started by that job). Check the Console and the exported log.

## Acceptance Criteria
- [ ] Running a file by hand reports the file's name, e.g. 'hello.py', in the started and finished lines
- [ ] A job's own runs still report the job's name

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/console_ui.rs`, `crates/studyfrog-service/src/scripting/script_launch.rs`.
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0151

## Open Questions
- None.
