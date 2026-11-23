---
id: BUG-0055
type: bug
title: "Mock exam question counts do not follow the 40 / 30 / 30 plan when the notes are short"
status: backlog
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

- The mock exam's question counts follow the 40 / 30 / 30 plan (recall, case scenarios, calculations) whenever the notes hold enough material for each part
- When a part has too little material, the exam says so, rather than silently giving fewer questions

## Design Notes
Read the part planner in the mock exam generator (grep "40" and "Part" in crates/studyfrog-service and crates/studyfrog-rehearsal). Keep the template path working without a model.

## Dependencies
- BUG-0053
- STORY-0186

## Open Questions
- None.
