---
id: STORY-0131
type: story
title: "Python client package for the control protocol"
status: done
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
- [x] A pure-Python package `studyfrog` (standard library only, type hints, README) connects to a running instance's socket or starts a headless one and offers typed functions for state, navigation, items, stacks, query, search, rehearsal, import and export, logs and event subscription
- [x] A Python unittest suite, run from a Rust integration test against the real `studyfrog` binary in a temporary home, passes (and the Rust test says clearly when python3 is missing)

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`). The binary embeds the package source and writes it to `<app home>/scripts/lib/` (STORY-0132).

## Progress (2026-11-08)
Built: `python/studyfrog/` (`transport.py`: Unix socket or the pipes of a started `--control-stdio`, line reads with a timeout through `select`; `client.py`: `Client`, `connect`, typed wrappers; `types.py`: `Item`, `Event`, `LogEntry`, `StudyFrogError`; `script.py`: what a runner-started script is told), `python/README.md`. Verified by `python/tests/` (10 unittest cases: state, create/query/edit/move, refusals as exceptions, a rehearsal to finished, export and logs over pipes; two clients on one socket with an `item_created` event; standard-library-only imports) run by `crates/studyfrog-app/tests/python_client.rs` against the real binary, which fails if any case is skipped and prints SKIPPED when python3 is missing. Not covered: Windows (no Unix socket; the pipe transport uses `select` on a pipe, which Windows does not support); calendar, board, diary and settings actions, which the control protocol does not have.

## Dependencies
STORY-0130.

## Open Questions
None.
