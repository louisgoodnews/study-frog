---
id: TASK-0073
type: task
title: Run setup screen (plan picker)
status: backlog
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
- [ ] Pick order (spaced, in order, shuffled, hardest/easiest first, stalest, lowest recall), filters (kind, stack, tags, state, difficulty), limit, requeue and schedule effect
- [ ] Defaults follow the standard plans; a practice-only run is clearly labelled
- [ ] Shows how many items match before starting

## Design Notes
Builds a `RunPlan`; queue size preview via `studyfrog_service::queue::new_run`.

## Dependencies
TASK-0037.

## Open Questions
Saved presets? Defer.
