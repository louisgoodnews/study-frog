---
id: BUG-0052
type: bug
title: "Mock exam reference answer for a card question starts with the card's question text"
status: done
estimate: 1
labels: [m14, rehearse, agent]
epic: EPIC-063
parents: ['EPIC-063']
acceptance_criteria:
  - "The model answer shown for a question made from a flashcard gives the card's answer only, without the card's question text in front of it"
  - "The rubric line for that question is not the card's question text"
---

## Context
Found while re-checking STORY-0187 criterion 3 in the running app (2026-11-22, no local model). The notes stack "Statistik" holds two cards, "Mittelwert" (Was ist der Mittelwert? / Summe durch Anzahl) and "Median" (Was ist der Median? / Mittlerer Wert der sortierten Reihe). In the result view of the mock exam, the model answer for question 4 reads "Was ist der Mittelwert? Summe durch Anzahl" (second line of the model answer), and the one for question 3 reads "Was ist der Median? Mittlerer Wert der sortierten Reihe". The reference answer should be the answer alone.

Screenshot: `/tmp/claude-0/review-e/16-bottom.png` (questions 3 and 4 of the result view).

## Description
The reference answer for a card-based question is built from the card's front and back. Use the back (the answer) only, or build the reference from the note's own content, and do not repeat the question. Check with the Statistik stack: the result view's model answer for each card question shows the answer alone.

## Acceptance Criteria
- [x] The model answer shown for a question made from a flashcard gives the card's answer only, without the card's question text in front of it
- [x] The rubric line for that question is not the card's question text

## Design Notes
- Touchpoints: `crates/studyfrog-rehearsal/src/curriculum_build.rs` (reference answers and rubrics), `crates/studyfrog-ui/src/setup/mock_exam/grading.rs` (the review lines).
- Guardrails: P1, P6 (≤ 250 lines per file).

## Dependencies
- STORY-0187
- BUG-0038 (the keyword rubric of template questions)

## Open Questions
- None.

## Fix (2026-11-23)

Cause: `notes_chunks` (`studyfrog-service/src/exam_save.rs`) built each item's text from its title, its prompt and its answer. For a card or question the prompt is the question, so the case template's reference answer (the chunk's text) began with the question. A note's text also began with its title.

Change: a note contributes its body, and a card or question its answer (its prompt only when it has no answer). The reference answer is then the answer alone. Side effect: when a model writes the questions, it sees a card's or question's answer without its prompt; the topic still names it.

Tests: `studyfrog-service/tests/exam_defects.rs` `a_cards_reference_answer_is_its_answer_alone` (fails on the old code: the key was "Was ist der Mittelwert?\nSumme durch Anzahl"); `studyfrog-rehearsal/tests/curriculum_build.rs` `a_case_rubric_point_is_a_sentence_of_the_notes`.

Verified in the running app (fixed build, 2026-11-23): the model answers read "Mittlerer Wert der sortierten Reihe" and "Summe durch Anzahl" (`/tmp/claude-0/exam/a-notes-04-result.png`). Before the fix: "Was ist der Median? Mittlerer Wert der sortierten Reihe" (`/tmp/claude-0/exam/b-notes-04-result.png`).
