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
- [x] Total score summation with percentage calculation and configurable pass mark threshold
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

Open: the answers are graded by the keyword check only. Grading with the local model (as in STORY-0200) is not wired into the exam yet, and the results are not kept after the view closes.

Tests: `studyfrog-ui/tests/mock_exam_ui.rs` (a full answer earns all points, an unwritten exam none), and `setup/mock_exam/grading.rs` (grading, breakdown). A screenshot (`--screen mock-graded`) shows the result.

## Open Questions
None.
