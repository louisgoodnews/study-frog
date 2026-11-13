---
id: TASK-0166
type: task
title: "Guided tutorials"
status: done
epic: EPIC-023
parents: [STORY-0066]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "A first-run tutorial offers a stack, a card, a rehearsal and the calendar, and can be skipped and repeated"
  - "Each tutorial step opens the screen it describes and is done when the user has done it"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the tutorial runner as a pure state machine and the screen.

## Acceptance Criteria
- [x] A first-run tutorial offers a stack, a card, a rehearsal and the calendar, and can be skipped and repeated
- [x] Each tutorial step opens the screen it describes and is done when the user has done it

## Design Notes
`studyfrog-ui/src/tutorial.rs`: a step is done when a count read from the state grew since the step began (stacks, items, ratings in the run, planned sessions once the calendar is read). The offer is remembered as `tutorial_offered` in `app.toml`.

## Dependencies
The viewer task of this story.

## Open Questions
None.
