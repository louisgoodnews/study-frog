---
id: TASK-0130
type: task
title: "Search query parser and evaluator"
status: done
epic: EPIC-023
parents: [STORY-0057]
estimate: 5
labels: [m10, core, store]
acceptance_criteria:
  - "A pure parser turns a query into a syntax tree and reports errors with a position and the expected token"
  - "The evaluator runs the tree in memory over typed rows, so no user text reaches SQL; a test shows hostile text is only ever a value"
  - "Fuzzing finds no panic, and 10k items filter within the budget in the spec, measured"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Implement the grammar from the spec as a hand-written parser and an evaluator over the index.

## Acceptance Criteria
- [x] A pure parser turns a query into a syntax tree and reports errors with a position and the expected token
- [x] The evaluator runs the tree in memory over typed rows, so no user text reaches SQL; a test shows hostile text is only ever a value
- [x] Fuzzing finds no panic, and 10k items filter within the budget in the spec, measured

## Design Notes
In `studyfrog-core`: `query` (the tree and errors), `query_lex`, `query_cursor`, `query_cond`, `query_parse`, `query_fields`, `query_values` (the parser), `query_print` (a query as text again, for the Standard mode and saved filters) and `query_eval` (`Row`, `matches`, `run`). The evaluator works on rows in memory, which is simpler and safer than building SQL and fast enough (2 to 18 ms for 10 000 items in a debug build); the rows are loaded from the index by TASK-0131. Errors are those of the specification, corrected for the column rule (a missing thing is reported one column after the last token).

## Dependencies
The specification task of this story.

## Open Questions
None.
