---
id: TASK-0081
type: task
title: "Difficulty marks during a run"
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, rehearsal, ui]
acceptance_criteria:
  - "`Action::Mark(Easy|Medium|Difficult)` is valid while an item is on screen, before or after the reveal"
  - "The mark is an `ItemMarked` event, stored and loaded with the run; the last mark counts"
  - "Buttons and the keys E, M and D send it; the chosen mark is highlighted"
---

## Context
Marks are separate from the Again/Hard/Good/Easy rating: they are how the item felt.

## Description
Difficulty marks during a run.

## Acceptance Criteria
- [x] `Action::Mark(Easy|Medium|Difficult)` is valid while an item is on screen, before or after the reveal
- [x] The mark is an `ItemMarked` event, stored and loaded with the run; the last mark counts
- [x] Buttons and the keys E, M and D send it; the chosen mark is highlighted

## Design Notes
They do not change the schedule or the rating. Aggregating marks into a per-item difficulty is left for reporting (M7).

## Dependencies
None.

## Open Questions
None.
