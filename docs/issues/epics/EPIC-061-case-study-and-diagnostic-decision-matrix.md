---
id: EPIC-061
type: epic
title: "Scenario-Based Case Studies, Diagnostic Decision Matrices, and Semantic Verification"
status: review
parents: []
labels: [m14, rehearse, agent, ui]
acceptance_criteria:
  - 'Specialized Case Study item format supporting rich scenario vignettes, decision inputs, and justification criteria'
  - 'Multi-criteria local SLM evaluator scoring both categorical decisions (e.g. test selection) and conceptual rationales'
  - 'Structured rehearsal UI rendering case scenarios, dual-input fields, and criterion-by-criterion semantic feedback'
  - 'Seamless integration with existing stacks, spaced repetition scheduling, and exam runs'
---

## Context
In higher university education—particularly fields like Psychological Assessment (Diagnostik & Evaluation), Clinical Psychology, Medicine, and Law—rote memorization of definitions is insufficient. Students must evaluate complex real-world case scenarios, select appropriate diagnostic procedures (e.g. cognitive ability testing vs. personality inventories under social desirability risk), and justify their decisions against scientific quality criteria (reliability, validity, norms).

## Description
Deliver an advanced scenario-based learning subsystem:
1. **Case Study Item Structure**: Extend StudyFrog's item model with a scenario vignette, target decision category, reference reasoning points, and critical boundary conditions (e.g. fabbing risk, test economy).
2. **Multi-Criteria SLM Grading**: Utilize the local SLM to independently verify:
   - Whether the chosen diagnostic method or procedure matches clinical/psychometric guidelines.
   - Whether the student's free-text justification correctly identifies critical test criteria and constraints.
3. **Interactive Decision Matrix UI**: A focused active-recall interface presenting the case study, input fields for choice and justification, and instant structured feedback highlighting matched points vs overlooked caveats.
4. **Integration**: Treat case studies as first-class items scheduled with FSRS and supported in mock exams.

## Acceptance Criteria
- [x] Specialized Case Study item format supporting rich scenario vignettes, decision inputs, and justification criteria
- [x] Multi-criteria local SLM evaluator scoring both categorical decisions (e.g. test selection) and conceptual rationales
- [x] Structured rehearsal UI rendering case scenarios, dual-input fields, and criterion-by-criterion semantic feedback
- [ ] Seamless integration with existing stacks, spaced repetition scheduling, and exam runs

## Design Notes
Implement domain types in `studyfrog-core` / `studyfrog-rehearsal`. Implement multi-criteria SLM prompting and parsing in `studyfrog-agent` (using local Ollama/candle). Ensure all functions remain pure free functions (Invariant P1) and files stay $\le 250$ LOC (Invariant P6).

## Dependencies
None.

Audit (2026-11-22): criteria 1 to 3 MET in the running app (`/tmp/claude-0/audit/g3/02-case-list.png`, `/tmp/claude-0/audit/g3/08-model-good.png`, `/tmp/claude-0/audit/g3/03-case-checked.png`, `/tmp/claude-0/audit/g3/15-run-case-checked.png`); the model path was checked with a local stub, not a real model. Criterion 4 is NOT MET in part: stacks and scheduling work (a case in a run is scheduled from its rating, `/tmp/claude-0/audit/g3/17-item-case.png`; a case-only stack is in the exam queue, `/tmp/claude-0/audit/g3/19-exam2-case.png`), but in an exam run the case shows only its vignette and a free-text box, without its decision prompt or options (`/tmp/claude-0/audit/g3/21-exam-case.png`). Filed BUG-0022. The epic stays in review.

## Open Questions
None.

## Verification (2026-11-22)

- Criterion 4 remains unticked: NOT MET in part. Stacks and scheduling work. In an exam run the case shows only its vignette, with no decision prompt or options (BUG-0022 reproduced, `/tmp/claude-0/verifyA/24-exam-run-case.png`).
- Status stays review.

Review (2026-11-22): criterion 4 re-checked in the running app (`/tmp/claude-0/review-d`, vault with stack `diag`: the case "Klinik-Screening" and three plain questions). Criteria 1 to 3 were not re-run in this round; they stay as ticked above.
- Stacks: MET. The case is in the stack's rehearse set (`rehearse setup` shows "4 items" for the stack) and in the library (`library.rows`, key QUESTION_1, kind question) (`/tmp/claude-0/review-d/09-library.png`).
- Spaced repetition: MET. A spaced run (`--preset spaced --mode answer`, effect `update_schedule`) rated the case "hard" after the answer "Short screen". The vault index now holds a `review` row for the case (rating 3) and a `mem_state` row with a due date about 2 days later (`/tmp/claude-0/review-d/11-spaced-after.png`). Checked with a copy of `.studyfrog/index.sqlite`.
- Exam runs: NOT MET. The exam run shows the vignette and "Your answer" only, with no decision prompt or options (BUG-0022 still open, `/tmp/claude-0/review-d/05-exam-case.png`). An exam run is also practice_only in the schedule (correct by design: "an exam never changes your review schedule").
- Criterion 4 stays unticked; the epic stays in review.
