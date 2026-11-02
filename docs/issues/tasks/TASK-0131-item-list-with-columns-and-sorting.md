---
id: TASK-0131
type: task
title: Item list with columns and sorting
status: done
epic: EPIC-023
parents: [STORY-0057]
estimate: 5
labels: [m10, ui]
acceptance_criteria:
  - "The list shows the columns of the spec, sorted by any column, with the match count"
  - "10k rows scroll inside the frame budget, measured"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Build the virtualised table.

## Acceptance Criteria
- [x] The list shows the columns of the spec, sorted by any column, with the match count
- [x] 10k rows scroll inside the frame budget, measured

## Design Notes
Rows come from `studyfrog-service::library_rows` (every item with its stack chain, subjects, teachers, tags, effective status, priority and difficulty, familiarity and dates; migration 18 keeps each item's text for searching and marks every file so it is indexed again once). The list is `view/library_table.rs` on `egui_extras::TableBuilder`, which builds only the rows in view; `library_ui.rs` keeps the rows, the columns (stored in `app.toml` as `library_columns`), the sort (a click sorts, a second click reverses, shift adds a key, three at most) and what the stack and the search let through. The search is still the title search; the Standard and Query modes are TASK-0132. Measured: 10 000 rows draw in 11 ms per frame in a debug build, and only the visible rows are built. The old overview of stacks is under the tree entry 'Overview'; 'All items' is the new default. A bug found on the way: the app font has no arrow glyphs, so a sort marker made of one would crash the window; the markers are `^` and `v`.

## Dependencies
The specification task of this story.

## Open Questions
None.
