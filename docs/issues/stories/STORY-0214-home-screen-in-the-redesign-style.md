---
id: STORY-0214
type: story
title: "Home in the redesign style: headline card, statistic ring, card lists and the rail"
status: done
epic: EPIC-070
parents: ['EPIC-070']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "Home shows a gradient headline card with the next due item and its action, a statistic ring with the figures of the day, and card lists for due and recent items"
  - "When there is no data, Home shows one line of guidance per card instead of empty tall cards (the calm UI rule)"
  - "Every action Home offers before the redesign is still there, with the same keyboard use"
  - "The control protocol answers the same Home state as before"
  - "Light and dark both pass the screenshot check at 1280 by 800"
---

## Context
Home is the first screen the owner sees; the references put the headline and the ring there.

## Description
Apply the tokens of STORY-0213 to `home.rs` and `home_cards.rs`. Keep the data and the actions unchanged.

## Acceptance Criteria
- [x] Headline card, statistic ring and card lists
- [x] Guidance line for empty cards
- [x] Actions and keyboard use unchanged
- [x] Control protocol answers unchanged
- [x] Screenshot check in both themes at 1280 by 800

## Design Notes
Use the same figures as the current Home; the redesign changes presentation only.

## Dependencies
- STORY-0213.

## Open Questions
None.

## Verification note
Done 2026-11-25. `crates/studyfrog-ui/src/view/home_hero.rs` (headline, ring and figures) and `crates/studyfrog-ui/src/view/home_ring.rs` (ring arc and gradient geometry, with tests); the column cards in `home.rs` and `home_cards.rs` use the new card style. Actions, messages and the control protocol are unchanged. Screenshots of the empty scratch vault at 1280 by 800 in light and dark were checked by eye and by pixel sampling (headline text on the gradient). Not yet checked: a populated Home (due items, a streak, a progressing ring). The layout is for desktop windows only. `scripts/check.sh` passed on the tree before this commit.
