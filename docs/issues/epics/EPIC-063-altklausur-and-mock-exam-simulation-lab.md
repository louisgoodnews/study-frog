---
id: EPIC-063
type: epic
title: "Altklausur Lab: Simulated Mock Exams, Slide Extraction, and University Grade Scale"
status: review
parents: []
labels: [m14, rehearse, agent, ui]
acceptance_criteria:
  - 'Comprehensive mock exam framework with configurable question points, total duration, and strict test rules'
  - 'Automated exam generation from lecture slides and course scripts using local SLM prompting'
  - 'Multi-format question mix combining free-text conceptual recall, case scenarios, and calculation problems'
  - 'Detailed exam grading report mapping achieved percentages to the German university grading scale (1.0 to 5.0)'
---

## Context
The user's original learning breakthrough came from creating an interactive "Altklausur" testing lab where past exam questions were answered under real test conditions and automatically graded. University courses rarely provide unlimited past exams. Transforming course materials into full, timed mock exams with automated SLM grading and German university grade conversion (1.0 to 5.0) brings this proven methodology directly into StudyFrog.

## Description
Develop a comprehensive Altklausur / Mock Exam Simulation Lab:
1. **Mock Exam Session Engine**: Timed exam runs with point budgets per question (e.g. 2-point definition, 5-point calculation, 8-point case study), strict countdown timers, and locked answer submissions.
2. **AI-Powered Exam Generation**: Parse lecture slide PDFs or Markdown notes and generate a balanced exam paper containing conceptual questions, calculation drills, and application vignettes.
3. **Automated Hybrid Grading**: Evaluate student submissions using the local SLM for semantic conceptual answers, formula validators for calculations, and exact match for choices.
4. **University Grade Conversion & Analytics**: Compute aggregate scores, assign a university grade according to standard distribution tables (1.0, 1.3, 1.7, ..., 4.0, 5.0), and highlight knowledge deficits per module chapter.

## Acceptance Criteria
- [ ] Comprehensive mock exam framework with configurable question points, total duration, and strict test rules
- [ ] Automated exam generation from lecture slides and course scripts using local SLM prompting
- [ ] Multi-format question mix combining free-text conceptual recall, case scenarios, and calculation problems
- [x] Detailed exam grading report mapping achieved percentages to the German university grading scale (1.0 to 5.0)

## Design Notes
Build session management in `studyfrog-rehearsal::mock_exam`. Use `studyfrog-agent` for SLM exam generation and grading. Adhere strictly to Invariant P1 (pure free functions) and Invariant P6 ($\le 250$ LOC/file).

## Dependencies
None.

## Open Questions
None.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): no local model is installed; model criteria are NOT CHECKABLE and the no-model message and keyword fallback were checked.
Criterion 1 NOT MET: the exam has configurable points only through the 3-minutes-a-point rule, shows "48 minutes" as a total, and has no countdown timer and no locked-answer rule in the exam view (/tmp/claude-0/audit/g5/0187-started.png). Left unticked.
Criterion 2 NOT CHECKABLE: exam generation from a lecture PDF and notes works offline from templates (/tmp/claude-0/audit/g5/0186-pdf-exam.png, 0186-mock-notes3.png); the local SLM prompting is not run (no model).
Criterion 3 NOT CHECKABLE: recall and case sections appear (/tmp/claude-0/audit/g5/0186-mock-notes3.png); a calculation section was not generated in the test (the formula note was added after the exam).
Criterion 4 MET: the result view shows total points, percentage, grade and word, pass or fail, points by topic, and each question's answer, model answer and rubric points (/tmp/claude-0/audit/g5/0187-result.png); the German band table is in `studyfrog-rehearsal::grade`. Defects: BUG-0038, BUG-0039, BUG-0040, BUG-0041.

## Verification (2026-11-22)

- Criterion 1 NOT MET: the exam view has no countdown. The preview says "48 minutes" (`/tmp/claude-0/verifyA/20-mock-preview.png`). The running exam shows an elapsed clock ("0.1 s" in the run view) and no timer, and answers stay editable until "Finish and grade" (`/tmp/claude-0/verifyA/21-exam-started.png`). The timer module `studyfrog-rehearsal::mock_timer` is not used by the UI. Filed BUG-0045. Left unticked.
- Criterion 2 NOT CHECKABLE: no local model is installed, so the model stage did not run. The template path runs offline, with the no-model note (`/tmp/claude-0/verifyA/20-mock-preview.png`).
- Criterion 3 NOT CHECKABLE: the mix follows the notes. This run gave case scenarios and one calculation, with no recall section (`/tmp/claude-0/verifyA/20-mock-preview.png`); the earlier audit run had recall (`/tmp/claude-0/audit/g5/0186-mock-notes3.png`). A model-driven mix cannot be checked.
- Criterion 4 not re-checked (already ticked). The no-model keyword grading ran: 0 of 16 points, "Graded by word overlap" note (`/tmp/claude-0/verifyA/23-exam-graded.png`). Typed answers could not be entered in this environment, so grading of real answers was not checked.
- Status stays review.

Review (2026-11-22): checked in the running app (no local model), `/tmp/claude-0/review-e/`.
- Criterion 1 (configurable points, duration, strict test rules): NOT MET. The exam view shows the points per question and a total of "42 minutes" (3 minutes a point, `/tmp/claude-0/review-e/07-notes-exam.png`), but no countdown, no warning and no automatic submission, and answers stay editable until "Finish and grade" (`/tmp/claude-0/review-e/08-started.png`, `21-960-preview.png`; BUG-0045 still open). No setting for points or duration in the UI. Left unticked.
- Criterion 2 (exam generation from slides and notes by the local SLM): NOT CHECKABLE for model quality; the no-model path is shown (`/tmp/claude-0/review-e/03-pdf-exam.png`).
- Criterion 3 (multi-format mix: recall, case, calculation): NOT CHECKABLE for the model mix. The template mix in the app is recall and case questions; no calculation part appeared with a formula note (`/tmp/claude-0/review-e/20-mock-stat.png`; BUG-0053). Not ticked.
- Criterion 4 (grading report and German scale): MET (`/tmp/claude-0/review-e/10-result.png`, `11-result-scroll.png`; the band table is checked in STORY-0187). Tick left as set.
- Children: STORY-0186 and STORY-0187 stay in review; BUG-0038, BUG-0039, BUG-0045 stay in backlog; BUG-0040 is done (verified, `/tmp/claude-0/review-e/19-mock-pair.png`). New: BUG-0052, BUG-0053.
- Status stays review.

