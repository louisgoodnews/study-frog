---
id: TASK-0073
type: task
title: "Run setup screen (plan picker)"
status: done
epic: EPIC-008
parents: [STORY-0017]
estimate: 3
labels: [m4, ui, rehearsal]
acceptance_criteria:
  - "Pick order (spaced, in order, shuffled, hardest/easiest first, stalest, lowest recall), filters (kind, stack, tags, state, difficulty), limit, requeue and schedule effect"
  - "Defaults follow the standard plans; a practice-only run is clearly labelled"
  - "Shows how many items match before starting"
---

## Context
Owner requirement: rehearsal is not only spaced repetition.

## Description
Run setup screen (plan picker).

## Acceptance Criteria
- [x] Pick order (spaced, in order, shuffled, hardest/easiest first, stalest, lowest recall), filters (kind, stack, tags, state, difficulty), limit, requeue and schedule effect
- [x] Defaults follow the standard plans; a practice-only run is clearly labelled
- [x] Shows how many items match before starting

## Design Notes
`ui/src/setup.rs` (pure): `SetupForm` -> validated `RunPlan` (`plan_from_form`, explains bad limits/difficulty), presets with the documented defaults, `apply_edit`, and the update that asks for a fresh count after every edit. The count comes from `studyfrog_service::queue::preview_queue`, which now builds queues from the index (`load_indexed_items`), not from parsed files. Practice-only is labelled on screen. The "Start rehearsal" button arrives with TASK-0039. The day used for daily limits is the UTC day (a local-midnight setting needs a time-zone source; flagged).

## Dependencies
TASK-0037.

## Open Questions
Saved presets? Defer.
