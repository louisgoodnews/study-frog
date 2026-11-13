---
id: STORY-0125
type: story
title: "Specify the exam run mode and its grade scale"
status: done
epic: EPIC-033
parents: [EPIC-033]
estimate: 2
labels: [m12, docs, rehearsal]
acceptance_criteria:
  - "docs/specs/exam-mode.md defines the exam rules (one-way progression, no skipping, no going back, no peeking, no advancing without a non-empty written answer, answers locked on submission, no manual rating)"
  - "The spec defines how each answer is scored (cards against back, questions against reference, accepted and rubric), how the exam percentage is aggregated and rounded, and the full grade table from A+ / 1+ to F / 5.0 with the 60 % pass threshold"
  - "The spec defines what is stored (run mode name, exam result record), what an aborted or timed-out exam means, and what the control protocol offers"
---
## Context
EPIC-033 needs one written source of truth for the rules before engine, service and UI are built.

## Description
Write `docs/specs/exam-mode.md` from EPIC-033 and PLANNED_FEATURES.md section 1, deciding the open details (rounding, unanswered items, abort, countdown, schedule effect, notes).

## Acceptance Criteria
- [x] docs/specs/exam-mode.md defines the exam rules (one-way progression, no skipping, no going back, no peeking, no advancing without a non-empty written answer, answers locked on submission, no manual rating)
- [x] The spec defines how each answer is scored (cards against back, questions against reference, accepted and rubric), how the exam percentage is aggregated and rounded, and the full grade table from A+ / 1+ to F / 5.0 with the 60 % pass threshold
- [x] The spec defines what is stored (run mode name, exam result record), what an aborted or timed-out exam means, and what the control protocol offers

## Design Notes
The rehearsal spec (`docs/specs/rehearsal-run.md`) links to it; the scoring spec stays the reference for a single comparison.

## Dependencies
None.

## Open Questions
None.
