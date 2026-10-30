---
id: TASK-0101
type: task
title: Home page
status: done
epic: EPIC-020
parents: [STORY-0038]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "The summary counts ready, due and new items for today, the streak, reviews today, stacks waiting, planned sessions and recent items and runs"
  - "Start today's rehearsal and Rehearse per stack begin a run or open its setup"
  - "A vault opens on the home page"
---

## Context
The owner asked for a consistent look, a home page, a clearer library and a better rehearsal screen.

## Description
Home page.

## Acceptance Criteria
- [x] The summary counts ready, due and new items for today, the streak, reviews today, stacks waiting, planned sessions and recent items and runs
- [x] Start today's rehearsal and Rehearse per stack begin a run or open its setup
- [x] A vault opens on the home page

## Design Notes
`service::home`, `store::home`, `home_ui.rs`, `view/home.rs`. The summary reloads when the home or library screen is opened.

## Dependencies
None.

## Open Questions
None.
