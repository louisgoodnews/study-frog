---
id: BUG-0045
type: bug
title: "Mock exam view has no countdown timer, no auto-submit and no locked answers"
status: done
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

Review (2026-11-22): still open. The started exam shows no countdown at 1280x800 or 960x640, answers stay editable until "Finish and grade" and no answer is locked (`/tmp/claude-0/review-e/08-started.png`, `22-960-started.png`). Status stays backlog.

## Fix (2026-11-23)

Cause: `studyfrog-ui` never used `studyfrog-rehearsal::mock_timer`; the exam view had no clock, no expiry and no lock.

Changes:
- `setup/mock_exam/clock.rs` (new): `elapsed_sec` and `tick`. A started, unlocked exam reads the app clock once a tick; `alerts_between` reports the expiry once, and the exam is then submitted through the same `finish` as "Finish and grade". Called from `update.rs` on `Msg::Tick`.
- `setup/mock_exam.rs`: the state keeps `started_at`, `elapsed_sec` and `locked`. "Start the exam" stamps the start; an answer is ignored while locked.
- `setup/mock_exam/finish.rs`: with a local model, the answers are locked from Finish until the result is shown; the lock clears when the result arrives.
- `view/mock_exam_view.rs`: "Time left mm:ss" from `timer_status`, "Ten minutes left" and "Two minutes left" in the danger colour; while locked, "Submitted. Your answers are locked until the result is shown.", the answers are read-only and Finish is hidden.

Tests: `studyfrog-ui/tests/mock_exam_ui.rs` `the_clock_counts_down_and_submits_the_exam_when_the_time_runs_out` (a reading at 10 minutes left changes nothing; the expiry grades the exam), `an_answer_is_locked_once_the_exam_is_submitted`.

Verified in the running app (fixed build, 2026-11-23, a one-note stack gives a 12-minute exam, started at 0):
- The countdown: "Time left 11:57" (`/tmp/claude-0/exam/t-03-started.png`).
- Ten minutes left: "Time left 09:30", "Ten minutes left" in red (`/tmp/claude-0/exam/t-04-ten-warning.png`).
- Two minutes left: "Time left 01:39", "Two minutes left" (`/tmp/claude-0/exam/t-05-two-warning.png`).
- Auto-submit: with no interaction, the exam at 12:00 showed its result, "0 / 4 points · 0 % · Grade 5.0 (Nicht bestanden)" (`/tmp/claude-0/exam/t-06-expired.png`).
- Locked: with a local model selected (a provider at 127.0.0.1 that accepts and never answers, so grading stays in progress), the view reads "Grading your answers with the local model", "Submitted. Your answers are locked until the result is shown.", and the answer box is read-only (`/tmp/claude-0/exam/l-05-judging.png`). A typed addition during grading did not change the answer (`/tmp/claude-0/exam/l-06-locked-typed.png`), though typing did not register in that instance at all, so the typed check is weak; the lock is also enforced in `update` and tested.

Before the fix: no countdown at the top of the exam (`/tmp/claude-0/exam/b04b-top.png`).
