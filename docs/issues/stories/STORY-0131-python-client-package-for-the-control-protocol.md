---
id: STORY-0131
type: story
title: Python client package for the control protocol
status: in_progress
epic: EPIC-028
parents: [EPIC-028]
estimate: 5
labels: [m12, app, docs]
acceptance_criteria:
  - "A pure-Python package `studyfrog` (standard library only, type hints, README) connects to a running instance's socket or starts a headless one and offers typed functions for state, navigation, items, stacks, query, search, rehearsal, import and export, logs and event subscription"
  - "A Python unittest suite, run from a Rust integration test against the real `studyfrog` binary in a temporary home, passes (and the Rust test says clearly when python3 is missing)"
---
## Context
EPIC-028 asks for a Python API that runs right away without recompiling (ADR 0013).

## Description
`python/studyfrog/` with a connection layer (Unix socket, or `--control-stdio` pipes) and an API layer; dataclasses for items, events and log entries. Tasks: TASK-0245, TASK-0246.

## Acceptance Criteria
- [ ] A pure-Python package `studyfrog` (standard library only, type hints, README) connects to a running instance's socket or starts a headless one and offers typed functions for state, navigation, items, stacks, query, search, rehearsal, import and export, logs and event subscription
- [ ] A Python unittest suite, run from a Rust integration test against the real `studyfrog` binary in a temporary home, passes (and the Rust test says clearly when python3 is missing)

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`). The binary embeds the package source and writes it to `<app home>/scripts/lib/` (STORY-0132).

## Dependencies
STORY-0130.

## Open Questions
None.
