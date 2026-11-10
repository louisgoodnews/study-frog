---
id: STORY-0152
type: story
title: Dedicated internal stdout and stderr execution logging for user scripts
status: backlog
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

## Open Questions
- Maximum buffer size for script output streams before rolling truncation (default: 10,000 lines or 2 MB).
