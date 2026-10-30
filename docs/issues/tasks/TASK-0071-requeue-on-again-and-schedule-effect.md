---
id: TASK-0071
type: task
title: Requeue on Again and schedule effect
status: done
epic: EPIC-007
parents: [STORY-0013]
estimate: 3
labels: [m3, rehearsal]
acceptance_criteria:
  - "With `requeue_again`, an item rated Again returns to the end of the queue (at most 3 times per item per run)"
  - "Events carry enough for the shell to apply the schedule only when the plan's effect is UpdateSchedule"
  - "Skipped items are not requeued"
---

## Context
Practice runs must not disturb the schedule; spaced runs behave like learning steps.

## Description
Requeue on Again and schedule effect.

## Acceptance Criteria
- [x] With `requeue_again`, an item rated Again returns to the end of the queue (at most 3 times per item per run)
- [x] Events carry enough for the shell to apply the schedule only when the plan's effect is UpdateSchedule
- [x] Skipped items are not requeued

## Design Notes
Requeue in `step.rs`: Again with `requeue_again` pushes the item to the end, at most `MAX_REQUEUES` (3) per item per run; skips and non-requeue plans never requeue. Applying the schedule effect is left to the shell, which reads `plan.effect` (wired in TASK-0033).

## Dependencies
TASK-0031.

## Open Questions
None.
