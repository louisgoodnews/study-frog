---
id: TASK-0147
type: task
title: Move an entry to another day
status: backlog
epic: EPIC-023
parents: [STORY-0062]
estimate: 3
labels: [m10, core, store]
acceptance_criteria:
  - "Moving an entry sets the due date of its items to the new day without changing their memory state"
  - "A moved item stays out of the rehearsal queue until its new day, and the next rehearsal replaces the moved date with the scheduler's own"
  - "A move can be undone, and a move past an exam or due date of the stack produces a warning"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the reschedule operation to the service layer with tests over a real vault and database.

## Acceptance Criteria
- [ ] Moving an entry sets the due date of its items to the new day without changing their memory state
- [ ] A moved item stays out of the rehearsal queue until its new day, and the next rehearsal replaces the moved date with the scheduler's own
- [ ] A move can be undone, and a move past an exam or due date of the stack produces a warning

## Design Notes
None.

## Dependencies
The planner task of this story.

## Open Questions
None.
