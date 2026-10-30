---
id: EPIC-011
type: epic
title: Scrum/kanban board
status: backlog
parents: []
labels: [m5, board]
acceptance_criteria:
  - "Columns with WIP limits; WIP is enforced (property-tested)"
  - "Drag and push semantics work in the UI"
  - "The board persists across restarts"
---

## Context
Task tracking inside the learning app.

## Description
Pure board model in `studyfrog-board`, persisted in SQLite (ADR 0003).

## Acceptance Criteria
- [ ] Columns with WIP limits; WIP is enforced (property-tested)
- [ ] Drag and push semantics work in the UI
- [ ] The board persists across restarts

## Design Notes
Children: STORY-0021, STORY-0022.

## Dependencies
M4 exit.

## Open Questions
Scrum sprints are out of scope for M5 (kanban with story points only); revisit if wanted.
