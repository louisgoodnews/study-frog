---
id: TASK-0297
type: task
title: "Wizard step model: pure state of the optional steps from settings and installed models"
status: backlog
epic: EPIC-069
parents: ['STORY-0212']
estimate: 3
labels: [m14, ui, tooling]
acceptance_criteria:
  - "The next and previous step, skip, and done are pure functions over the step list and the installed state"
  - "Steps: vault, local model, remote provider, MiniLM, summary; a step already satisfied is shown as done and can still be revisited"
  - "Unit tests cover every transition, including skip and reopen from Settings"
---

## Context
STORY-0212 needs one place that decides which step is next, so the screen and the control protocol agree.

## Description
Add the step model in a pure module (no inherent `impl` blocks, files under 250 lines).

## Acceptance Criteria
- [ ] Pure transitions for next, previous, skip and done
- [ ] Steps and satisfied state derived from settings and installed models
- [ ] Unit tests for every transition

## Design Notes
Keep the step list data-only so the control protocol can print it.

## Dependencies
- STORY-0212.

## Open Questions
None.
