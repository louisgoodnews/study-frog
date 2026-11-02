---
id: STORY-0059
type: story
title: Familiarity in reports, on Home and in scheduling
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, report, ui, srs]
acceptance_criteria:
  - "A per-item report shows how familiar the user is with each item, stack and subject"
  - "Home lists the items and stacks that need the most attention, with the reason"
  - "Familiarity is a criterion of scheduling: items near 1.0 are not rehearsed in the near future, the least familiar come first, and more familiar items get longer intervals than less familiar ones"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Report the value to the user and use it, with the schedule, to decide what to study next.

## Acceptance Criteria
- [ ] A per-item report shows how familiar the user is with each item, stack and subject
- [ ] Home lists the items and stacks that need the most attention, with the reason
- [ ] Familiarity is a criterion of scheduling: items near 1.0 are not rehearsed in the near future, the least familiar come first, and more familiar items get longer intervals than less familiar ones

## Design Notes
Reports get a familiarity view per item (distribution, per stack, per subject, change over time) that says how good the user is with each item and why. Home gets a 'Needs attention' card with the least familiar items, weighted by priority and due dates. Run setup gets an order 'least familiar first' next to the existing orders and filters. Scheduling (the owner's decision, 2026-11-02: familiarity is a sub-criterion of scheduling): under a daily budget the queue prefers lower familiarity among due items, and items at or above a threshold (a setting, proposal 0.95) are left out of budget-limited queues and the calendar planner (STORY-0062) plans their next review later. Familiarity also scales the scheduler's own interval (the owner's decision, 2026-11-02: it affects both selection and interval): more familiar items are scheduled more generously than less familiar or unfamiliar ones. A pure function takes the interval the algorithm (FSRS or SM-2) computed and the familiarity and returns a scaled interval, with a multiplier that rises with familiarity within a lower and an upper bound (the proposal is 0.7 to 1.5; the spec fixes the curve), never past an exam or due date, and a setting for its strength (0 switches it off). The algorithm's interval stays the base. The default strength is set only after a simulated history shows retention does not fall by more than the spec allows while workload does (numbers recorded in `docs/specs/reports.md`). Every suggestion says why ('few reviews', 'two hard ratings', 'not seen for 40 days'). Marks (easy, medium, difficult) still change no schedule on their own.

Tasks: TASK-0138, TASK-0139, TASK-0140, TASK-0141.

## Dependencies
Needs STORY-0058. The planner part touches STORY-0062.

## Open Questions
Answered by the owner on 2026-11-02: familiarity affects both the selection, order and planned day and the scheduler's own interval. Still open: how strong by default? Proposal: set by the simulation described above, with the strength adjustable in Settings.
