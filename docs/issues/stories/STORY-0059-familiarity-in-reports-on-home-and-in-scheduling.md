---
id: STORY-0059
type: story
title: Familiarity in reports, on Home and in scheduling
status: done
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
- [x] A per-item report shows how familiar the user is with each item, stack and subject
- [x] Home lists the items and stacks that need the most attention, with the reason
- [x] Familiarity is a criterion of scheduling: items near 1.0 are not rehearsed in the near future, the least familiar come first, and more familiar items get longer intervals than less familiar ones

## Design Notes
Reports get a familiarity view per item (distribution, per stack, per subject, change over time) that says how good the user is with each item and why. Home gets a 'Needs attention' card with the least familiar items, weighted by priority and due dates. Run setup gets an order 'least familiar first' next to the existing orders and filters. Scheduling (the owner's decision, 2026-11-02: familiarity is a sub-criterion of scheduling): under a daily budget the queue prefers lower familiarity among due items, and items at or above a threshold (a setting, proposal 0.95) are left out of budget-limited queues and the calendar planner (STORY-0062) plans their next review later. Familiarity also changes the scheduler's own interval (the owner's decision, 2026-11-02): more familiar items are scheduled more generously than less familiar ones, so a card rated Easy with a familiarity of 0.5 comes back after about 2.5 months where plain FSRS says 5. The researched model is in `docs/research/familiarity-and-fsrs.md` (TASK-0183): a multiplier on the FSRS interval, `m = exp(beta * (F - F_expected(S)))`, that depends on how familiarity compares with the familiarity FSRS's own stability would lead one to expect, so it reacts to disagreement between the two. The stored FSRS state is never changed and a strength of 0 gives exactly the plain interval. It is on from the start (owner's decision, 2026-11-02) with the accepted defaults (beta 2, clamp 0.25 to 2, no change before the third review); a replay on the user's own review history (TASK-0184) tells the user whether it predicts recall better than plain FSRS, and Settings and each stack can switch it off. Never past an exam or due date. The 30-day figure that came up earlier was only an example and is not a cap. Every suggestion says why ('few reviews', 'two hard ratings', 'not seen for 40 days'). Marks (easy, medium, difficult) still change no schedule on their own.

Tasks: TASK-0138, TASK-0139, TASK-0140, TASK-0141, TASK-0183 (research, done) and TASK-0184.

## Dependencies
Needs STORY-0058. The planner part touches STORY-0062.

## Open Questions
Answered by the owner on 2026-11-02: familiarity affects both the selection, order and planned day and the scheduler's own interval. Strength: the owner clarified on 2026-11-02 that 30 days at familiarity 1.0 was an example, never a cap; the research proposes a relative factor with defaults beta 2, clamp 0.25 to 2 and no change before the third review. Decided by the owner on 2026-11-02 (section 10 of the research): the relative reading, the defaults, and on from the start. Nothing is open.
