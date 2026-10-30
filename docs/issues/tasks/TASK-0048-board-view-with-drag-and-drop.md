---
id: TASK-0048
type: task
title: Board view with drag and drop
status: backlog
epic: EPIC-011
parents: [STORY-0022]
estimate: 5
labels: [m5, ui, board]
acceptance_criteria:
  - "Columns side by side with WIP counts ('2/3') and an over-limit highlight"
  - "Drag a card to another column or position; buttons push, pull, move up and down"
  - "Add, edit title/notes and delete cards; link a card to a stack"
  - "Accessibility audit passes; moves go through the pure board functions"
---

## Context
egui drag and drop (`dnd_drag_source`/`dnd_drop_zone`).

## Description
Board view with drag and drop.

## Acceptance Criteria
- [ ] Columns side by side with WIP counts ('2/3') and an over-limit highlight
- [ ] Drag a card to another column or position; buttons push, pull, move up and down
- [ ] Add, edit title/notes and delete cards; link a card to a stack
- [ ] Accessibility audit passes; moves go through the pure board functions

## Design Notes
Drag and drop cannot be driven headlessly; the same moves are covered through messages and buttons.

## Dependencies
TASK-0047, TASK-0080.

## Open Questions
None.
