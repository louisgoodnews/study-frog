---
id: STORY-0056
type: story
title: Current vault in the top bar
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 2
labels: [m10, ui]
acceptance_criteria:
  - "The top bar shows the name of the open vault next to the StudyFrog name"
  - "A long name is shortened with an ellipsis and the full name and path are in a tooltip"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Show which vault is open, because several vaults and profiles can be used and the bar does not say which one is active.

## Acceptance Criteria
- [ ] The top bar shows the name of the open vault next to the StudyFrog name
- [ ] A long name is shortened with an ellipsis and the full name and path are in a tooltip

## Design Notes
This reverses STORY-0041, which removed the vault folder name from the bar, status line and notices. The owner now asks for the name in the bar. Show the folder name only (not the path) in the bar; the path appears only in the tooltip. Status line and notices stay as STORY-0041 left them unless the owner says otherwise. Update `docs/specs/UX_EVIDENCE.md` with the reason for the reversal.

Tasks: TASK-0128.

## Dependencies
None.

## Open Questions
The owner confirmed on 2026-11-02 that reversing STORY-0041 is wanted. Still open: folder name or profile name when they differ? Proposal: the folder name, with the profile in the tooltip.
