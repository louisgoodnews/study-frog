---
id: STORY-0247
type: story
title: "Adaptive preparation-window estimator and windowed item scheduler"
status: backlog
epic: EPIC-076
parents: ['EPIC-076']
estimate: 5
labels: [m14, srs, report, calendar]
acceptance_criteria:
  - 'A pure estimator function proposes a preparation window in weeks from linked item count, familiarity, FSRS stability/retrievability, historical accuracy, and daily capacity, returning both the week count and a short explanation of factors'
  - 'A pure windowed scheduler distributes linked items across [exam_date - N weeks, exam_date - 1 day], front-loading new and low-familiarity items and pacing spaced reviews toward exam day'
  - 'Re-balancing recomputes remaining days cleanly when items are added/removed, the exam date shifts, or planned days are missed'
  - 'Deterministic unit and property tests verify schedule coverage, date bounds, and re-balancing behavior'
---

## Context
The core value of the Exam Scheduler (`EPIC-076`) is turning a target exam date and a set of study items into a realistic, evidence-based study plan—either for a week window chosen by the user or one proposed by StudyFrog from the learner's own stats.

## Description
Implement two pure, deterministic engines in `studyfrog-calendar` / `studyfrog-srs`:
1. **Preparation-Window Estimator**: computes recommended weeks and factor notes from item count, familiarity, FSRS state, lapse/accuracy stats, and daily study budget.
2. **Windowed Item Scheduler & Re-balancer**: distributes associated items across the preparation window, prioritizes weak/new material early, respects daily caps, and re-balances smoothly when plans change.

## Acceptance Criteria
- [ ] A pure estimator function proposes a preparation window in weeks from linked item count, familiarity, FSRS stability/retrievability, historical accuracy, and daily capacity, returning both the week count and a short explanation of factors
- [ ] A pure windowed scheduler distributes linked items across `[exam_date - N weeks, exam_date - 1 day]`, front-loading new and low-familiarity items and pacing spaced reviews toward exam day
- [ ] Re-balancing recomputes remaining days cleanly when items are added/removed, the exam date shifts, or planned days are missed
- [ ] Deterministic unit and property tests verify schedule coverage, date bounds, and re-balancing behavior

## Design Notes
- Reuse existing `studyfrog-srs` and `studyfrog-report` familiarity/velocity metrics; inject clocks/dates as arguments (P2) so all scheduling math is 100% deterministic.

## Dependencies
- `EPIC-076`, `STORY-0246`
- Child tasks: `TASK-0307`, `TASK-0308`

## Open Questions
None.
