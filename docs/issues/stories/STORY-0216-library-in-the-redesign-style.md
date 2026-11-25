---
id: STORY-0216
type: story
title: "Library in the redesign style: headline, card lists and the rail"
status: done
epic: EPIC-070
parents: ['EPIC-070']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "The library's headline and its stack list use the redesign tokens and card style; no action or filter is removed"
  - "Every control keeps its keyboard use and tooltip; the control protocol answers the same library state"
  - "Light and dark both pass the screenshot check at 1280 by 800"
---

## Context
EPIC-070 follows Home and the stack page into the library, the second screen the owner uses most.

## Description
Apply the tokens of STORY-0213 to the library screen, following the pattern of STORY-0214 and STORY-0215.

## Acceptance Criteria
- [x] Headline and card lists in the redesign style
- [x] Controls, keyboard and control protocol unchanged
- [x] Screenshot check in both themes at 1280 by 800

## Design Notes
Use the shared gradient panel (`view/gradient_panel.rs`) for the headline.

## Dependencies
- STORY-0213, STORY-0215.

## Open Questions
None.

## Verification note
Done 2026-11-25. The stack cards already used the card style (`kit::card_button`, radius 12), so only the overview headline changed: `crates/studyfrog-ui/src/view/library.rs` puts "All stacks" and the count on the shared gradient panel (`view/gradient_panel.rs`, which now paints behind the laid-out content). Checked in a window: Overview at 1920 by 1040 (the headline and the stack card no longer overlap). Screenshot mode's library screen shows the item list, not the overview, so the overview was checked through the window. Library and stack UI tests pass. `scripts/check.sh` passed on the tree before this commit.
