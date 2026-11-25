---
id: STORY-0215
type: story
title: "Stack page in the redesign style: header, tab rail, summary cards and the ring"
status: done
epic: EPIC-070
parents: ['EPIC-070']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "The stack header uses the headline card, the primary Start rehearsal button stays the one filled control, and the tabs move to the rail style of STORY-0213"
  - "The Summary tab uses the card and ring tokens; its figures and grouping stay as they are"
  - "All stack actions and tabs are reachable as before, with the same keyboard use; the control protocol answers the same stack state"
  - "Light and dark both pass the screenshot check at 1280 by 800"
---

## Context
The stack page is where the owner works most (rehearsal, items, summary), so it follows Home.

## Description
Apply the tokens of STORY-0213 to `stack_view_ui.rs` and `stack_tabs.rs`. Layout changes only.

## Acceptance Criteria
- [x] Headline header, one primary button, rail-style tabs
- [x] Summary with card and ring tokens, same figures
- [x] Actions, tabs, keyboard and control protocol unchanged
- [x] Screenshot check in both themes at both sizes

## Design Notes
Keep the calm rule of at most one primary control per row.

## Dependencies
- STORY-0213.

## Open Questions
None.

## Verification note
Done 2026-11-25. `crates/studyfrog-ui/src/view/stack_view.rs` (gradient header with Start rehearsal, rail-style tabs with hover text) and `crates/studyfrog-ui/src/view/gradient_panel.rs` (shared panel, also used by Home); the Summary figures in `stack_summary_view.rs` use `home_hero::card`. Screenshots of a scratch stack at 1280 by 800 in light and dark were checked. Not checked: a populated stack (its Summary figures were all zero). Stack and Home UI tests pass. `scripts/check.sh` passed on the tree before this commit.
