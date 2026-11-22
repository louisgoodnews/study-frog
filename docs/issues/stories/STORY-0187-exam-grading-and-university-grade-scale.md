---
id: STORY-0187
type: story
title: "Mock Exam Grading Report, German University Grade Scale (1.0 - 5.0), and Chapter Analytics"
status: review
epic: EPIC-063
parents: ['EPIC-063']
estimate: 3
labels: [m14, rehearse, ui, report]
acceptance_criteria:
  - 'Total score summation with percentage calculation and configurable pass mark threshold'
  - 'German university grading scale conversion mapping percentages to 1.0, 1.3, 1.7, ..., 4.0, 5.0'
  - 'Comprehensive exam review screen showing question-by-question student answers, reference solutions, and earned points'
---

## Context
After taking a mock exam, the learner needs a definitive evaluation replicating the real university grading experience: a German academic grade (1.0 - 5.0), percentage earned, and a breakdown of performance by course chapter to focus subsequent study sessions.

## Description
Build the exam evaluation and review system:
- Implement `percentage_to_german_grade(pct: f32, pass_mark: f32) -> (GradeBadge, &'static str)` supporting standard German university thresholds:
  - $\ge 95\% \implies 1.0$ (Sehr Gut)
  - $\ge 90\% \implies 1.3$
  - $\ge 85\% \implies 1.7$ (Gut)
  - $\dots$
  - $\ge 50\% \implies 4.0$ (Ausreichend / Bestanden)
  - $< 50\% \implies 5.0$ (Nicht Bestanden)
- Render the Exam Result View in `studyfrog-ui`:
  - Grade badge and total score (e.g. "82/100 Points • Grade 2.0 (Gut) • Passed").
  - Breakdown by topic showing where points were lost.
  - Expandable review of each question with the student's answer, SLM points awarded, and model answer.

## Acceptance Criteria
- [ ] Total score summation with percentage calculation and configurable pass mark threshold
- [x] German university grading scale conversion mapping percentages to 1.0, 1.3, 1.7, ..., 4.0, 5.0
- [x] Comprehensive exam review screen showing question-by-question student answers, reference solutions, and earned points

## Design Notes
Keep grading pure and deterministic in `studyfrog-rehearsal`. Adhere to `palette_of(ui)` theme tokens and Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-063, STORY-0185, STORY-0186

## Implementation so far (2026-11-21)
`studyfrog-rehearsal::grade` holds the percentage, the eleven-band German scale, the pass mark (below it the grade is always 5.0), and `exam_result`. Tests: `tests/grade.rs`.

## Implementation (2026-11-22)
The exam can now be written and graded in the app (`setup::mock_exam`, `setup::mock_exam::grading`, `view::mock_exam_view`, `view::mock_exam_view::result`). "Start the exam" shows one answer box per question; "Finish and grade" checks each answer against its question's rubric (the keyword check, `rubric_eval::fallback_evaluate`): a question's points are its points times the share of rubric points the answer makes. The result shows the total and percentage, the grade and pass or fail (pass mark 50 %), the points by topic, and for each question the student's answer, the model answer and the rubric points made.

Open: the results are not kept after the view closes. (Model grading: see the notes below.)

Tests: `studyfrog-ui/tests/mock_exam_ui.rs` (a full answer earns all points, an unwritten exam none), and `setup/mock_exam/grading.rs` (grading, breakdown). A screenshot (`--screen mock-graded`) shows the result.

## Implementation notes (2026-11-22, model grading)

Built:
- Model grading of written answers (`studyfrog-agent::exam_grade`): the local model judges each rubric point from the student's answer alone, in JSON, matched to the rubric by text. Points it does not mention count as not met.
- Fallback: with no local model selected, the keyword check grades at once (no model call). If the model cannot be reached or its reply cannot be read, the keyword check grades that answer, and the reason is shown under the question.
- Only a local model grades (as in the case-study check): a remote model is not sent the student's answers.
- The worker grades in one call per exam (`exec::mock`, `MockExamEffect::Grade`); "Finish and grade" shows "Grading your answers with the local model" while it runs.
- The result says who graded: all by the model, all by word overlap, or how many of each. Each question shows whether the model or the keyword check made the call.
- Summation: points are summed, not averaged by percentage (test: 1 of 3 and 10 of 10 is 84.6 %, grade 2.0). The pass mark is a parameter of `exam_result` and applies to the whole exam; the exam view uses 50 %. It is not yet a setting in the UI.

Tests: `studyfrog-agent/tests/exam_grade.rs` (model judgements, prompt contents, no model, failed call, unreadable reply, skipped rubric point, empty rubric); `studyfrog-ui/src/setup/mock_exam/grading.rs` (keyword grading with no model; model grading with a fake model); `studyfrog-ui/tests/mock_exam_model_ui.rs` (Finish with no model grades at once; with a local model it asks for grading, then shows the result); `studyfrog-rehearsal/tests/grade.rs` (summation by points, configured pass mark).

Not done or not verified:
- A real local model was not run: this container has none. Model grading is tested with a fake model only. How well a real model judges rubric points is unmeasured.
- A person at a window should check the result view's layout, the grading line and the per-question reasons.
- The results are still not kept after the view closes.

## Open Questions
None.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): criterion 1 NOT MET for the configurable pass mark: the result view uses a fixed 50 % pass mark and the app has no setting for it (the parameter exists in `studyfrog-rehearsal::grade::exam_result`). Unticked by this audit. Summation and percentage are shown ("0 / 6 points - 0 % - Grade 5.0 (Nicht bestanden)", /tmp/claude-0/audit/g5/0187-result.png); a non-zero percentage was not reached in the app, so the summation is checked by the tests only.
Criterion 2 MET: the bands in `crates/studyfrog-rehearsal/src/grade.rs` run 95, 90, 85, 80, 75, 70, 65, 60, 55, 50 % to grades 1.0, 1.3, 1.7, 2.0, 2.3, 2.7, 3.0, 3.3, 3.7, 4.0, and below the pass mark 5.0; the app shows 5.0 for 0 % (/tmp/claude-0/audit/g5/0187-result.png). Intermediate bands were read in the code, not on screen.
Criterion 3 MET: the review lists each question with the student's answer, the model answer and the rubric points, and marks each as Missing (/tmp/claude-0/audit/g5/0187-result.png). The keyword check gives 0 to a correct answer (BUG-0038). Results are not kept after the view closes (open, per the author's note).

## Verification (2026-11-22)

- Keyword grading with no model verified: result "0 / 16 points - 0 % - Grade 5.0 (Nicht bestanden)" with the note "Graded by word overlap: no local model is selected" (`/tmp/claude-0/verifyA/23-exam-graded.png`).
- Model grading NOT CHECKABLE (no model installed). Typed answers could not be entered in this environment, so a correct answer in other words was not checked.
- The model answer for question 1 shows the card's question text followed by its answer ("Was ist der Mittelwert? Summe durch Anzahl"), not a reference answer. Not filed here; see BUG-0038 for the keyword check.
- Ticks are left as set.

Review (2026-11-22): checked in the running app (no local model), `/tmp/claude-0/review-e/`.
- Criterion 1 (summation, percentage, configurable pass mark): NOT MET. The sums and percentage are shown ("0 / 14 points · 0 % · Grade 5.0 (Nicht bestanden)", `/tmp/claude-0/review-e/10-result.png`), but the pass mark is fixed at 50 % in the view and no setting exists in the app. A non-zero percentage could not be reached: the keyword check gave 0 for every answer, including answers with the reference meaning (BUG-0038). The summation rule (11 of 13 = 84.6 %, grade 2.0) is covered by `tests/grade.rs` only. Left unticked.
- Criterion 2 (bands 1.0 to 5.0): MET by the code and the tests, with the screen showing the fail band. `grade.rs` bands run 95, 90, 85, 80, 75, 70, 65, 60, 55, 50 % to 1.0, 1.3, 1.7, 2.0, 2.3, 2.7, 3.0, 3.3, 3.7, 4.0 and below the pass mark 5.0. The worked examples check out by hand: 82 % gives 2.0 (Gut) and 11 of 13 (84.6 %) gives 2.0. On screen: 0 % gives 5.0 (`/tmp/claude-0/review-e/10-result.png`). Tick left as set.
- Criterion 3 (review screen): MET. Each question shows the student's answer, the model answer and the rubric points, marked Missing (`/tmp/claude-0/review-e/10-result.png`, `11-result-scroll.png`, `16-bottom.png`). The reference lines for card questions repeat the question (BUG-0052). Tick left as set.
- Layout at 960x640: the exam view fits (`/tmp/claude-0/review-e/21-960-preview.png`, `22-960-started.png`).
- Status stays review: criterion 1 is open.

