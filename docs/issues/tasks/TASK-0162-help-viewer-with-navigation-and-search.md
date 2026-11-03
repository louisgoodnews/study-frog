---
id: TASK-0162
type: task
title: Help viewer with navigation and search
status: done
epic: EPIC-023
parents: [STORY-0066]
estimate: 5
labels: [m10, ui]
acceptance_criteria:
  - "The help area shows a page tree, renders the pages with links between them and searches titles and text"
  - "The question mark in the bar and a help button on every screen open the right page"
  - "Pages are compiled into the binary and a check fails on a broken link"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the screen, the page registry as plain data and the link check script.

## Acceptance Criteria
- [x] The help area shows a page tree, renders the pages with links between them and searches titles and text
- [x] The question mark in the bar and a help button on every screen open the right page
- [x] Pages are compiled into the binary and a check fails on a broken link

## Design Notes
The page table is plain data in `studyfrog-service` (`help.rs`, `help_pages.rs`, `include_str!` of `docs/help/`). Help opens from the `?` in the top bar, the Help button of the status bar, `F1` and the sidebar, at the page of the screen showing. The link check is a test (`crates/studyfrog-service/tests/help.rs`) run by `scripts/check.sh`, not a separate script. The search is an in-memory substring search over titles and text rather than the vault's SQL search: the help pages are not in the index, and 26 pages plus the user's are searched in well under a millisecond. Build vs buy: no crate needed.

## Dependencies
The planning task of this story.

## Open Questions
None.
