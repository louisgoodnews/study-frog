---
id: STORY-0058
type: story
title: Familiarity value
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, srs, report, core, store]
acceptance_criteria:
  - "Every note, card, question and stack has a familiarity value, a float from 0.0 to 1.0 where 1.0 means the user knows it very well, that is updated by views and ratings"
  - "A view raises it marginally, an easier rating raises it significantly and a harder rating lowers it significantly, shorter view times raise it and longer ones lower it, and the formula is documented"
  - "The value is validated on a recorded history and its error is written down"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Measure how familiar the user is with each item and stack, so StudyFrog can tell what needs attention and what can wait.

## Acceptance Criteria
- [ ] Every note, card, question and stack has a familiarity value, a float from 0.0 to 1.0 where 1.0 means the user knows it very well, that is updated by views and ratings
- [ ] A view raises it marginally, an easier rating raises it significantly and a harder rating lowers it significantly, shorter view times raise it and longer ones lower it, and the formula is documented
- [ ] The value is validated on a recorded history and its error is written down

## Design Notes
The owner defined the meaning on 2026-11-02: familiarity is how familiar the user is with a flashcard, note, question or stack. It grows marginally with every view, grows significantly with an easier rating and drops significantly with a harder one; it grows the shorter the view time and the easier the ratings the item has, and a longer view time means less familiarity (a heuristic). A card at 1.0 is assumed to be known very well and need not be rehearsed in the near future. The value is a sub-criterion of scheduling (STORY-0059) and a per-item report of how good the user is with an item. Inputs: views and their durations (item page open time, not counting idle time), answer latency, the ratings (again, hard, good, easy) and the marks (easy, medium, difficult), suggested difficulty and priority as small adjustments, and the rating history. View time is judged against the content: the reading-speed baseline and the length of the text give the expected time, so a long note is not punished for taking longer. Without contact the value falls slowly (the forgetting curve sets the shape), but the fall stops while the item's status is Complete and after the exam date of its stack has passed, so finished or examined material does not drift to 'unfamiliar'. Notes have views and edits but no ratings, so their value rests on those and carries a lower confidence flag. A stack's value is the mean of its items weighted by priority. The value is a pure function of the history plus a clock (functional style, no hidden state). Build, do not buy (P10). Validate on the labelled reviews of the simulated history already used by the forecast, and record the precision in `docs/specs/reports.md`; measure the cost of valuing 10k items and keep it within a budget set in the spec.

Tasks: TASK-0135, TASK-0136, TASK-0137.

## Dependencies
STORY-0061 (statuses) and the exam dates of the calendar for pausing the fall. The value feeds STORY-0059 and the status suggestions of STORY-0061.

## Open Questions
Answered by the owner on 2026-11-02: what familiarity means, the range 0.0 to 1.0 and the direction of every input. Confirmed: 'decreases significantly upon higher rating' means a harder rating (again, hard, difficult). Familiarity also falls slowly over time without contact, but only while the item's status is not Complete and the exam date of its stack has not passed (owner, 2026-11-02); the rate is set in the spec.
