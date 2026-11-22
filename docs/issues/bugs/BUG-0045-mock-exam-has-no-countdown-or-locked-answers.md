---
id: BUG-0045
type: bug
title: "Mock exam view has no countdown timer, no auto-submit and no locked answers"
status: backlog
epic: EPIC-063
parents: ['EPIC-063']
estimate: 3
labels: [m14, rehearse, ui]
acceptance_criteria:
  - "The mock exam view shows the time remaining, counting down from the exam's total duration"
  - "A warning shows at 10 minutes and at 2 minutes remaining, and the exam is submitted automatically when the time runs out"
  - "A submitted answer is locked and cannot be edited until the result is shown"
---

## Context
Found while checking EPIC-063 criterion 1 (2026-11-22). A mock exam made from the Statistik notes shows "48 minutes" in its preview (`/tmp/claude-0/verifyA/20-mock-preview.png`). Once started, the exam view shows no countdown: the run shows an elapsed clock only, and the answer boxes stay editable until "Finish and grade" (`/tmp/claude-0/verifyA/21-exam-started.png`).

## Description
The domain code exists: `studyfrog-rehearsal::mock_timer` (10 and 2 minute alerts, expiry) and `mock_nav` (answer state and review). `studyfrog-ui` does not use `mock_timer` (no reference in the crate), so the exam view never shows a remaining time, never warns, and never submits on expiry. Answers are not locked on submission.

Wire the timer into the mock exam view and the exec path, following the domain rules already tested in `tests/mock_exam.rs`.

## Acceptance Criteria
- The mock exam view shows the time remaining, counting down from the exam's total duration
- A warning shows at 10 minutes and at 2 minutes remaining, and the exam is submitted automatically when the time runs out
- A submitted answer is locked and cannot be edited until the result is shown

## Design Notes
Keep all functions free (P1) and files within the limit (P6). Reuse `mock_timer::timer_status` and `alerts_between` rather than a second clock.

## Dependencies
- EPIC-063
- STORY-0185 (mock exam structure and timer controls)
- STORY-0187 (grading and result view)

## Open Questions
- Whether the exam should also run in the Altklausur lab screen, which does not exist yet (STORY-0186 notes).
