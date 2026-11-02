---
id: TASK-0184
type: task
title: Replay familiarity against the user's review history to approve it
status: backlog
epic: EPIC-023
parents: [STORY-0059]
estimate: 3
labels: [m10, srs, report]
acceptance_criteria:
  - "A replay walks the user's review history in order and scores plain FSRS recall predictions against the familiarity-adjusted ones by log loss and a binned calibration table"
  - "The model is approved for a user only with at least 1000 reviews and a lower log loss on the last 20 percent of the history after fitting on the rest, and the verdict and the change in reviews per year are shown in Settings"
  - "The replay is a pure function with tests on simulated histories, and 10k reviews replay within a measured budget"
---

## Context
The owner asked on 2026-11-02 that familiarity influence FSRS so that well-known cards are scheduled more generously (a card rated Easy at familiarity 0.5 should come back after about 2.5 months, not 5), and tasked the research and proposal of a model that combines the two. The 30 days mentioned for familiarity 1.0 were an example, never a cap.

## Description
Decide per user, from their own data, whether familiarity improves FSRS predictions, before the factor of TASK-0141 is switched on.

## Acceptance Criteria
- [ ] A replay walks the user's review history in order and scores plain FSRS recall predictions against the familiarity-adjusted ones by log loss and a binned calibration table
- [ ] The model is approved for a user only with at least 1000 reviews and a lower log loss on the last 20 percent of the history after fitting on the rest, and the verdict and the change in reviews per year are shown in Settings
- [ ] The replay is a pure function with tests on simulated histories, and 10k reviews replay within a measured budget

## Design Notes
Method in section 7 of docs/research/familiarity-and-fsrs.md. Fit beta, S_half and the expected-familiarity curve on the first 80 percent of the history and test on the rest. The workload effect is estimated with the forecast's simulator.

## Dependencies
STORY-0058 (the familiarity values), TASK-0141 and the view-time records of TASK-0136.

## Open Questions
None.
