---
id: BUG-0053
type: bug
title: "Mock exam from notes with a formula line gives no calculation question in the template path"
status: backlog
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
- [ ] A note with a formula line (a line with '=' and a digit) makes a calculation question in the mock exam when no model is selected
- [ ] The mock exam has a 'Part C: calculations' section when the notes hold formulas, and the question counts follow the 40 / 30 / 30 plan

## Design Notes
- Touchpoints: `crates/studyfrog-rehearsal/src/curriculum.rs` (formula detection), `crates/studyfrog-rehearsal/src/curriculum_build.rs` (balance and templates).
- Guardrails: P1, P6 (≤ 250 lines per file).

## Dependencies
- STORY-0186
- EPIC-063

## Open Questions
- Whether the formula line is kept when it sits in a note whose title is a term (the case above), or only in a note of its own.
