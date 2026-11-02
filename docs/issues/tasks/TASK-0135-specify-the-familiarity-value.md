---
id: TASK-0135
type: task
title: Specify the familiarity value
status: backlog
epic: EPIC-023
parents: [STORY-0058]
estimate: 3
labels: [m10, docs, decision]
acceptance_criteria:
  - "docs/specs/familiarity.md defines the inputs, the size of each change (marginal for a view, significant for a rating), the view-time heuristic, the fall without contact and when it pauses, the note case, the stack roll-up and the confidence flag"
  - "The range 0.0 to 1.0 is fixed in the spec and the UI shows it as a percentage"
  - "The owner has accepted the formula's behaviour on worked examples"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Write the formula and its reasoning, with worked examples of a card over ten reviews.

## Acceptance Criteria
- [ ] docs/specs/familiarity.md defines the inputs, the size of each change (marginal for a view, significant for a rating), the view-time heuristic, the fall without contact and when it pauses, the note case, the stack roll-up and the confidence flag
- [ ] The range 0.0 to 1.0 is fixed in the spec and the UI shows it as a percentage
- [ ] The owner has accepted the formula's behaviour on worked examples

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
