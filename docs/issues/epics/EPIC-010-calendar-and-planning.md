---
id: EPIC-010
type: epic
title: Calendar and planning
status: in_progress
parents: []
labels: [m5, calendar]
acceptance_criteria:
  - "Sessions can be scheduled, linked to a stack, and recur"
  - "Recurrence expansion is pure and property-tested"
  - "Sessions persist across restarts"
  - "A session can start a rehearsal of its stack"
---

## Context
Plan learning sessions (docs/VISION.md).

## Description
Pure session and recurrence model in `studyfrog-calendar`; local time handled at the edge.

## Acceptance Criteria
- [ ] Sessions can be scheduled, linked to a stack, and recur
- [ ] Recurrence expansion is pure and property-tested
- [ ] Sessions persist across restarts
- [ ] A session can start a rehearsal of its stack

## Design Notes
Children: STORY-0020.

## Dependencies
M4 exit.

## Open Questions
None.
