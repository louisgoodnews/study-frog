---
id: TASK-0048
type: task
title: "Board view with drag and drop"
status: done
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
- [x] Columns side by side with WIP counts ('2/3') and an over-limit highlight
- [x] Drag a card to another column or position; buttons push, pull, move up and down
- [x] Add, edit title/notes and delete cards; link a card to a stack
- [x] Accessibility audit passes; moves go through the pure board functions

## Design Notes
Drag and drop cannot be driven headlessly; the same moves are covered through messages and buttons.

## Dependencies
TASK-0047, TASK-0080.

## Open Questions
None.

## Outcome
Board screen (`kanban`, `kanban_msg`, `view/board`): columns side by side with `name n/limit` headers and an "Over the WIP limit" line (not colour alone); cards are drag sources and each column a drop zone (drop position from the pointer); push, pull, move up/down, edit and delete are buttons with the card title in their accessible name; a "Allow exceeding WIP limits" switch selects Force vs Strict; the card editor edits title, notes, estimate and linked stack. Every change is a `BoardOp` applied by the service, so the pure board rules decide; a refused move shows the readable reason and changes nothing. Dragging cannot be driven headlessly, so the same moves are tested through messages (`Move`) and buttons.
