---
id: TASK-0083
type: task
title: Post-run review and applying a suggestion
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 5
labels: [m6, ui, service]
acceptance_criteria:
  - "The review lists previous rating, own mark, own rating and suggested rating with the reason and the reading time"
  - "Applying replaces the rating of the item's latest review and rebuilds its schedule, giving the state the other rating would have produced"
  - "Practice-only runs show suggestions but offer no change; a changed history is refused"
---

## Context
Suggestions are only useful if the user can act on them.

## Description
Post-run review and applying a suggestion.

## Acceptance Criteria
- [x] The review lists previous rating, own mark, own rating and suggested rating with the reason and the reading time
- [x] Applying replaces the rating of the item's latest review and rebuilds its schedule, giving the state the other rating would have produced
- [x] Practice-only runs show suggestions but offer no change; a changed history is refused

## Design Notes
`service::rerate` (replay of the review history; FSRS and SM-2), `ui::review`, `view/summary`. The run log keeps what the user pressed; only the review and the schedule change. The change is one click per item, never automatic.

## Dependencies
TASK-0081, TASK-0082.

## Open Questions
None.
