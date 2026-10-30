---
id: TASK-0037
type: task
title: Elm-style update loop and effects
status: ready
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
- [ ] `update(State, Msg) -> (State, Vec<Effect>)` is pure and unit-tested
- [ ] Effects (disk, DB, clock, SLM) run at the edge and return as Msg
- [ ] Poll-based vault reindex wired as an effect

## Design Notes
Effects are plain data; the executor calls studyfrog-service.

## Dependencies
TASK-0036.

## Open Questions
None.
