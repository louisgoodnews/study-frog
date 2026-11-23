---
id: BUG-0038
type: bug
title: "Mock exam keyword grading scores correct answers 0 because the rubric points are generic text"
status: done
estimate: 3
labels: [m14, rehearse, agent]
acceptance_criteria:
  - "An answer that states the reference meaning in other words earns points on a question whose rubric point is generic text"
  - "A rubric point the keyword check cannot test is shown as 'not checked by word overlap' and does not count as missing"
  - "The result says which points were counted by word overlap and which were not checked"
---

## Context
Found while auditing STORY-0186 and STORY-0187 in the running app (2026-11-22), with no local model selected. The mock exam from the PDF "Vorlesung.pdf" asked "Explain the following in your own words: Gueltigkeit". The reference answer is "Gueltigkeit: misst, was er soll." with the rubric point "gives the meaning, not just the term". The student answered "Gueltigkeit bedeutet, dass der Test misst, was er soll.", which states the meaning. The result gave 0 of 2 points, marked "Missing: gives the meaning, not just the term".

The rubric point is the same generic sentence on every template question, so the keyword check has no words in it that a correct answer would share. The second question scored 0 of 4 for "Objektivitaet und Reliabilitaet" with the rubric "names one key point of the topic", which is also generic. The result says "Graded by word overlap ... a correct answer in other words can be marked missing" but gives no way to tell a wrong answer from a correct one.

Screenshot: `/tmp/claude-0/audit/g5/0187-result.png`.

## Description
For template questions, build the rubric points from the note's own content (the definition or formula the answer must give), or stop counting a generic rubric point as missing when the keyword check cannot test it. Show "not checked" for those points. Check with the PDF exam and the notes exam, answering the reference answers in other words.

## Acceptance Criteria
- [x] An answer that states the reference meaning in other words earns points on a question whose rubric point is generic text
- [x] A rubric point the keyword check cannot test is shown as 'not checked by word overlap' and does not count as missing
- [x] The result says which points were counted by word overlap and which were not checked

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/setup/mock_exam/grading.rs`, `studyfrog-rehearsal` (rubric construction in `curriculum_build`), `rubric_eval::fallback_evaluate`.
- Guardrails: P1, P6.

## Dependencies
- STORY-0186
- STORY-0187

## Open Questions
- None.

Review (2026-11-22): still open in the running app. The answer "Gueltigkeit bedeutet, dass der Test misst, was er soll." (the reference meaning in other words) gets 0 of 2 with the rubric "gives the meaning, not just the term" shown as Missing; the same answer typed as "Gueltigkeit: misst, was er soll." also gets 0; "Mittlerer Wert der sortierten Reihe" gets 0 of 4 on "names one key point of the topic" (`/tmp/claude-0/review-e/10-result.png`, `13-result2.png`). No line says "not checked by word overlap". Status stays backlog.

## Fix (2026-11-23)

Cause: the template rubric points were generic lines ("gives the meaning, not just the term", "names one key point of the topic", "names the formula"). None of their words is in a correct answer, so the keyword check (`rubric_eval::fallback_evaluate`) scored every such answer 0.

Changes:
- `studyfrog-rehearsal/src/curriculum_templates.rs` (moved out of `curriculum_build.rs`): a recall point is the meaning from the note ("misst, was er soll"); a case point is the note's first sentence; a calculation has two points, the quantity it names and the expression it gives. The calculation prompt no longer prints the formula.
- `studyfrog-agent/src/rubric_eval.rs`: `CriterionResult.checked`. A point with no word of four letters or more is not checked: it is neither met nor missing, and the feedback says "Not checked by word overlap". The overall score of a case counts checked points only.
- `studyfrog-agent/src/exam_grade.rs`: the share counts checked points; `is_counted` tells whether any point was checked.
- `studyfrog-ui/src/setup/mock_exam/grading.rs`: a question with no checked point is not counted and is left out of the total; each point is met, missing or not checked.
- `studyfrog-ui/src/view/mock_exam_view/result.rs` and `view/run_card/parts.rs`: "Not checked by word overlap" pills, "not counted" on the question, and a line that says how many questions were left out.

Tests: `studyfrog-rehearsal/tests/curriculum_build.rs` (`a_recall_rubric_point_is_the_meaning_from_the_notes`, `a_calculation_is_asked_for_by_its_quantity_and_checked_by_its_expression`, `a_case_rubric_point_is_a_sentence_of_the_notes`); `studyfrog-agent/tests/rubric_eval.rs` (`a_point_with_no_word_to_match_is_not_checked_and_not_counted_as_missing`); `studyfrog-agent/tests/exam_grade.rs` (`unchecked_points_leave_the_share_to_the_checked_ones`); `studyfrog-service/tests/exam_defects.rs`.

Verified in the running app (fixed build, 2026-11-23, Statistik stack, no model):
- The answer "Gueltigkeit bedeutet, dass der Test misst, was er soll." scores 2 of 2 with the point "misst, was er soll" made (`/tmp/claude-0/exam/a-notes-04-result.png`). Before the fix it scored 0 of 2 with "gives the meaning, not just the term" (`/tmp/claude-0/exam/b-notes-04-result.png`).
- The formula "2 + 2 = 4" has no checkable point: the question reads "(not counted, ...)" and each point reads "Not checked by word overlap" (`/tmp/claude-0/exam/a2-result-low.png`).

Limit, kept in the result's caveat: the keyword check still cannot recognise a paraphrase that shares no words with the note. The model grading of STORY-0187 covers that when a local model is selected; no model is installed here, so it was not measured.
