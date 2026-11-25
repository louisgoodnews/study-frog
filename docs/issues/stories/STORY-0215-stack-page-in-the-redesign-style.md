---
id: STORY-0215
type: story
title: "Stack page in the redesign style: header, tab rail, summary cards and the ring"
status: backlog
epic: EPIC-070
parents: ['EPIC-070']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "The stack header uses the headline card, the primary Start rehearsal button stays the one filled control, and the tabs move to the rail style of STORY-0213"
  - "The Summary tab uses the card and ring tokens; its figures and grouping stay as they are"
  - "All stack actions and tabs are reachable as before, with the same keyboard use; the control protocol answers the same stack state"
  - "Light and dark both pass the screenshot check at 1280 by 800 and at phone width"
---

## Context
The stack page is where the owner works most (rehearsal, items, summary), so it follows Home.

## Description
Apply the tokens of STORY-0213 to `stack_view_ui.rs` and `stack_tabs.rs`. Layout changes only.

## Acceptance Criteria
- [ ] Headline header, one primary button, rail-style tabs
- [ ] Summary with card and ring tokens, same figures
- [ ] Actions, tabs, keyboard and control protocol unchanged
- [ ] Screenshot check in both themes at both sizes

## Design Notes
Keep the calm rule of at most one primary control per row.

## Dependencies
- STORY-0213.

## Open Questions
None.
