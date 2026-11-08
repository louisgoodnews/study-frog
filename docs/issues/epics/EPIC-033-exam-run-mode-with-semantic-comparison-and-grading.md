---
id: EPIC-033
type: epic
title: Exam run mode with mandatory text entry, semantic comparison, and percentage grading
status: done
parents: []
labels: [m12, core, rehearsal, slm, service, ui]
acceptance_criteria:
  - "Rehearsal setup offers an Exam run mode that enforces strict one-way progression (no skipping, no moving forward without submitting an answer, and no backward navigation)"
  - "Every flashcard and question in an Exam run requires the user to enter a written answer that is scored automatically against the reference answer via semantic comparison"
  - "Completing an Exam run computes an overall percentage score and grade with a 60% threshold (D / 4.0) to pass and 5% grade steps up to 95%-100% for A+ / 1+ (very good)"
---

## Context
While standard rehearsal modes (`Flip cards` and `Write answers`) are designed for formative self-paced practice with manual rating overrides and skipping, learners preparing for university module exams need a realistic, summative **Exam run mode**. In an exam simulation, the learner must not be able to skip cards, peek ahead, or go back to revise earlier answers; every flashcard or question must be answered in writing and graded objectively via semantic comparison, culminating in a percentage score and a standardized university grade scale (`60%` = `D`/`4` to pass, `95%`+ = `A+`/`1+` very good).

## Description
Introduce an end-to-end **Exam Run Mode** across `studyfrog-core`, `studyfrog-rehearsal`, `studyfrog-slm`, `studyfrog-service`, and `studyfrog-ui`:
1. Exam Run Mode & Strict Navigation Rules:
   - Add `RunMode::Exam` (or exam configuration on rehearsal runs) selectable in the Rehearsal setup screen.
   - Prohibit skipping (`Skip`), moving forward without non-empty text submission, or navigating backward during an active exam run.
   - Defer answer reveal and grading feedback until the entire exam run is completed (or Lock each submitted item immediately upon submission without allowing self-rating overrides).
2. Mandatory Written Answer & Semantic Comparison:
   - Force written text input for both flashcards (`Doc::Card`, comparing against `back`) and questions (`Doc::Question`, comparing against `reference`, `accepted`, and `rubric`).
   - Evaluate each submitted answer using StudyFrog's semantic comparison engine (`studyfrog-slm` / similarity scoring) to produce a per-item score in `[0.0, 1.0]` (`0%`–`100%`).
3. Grading Scale (`60%` Pass Threshold & `5%` Step / Top `5%` for `A+` / `1+`):
   - Compute the aggregate exam percentage across all exam items and map it to a combined international (`A+`–`F`) and German (`1+`–`5`) grade scale using `5%` intervals:
     - `95% – 100%` (top `5%` band): `A+` / `1+` (*Sehr gut+*, very good)
     - `90% – <95%`: `A` / `1.0` (*Sehr gut*)
     - `85% – <90%`: `A-` / `1.3`
     - `80% – <85%`: `B+` / `1.7`
     - `75% – <80%`: `B` / `2.0` (*Gut*)
     - `70% – <75%`: `B-` / `2.3`
     - `65% – <70%`: `C` / `3.0` (*Befriedigend*)
     - `60% – <65%`: `D` / `4.0` (*Ausreichend* — minimum `60%` required to pass)
     - `< 60%`: `F` / `5.0` (*Nicht bestanden* — failed)
4. Exam Results View & Persistence:
   - Display a dedicated Exam Result summary banner with the final percentage, grade badge (`A+ / 1+` to `F / 5.0`), Pass/Fail status, and a scrollable item-by-item review comparing the user's submitted text, the reference answer, and the semantic similarity score.
   - Persist exam run results so they can be linked in the learning diary and reflected in stack familiarity metrics.

## Acceptance Criteria
- [x] Rehearsal setup offers an Exam run mode that enforces strict one-way progression (no skipping, no moving forward without submitting an answer, and no backward navigation)
- [x] Every flashcard and question in an Exam run requires the user to enter a written answer that is scored automatically against the reference answer via semantic comparison
- [x] Completing an Exam run computes an overall percentage score and grade with a 60% threshold (D / 4.0) to pass and 5% grade steps up to 95%-100% for A+ / 1+ (very good)

## Design Notes
Touchpoints across crates:
- `crates/studyfrog-core`: `RunMode::Exam` (or exam run metadata) and grade scale calculation (`exam_grade(percent: f32)`).
- `crates/studyfrog-rehearsal`: enforce non-skippable, forward-only step transitions and mandatory text submission in exam mode.
- `crates/studyfrog-slm` & `crates/studyfrog-service`: semantic comparison scoring for flashcards (`Card.back`) as well as questions (`Question.reference`), plus exam score aggregation.
- `crates/studyfrog-ui`: exam mode option in `view/setup.rs`, locked answer input without skip/back controls in `view/rehearse.rs`, and percentage + grade (`A+/1+` to `F/5`) summary in `view/summary.rs`.

## Progress (2026-11-08)
Split into STORY-0125 (spec, `docs/specs/exam-mode.md`), STORY-0126 (data model and engine rules), STORY-0127 (scoring, grading, history), STORY-0128 (screens) and STORY-0129 (help, control protocol), all done. Exam runs never change the review schedule (decided in the spec: an exam assesses, it does not schedule), so they are not part of familiarity; their results are stored (`exam_result`), listed in Reports > Runs and offered to the diary prompt. Verified by unit, property, fixture, harness and headless control tests and by screenshots of the setup, run and result screens (`--screen exam-setup|exam|exam-result`); not yet used by a person in an interactive window.

## Dependencies
EPIC-007 (Rehearsal Engine) and EPIC-013 (Comparison Scoring).

## Open Questions
None.
