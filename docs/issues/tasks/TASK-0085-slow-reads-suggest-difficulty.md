---
id: TASK-0085
type: task
title: Slow reads suggest difficulty
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, rehearsal, ui]
acceptance_criteria:
  - "Reading over 200 % of the expected time suggests a harder rating: Good to Hard, Easy to Good"
  - "Again and Hard are never softened; short texts are ignored"
  - "The review explains the suggestion"
---

## Context
A slow read means struggle even when the answer came in the end.

## Description
Slow reads suggest difficulty.

## Acceptance Criteria
- [x] Reading over 200 % of the expected time suggests a harder rating: Good to Hard, Easy to Good
- [x] Again and Hard are never softened; short texts are ignored
- [x] The review explains the suggestion

## Design Notes
Reason::Slow in `rehearsal::reading`, property-tested with the fast rules: a slow suggestion is always harder than the given rating.

## Dependencies
TASK-0082.

## Open Questions
None.
