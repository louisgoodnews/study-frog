---
id: EPIC-071
type: epic
title: "In-house logging and debugging: studyfrog-log and studyfrog-debug, written without external crates"
status: in_progress
parents: []
labels: [m14, tooling]
acceptance_criteria:
  - "studyfrog-log provides levels, a minimum level and sink set at start-up, a ring of recent records, a rotating file sink and the log_*! macros, with no external crate"
  - "studyfrog-debug provides timers, checkpoints, dumps, checks and counters on top of studyfrog-log, with no external crate"
  - "Both crates follow the project rules: no inherent impl blocks, files of at most 250 lines, and the dependency table in scripts/check-deps.sh"
---

## Context
StudyFrog logs through its own code today (STORY-0122), and it has no helpers for debugging a running session. The
project's rules (P1 no OOP, P10 build versus buy) favour small, owned code where the need is small. Logging and
debugging are such a need.

## Description
Two new crates. `studyfrog-log` is the logging API and has no studyfrog dependency. `studyfrog-debug` builds timers,
dumps, checks and counters on it. Application wiring is not part of this epic's first story.

## Acceptance Criteria
- [ ] `studyfrog-log` crate (STORY-0219)
- [ ] `studyfrog-debug` crate (STORY-0219)
- [ ] Both crates pass the project's structural gates (no inherent impl, line limits, dependency table)

## Design Notes
- The record format is one line: `<unix_ms> <LEVEL> <target> <message> key=value ...`, with quoting for values that need it.
- The level check comes before formatting in every macro and helper, so disabled logging is cheap.
- Each crate's tests run in the owner's gate; the first story was written without building.

## Dependencies
- STORY-0122 (the existing rotating log; the application may move onto the new crate in a later story).
- `scripts/check-deps.sh`, `scripts/check-impls.sh`, `scripts/linecount.sh`.

## Open Questions
- Whether the application's log moves onto `studyfrog-log`, and when. Owner decision.
