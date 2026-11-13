---
id: EPIC-023
type: epic
title: "Usability and insight for 0.3.0"
status: done
parents: []
labels: [m10, ui]
acceptance_criteria:
  - "The library, stack view, settings and notification center are redesigned for easier use"
  - "Items have statuses and a familiarity value, and StudyFrog suggests what to focus on and shows the next reviews on the calendar, movable by the user"
  - "Data can be exported and imported in open formats (Anki and Obsidian first), and a built-in wiki, FAQ and tutorials explain StudyFrog"
  - "A stack opens as an overview with sub-views apart from the existing detail page"
  - "A rehearsal can run against a countdown or count up"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Make StudyFrog easier to use and let it tell the user what to study.

## Acceptance Criteria
- [x] The library, stack view, settings and notification center are redesigned for easier use
- [x] Items have statuses and a familiarity value, and StudyFrog suggests what to focus on and shows the next reviews on the calendar, movable by the user
- [x] Data can be exported and imported in open formats (Anki and Obsidian first), and a built-in wiki, FAQ and tutorials explain StudyFrog
- [x] A stack opens as an overview with sub-views apart from the existing detail page
- [x] A rehearsal can run against a countdown or count up

## Design Notes
Stories STORY-0055, STORY-0056, STORY-0057, STORY-0058, STORY-0059, STORY-0060, STORY-0061, STORY-0062, STORY-0063, STORY-0064, STORY-0065, STORY-0066, STORY-0067, STORY-0070, STORY-0071. Release target: 0.3.0. STORY-0068 (diary) moved to EPIC-024 for 0.4.0 on 2026-11-03, when the owner chose to release 0.3.0 without it. The Jira screenshots of the owner are the reference for the library list and the stack view.

## Dependencies
None.

## Open Questions
Build order proposed: statuses and settings tabs first (they unblock the rest), then notifications and the vault name (independent, small), familiarity, the library list, the calendar planner, the stack page and its sub-views, import and export and help in parallel.
