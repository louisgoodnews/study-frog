---
id: STORY-0128
type: story
title: "Exam mode in the rehearsal screens"
status: done
epic: EPIC-033
parents: [EPIC-033]
estimate: 8
labels: [m12, ui]
acceptance_criteria:
  - "The Rehearsal setup offers Exam next to Flip cards and Type answers and explains its rules; the plan of an exam leaves notes out and does not change the schedule"
  - "The exam run screen shows the prompt and a written-answer field only: no show-answer, skip, mark or rating controls, Submit is disabled until the answer is non-empty, a submitted answer is locked and the next item follows"
  - "The result screen shows the final percentage, the grade badge (A+ / 1+ to F / 5.0), pass or fail, and a scrollable review of every item with the written answer, the reference and the score; exam results appear in the Runs report and the diary prompt is pre-filled with the result"
  - "Harness tests cover setup, run and result, and screenshots of the setup, run and result screens were looked at"
---
## Context
The learner meets the exam mode in the Rehearsal screen.

## Description
Setup and run screen (TASK-0238), result screen and review (TASK-0239), Runs report, diary prompt and screenshots (TASK-0240).

## Acceptance Criteria
- [x] The Rehearsal setup offers Exam next to Flip cards and Type answers and explains its rules; the plan of an exam leaves notes out and does not change the schedule
- [x] The exam run screen shows the prompt and a written-answer field only: no show-answer, skip, mark or rating controls, Submit is disabled until the answer is non-empty, a submitted answer is locked and the next item follows
- [x] The result screen shows the final percentage, the grade badge (A+ / 1+ to F / 5.0), pass or fail, and a scrollable review of every item with the written answer, the reference and the score; exam results appear in the Runs report and the diary prompt is pre-filled with the result
- [x] Harness tests cover setup, run and result, and screenshots of the setup, run and result screens were looked at

## Design Notes
New modules `exam_ui.rs` (messages, effects, pure update) and `view/exam_result.rs`; `--screen exam-setup|exam|exam-result` for screenshots.

## Dependencies
STORY-0127.

## Open Questions
None.
