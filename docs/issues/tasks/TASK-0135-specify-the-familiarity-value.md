---
id: TASK-0135
type: task
title: Specify the familiarity value
status: review
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
- [x] docs/specs/familiarity.md defines the inputs, the size of each change (marginal for a view, significant for a rating), the view-time heuristic, the fall without contact and when it pauses, the note case, the stack roll-up and the confidence flag
- [x] The range 0.0 to 1.0 is fixed in the spec and the UI shows it as a percentage
- [ ] The owner has accepted the formula's behaviour on worked examples

## Design Notes
Written 2026-11-02 in `docs/specs/familiarity.md`. The constants (gains 0.45 and 0.30, losses 0.25 and 0.50, half-life 365 days) were tuned on a few reference histories so that a mostly-good card reaches about 0.9 and a lapse drops it to about 0.4; TASK-0137 calibrates them on the simulated history. Proposals awaiting the owner: the note cap of 0.8, the mark factors 1.1 and 0.8, at most one counted view per hour, and the fall pausing only for the stretch since the last event.

## Dependencies
None.

## Open Questions
Does the owner accept the behaviour on the worked examples (section 8 of the specification)?
