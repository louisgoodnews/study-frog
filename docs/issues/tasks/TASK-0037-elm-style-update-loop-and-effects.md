---
id: TASK-0037
type: task
title: "Elm-style update loop and effects"
status: done
epic: EPIC-008
parents: [STORY-0016]
estimate: 5
labels: [m4, ui, service]
acceptance_criteria:
  - "`update(State, Msg) -> (State, Vec<Effect>)` is pure and unit-tested"
  - "Effects (disk, DB, clock, SLM) run at the edge and return as Msg"
  - "Poll-based vault reindex wired as an effect"
---

## Context
ADR 0002: our own discipline on top of egui.

## Description
Elm-style update loop and effects.

## Acceptance Criteria
- [x] `update(State, Msg) -> (State, Vec<Effect>)` is pure and unit-tested
- [x] Effects (disk, DB, clock, SLM) run at the edge and return as Msg
- [x] Poll-based vault reindex wired as an effect

## Design Notes
Pure core with no egui types: `State`, `Msg`, `Effect`, `update(State, Msg) -> (State, Vec<Effect>)`. Effects run on a worker thread (`runtime.rs`) through `exec.rs`, which only maps `studyfrog-service` results to messages; the vault session logic (open, index, poll) lives in `studyfrog-service::session`. A once-a-second `Tick` triggers at most one outstanding `PollVault`, which re-reads the index and `studyfrog.toml`. 9 update tests, 5 executor tests over a real vault, 7 headless view tests.

## Dependencies
TASK-0036.

## Open Questions
None.
