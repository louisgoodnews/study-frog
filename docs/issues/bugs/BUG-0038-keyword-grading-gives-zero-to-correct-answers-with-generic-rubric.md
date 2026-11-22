---
id: BUG-0038
type: bug
title: "Mock exam keyword grading scores correct answers 0 because the rubric points are generic text"
status: backlog
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
- [ ] An answer that states the reference meaning in other words earns points on a question whose rubric point is generic text
- [ ] A rubric point the keyword check cannot test is shown as 'not checked by word overlap' and does not count as missing
- [ ] The result says which points were counted by word overlap and which were not checked

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/setup/mock_exam/grading.rs`, `studyfrog-rehearsal` (rubric construction in `curriculum_build`), `rubric_eval::fallback_evaluate`.
- Guardrails: P1, P6.

## Dependencies
- STORY-0186
- STORY-0187

## Open Questions
- None.
