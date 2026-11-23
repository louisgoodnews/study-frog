---
id: BUG-0053
type: bug
title: "Mock exam from notes with a formula line gives no calculation question in the template path"
status: review
estimate: 2
labels: [m14, rehearse, agent]
epic: EPIC-063
parents: ['EPIC-063']
acceptance_criteria:
  - "A note with a formula line (a line with '=' and a digit) makes a calculation question in the mock exam when no model is selected"
  - "The mock exam has a 'Part C: calculations' section when the notes hold formulas, and the question counts follow the 40 / 30 / 30 plan"
---

## Context
Found while checking EPIC-063 criterion 3 and STORY-0186 criterion 1 in the running app (2026-11-22, no local model). The notes stack "Statistik" holds the note "Formel" with the line "Standardfehler = s / Wurzel(n) = 2 / 2 = 1". The mock exam from the notes made "4 questions, 42 minutes, 14 points" with a Part A (recall) and a Part B (case scenarios). "Formel" was used as the term of a case question ("A colleague applies the ideas of Formel in a new study"), and no Part C (calculations) appeared.

Screenshot: `/tmp/claude-0/review-e/20-mock-stat.png`.

## Description
The template path should turn a formula line in the notes into a calculation question with its reference solution and point scheme, as STORY-0186 describes. Find out where the formula line is lost (the chunking, the balance step or the template choice) and fix it. Check with the Statistik stack: the exam should have a calculation part.

## Acceptance Criteria
- [x] A note with a formula line (a line with '=' and a digit) makes a calculation question in the mock exam when no model is selected
- [ ] The mock exam has a 'Part C: calculations' section when the notes hold formulas, and the question counts follow the 40 / 30 / 30 plan

## Design Notes
- Touchpoints: `crates/studyfrog-rehearsal/src/curriculum.rs` (formula detection), `crates/studyfrog-rehearsal/src/curriculum_build.rs` (balance and templates).
- Guardrails: P1, P6 (≤ 250 lines per file).

## Dependencies
- STORY-0186
- EPIC-063

## Open Questions
- Whether the formula line is kept when it sits in a note whose title is a term (the case above), or only in a note of its own.

## Triage (2026-11-23): not reproduced as filed; stays backlog

- The vault that showed the defect had a damaged note: `formel.md` held only `'Standardfehler` (the rest of the line was lost when the fixture was written), so the note had no formula line. Its calculation question could not exist.
- With the complete line ("Standardfehler = s / Wurzel(n) = 2 / 2 = 1") the pre-fix binary already gives a "Part C: calculations" section, in the running app (`/tmp/claude-0/exam/b02-mock-notes.png`, Part C present). The template path is therefore not losing formula lines.
- Changed on the way (BUG-0038 and BUG-0052 changes touch the same code): the calculation question no longer prints the formula, and its rubric points come from the formula's sides. Tests: `studyfrog-service/tests/exam_defects.rs` `a_formula_note_gives_a_calculation_part_and_the_prompt_does_not_print_the_formula`, `studyfrog-rehearsal/tests/curriculum_build.rs` `a_calculation_is_asked_for_by_its_quantity_and_checked_by_its_expression`.
- Not met: acceptance criterion 2 ("the question counts follow the 40 / 30 / 30 plan"). The template path makes at most one question per definition and per chunk for cases, so a short note set gives a shorter exam (the preview says so: "The notes supplied 6 of the 10 questions asked for"). The counts follow the plan only when the notes hold enough items. A decision is needed on whether the exam should be padded or the criterion reworded.
- Next step: the reporter should confirm with the full vault, and the criterion decision is open.

## Verification (2026-11-23): not reproduced with a healthy vault

Vault: stack Statistik with the note "Formel" in full ("Standardfehler = s / Wurzel(n) = 2 / 2 = 1"), the note "Normalverteilung" (sentences and "z = (x - Mittelwert) / s = 6 / 3 = 2") and two cards. No model selected. The app is this branch's build.

- Criterion 1 (a formula note makes a calculation question with no model): MET. The mock exam from the notes has calculation questions 6 (from "Formel", reference "Standardfehler = s / Wurzel(n) = 2 / 2 = 1") and 7 (from "Normalverteilung") under "Part C: calculations" (`/tmp/claude-0/g3/07-mock-exam-preview.png`).
- Criterion 2 (Part C present, counts follow 40 / 30 / 30): Part C present, so that half holds. The counts do not follow the plan: 7 questions (2 recall, 3 case, 2 calculation), because the notes supplied 7 of the 10 asked for. The decision on padding the exam or rewording the criterion is still open (see Triage above).
- The reported symptom came from the reporter's truncated note (`formel.md` holding only `'Standardfehler`), not from the template path. The reporter must confirm with the full vault. The bug stays open in the backlog; no code changed for it.
- Observed, not filed: a sentence with "=" counts as a formula. Question 7 asks for the formula of "Der z-Wert misst den Abstand eines Werts vom Mittelwert in Standardabweichungen: z" and its model answer is the whole sentence with "z = (x - Mittelwert) / s = 6 / 3 = 2". A quantity name should not be a sentence. Candidate for a follow-up under STORY-0186.

## Fix and verification (2026-11-23): cause confirmed, in review

No code changed. The report came from a note that was cut short, not from the template path.

Reproduction in the running app (this branch's build), no model selected, the stack Statistik ticked under Stacks, "Mock exam" on the rehearse setup:

- Healthy stack (`formel.md` complete, plus Mittelwert, Median and Normalverteilung): 7 questions, Part C present, with question 6 "Give the formula for Standardfehler and explain its symbols" from Formel and question 7 from Normalverteilung (`/tmp/claude-0/bug53/h2-mock.png`).
- Damaged stack (`formel.md` body only `'Standardfehler`, otherwise the same): 6 questions, Part C present, but only question 6 from Normalverteilung; the Formel calculation is gone (`/tmp/claude-0/bug53/d2-mock.png`).
- Isolated: Formel as the only formula note (no Normalverteilung), complete note: 4 questions, Part C present with "4. (4 points, Formel) Give the formula for Standardfehler" (`/tmp/claude-0/bug53/s1-solo-healthy.png`).
- Isolated, damaged note: 3 questions, Part B only, no Part C at all, which matches the report (`/tmp/claude-0/bug53/s3-solo-damaged.png`).

A note without a formula line makes no calculation question, so the cut note is correct behaviour; the complete note gives one. Criterion 1 is met. Criterion 2's Part C half is met. Its count half (40 / 30 / 30) is not: the notes supply 3 to 7 of the 10 questions asked for, and that is an open decision recorded in the triage above (pad the exam or reword the criterion), so the criterion stays unticked. The bug stays in review, not done, because the issue rules do not allow `done` with an unchecked criterion: the reporter must confirm the cause with the full vault, and the owner must decide the count clause. If the full vault shows a different result, reopen this bug.
