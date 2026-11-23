---
id: BUG-0055
type: bug
title: "Mock exam question counts do not follow the 40 / 30 / 30 plan when the notes are short"
status: done
epic: EPIC-063
parents: ['EPIC-063']
estimate: 2
labels: [m14, ui, rehearse]
acceptance_criteria:
  - "The mock exam's question counts follow the 40 / 30 / 30 plan (recall, case scenarios, calculations) whenever the notes hold enough material for each part"
  - "When a part has too little material, the exam says so, rather than silently giving fewer questions"
---

## Context
Split from BUG-0053 at its sign-off (2026-11-23). The reproduction showed that a full note set gives 7 questions (with Part C), and a damaged set gives 6, but the 40 / 30 / 30 plan in STORY-0186 is not kept: the counts range from 3 to 7 of 10, depending on the notes. Evidence: `/tmp/claude-0/bug53/`.

## Description
Make the question counts per part follow the plan, and make a shortfall visible.

## Acceptance Criteria

- [x] The mock exam's question counts follow the 40 / 30 / 30 plan (recall, case scenarios, calculations) whenever the notes hold enough material for each part
- [x] When a part has too little material, the exam says so, rather than silently giving fewer questions

## Design Notes
Read the part planner in the mock exam generator (grep "40" and "Part" in crates/studyfrog-service and crates/studyfrog-rehearsal). Keep the template path working without a model.

## Dependencies
- BUG-0053
- STORY-0186

## Open Questions
- None.

## Fix note (2026-11-23): fixed, verified

The 40 / 30 / 30 split itself (`curriculum_build::plan`, 4 / 3 / 3 of 10) was right. The shortfall came from the material: templates made one case per chunk (so a section gave at most one case question), and recall and calculation questions only where the notes had a definition or a formula line. The rule now:

- Each part asks for its plan share. A part gets what the notes hold, up to that share (`curriculum_templates::material` gives the capacity: definitions for recall, paragraphs for cases, formulas for calculations). No part is padded from another.
- A part short of its share is named: `curriculum_build::shortfalls` and `short_note` say how many it has, and what the notes lack. `studyfrog-service::exam_gen::Generated.short` carries them, and the exam preview shows one line per short part.
- The template path needs no model. The model path is unchanged; it was not run (no model in this container).

Tests: `studyfrog-rehearsal/tests/curriculum_build.rs` (full material gives 4 / 3 / 3 with no shortfall; partial material gives what it holds and names each short part; empty material makes no questions; the parts add up to the total for totals 3 to 20), and `studyfrog-service/src/exam_gen.rs` (`a_part_the_notes_cannot_fill_is_named_and_the_rest_follows_the_plan`).

Running app (1280x800, the Statistik stack ticked, no model, "Mock exam" on the rehearse setup): the preview names the two short parts, "Part A (conceptual recall) has 2 of the 4 questions the plan asks for" and "Part C (calculations) has 2 of the 3 ...", and Part B has its 3 (`/tmp/claude-0/b55/m1-exam-after.png`). Before: the same stack gave 7 questions with no note (`/tmp/claude-0/bug53/h2-mock.png`, BUG-0053 evidence). The count is the same for this vault because its notes hold only two definitions and two formulas; the difference is that the shortfall is now said. The exam with more material gives the full 4 / 3 / 3 (the unit test).
