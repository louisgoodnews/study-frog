---
id: STORY-0214
type: story
title: "Home in the redesign style: headline card, statistic ring, card lists and the rail"
status: backlog
epic: EPIC-070
parents: ['EPIC-070']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - "Home shows a gradient headline card with the next due item and its action, a statistic ring with the figures of the day, and card lists for due and recent items"
  - "When there is no data, Home shows one line of guidance per card instead of empty tall cards (the calm UI rule)"
  - "Every action Home offers before the redesign is still there, with the same keyboard use"
  - "The control protocol answers the same Home state as before"
  - "Light and dark both pass the screenshot check at 1280 by 800 and at phone width"
---

## Context
Home is the first screen the owner sees; the references put the headline and the ring there.

## Description
Apply the tokens of STORY-0213 to `home.rs` and `home_cards.rs`. Keep the data and the actions unchanged.

## Acceptance Criteria
- [ ] Headline card, statistic ring and card lists
- [ ] Guidance line for empty cards
- [ ] Actions and keyboard use unchanged
- [ ] Control protocol answers unchanged
- [ ] Screenshot check in both themes at both sizes

## Design Notes
Use the same figures as the current Home; the redesign changes presentation only.

## Dependencies
- STORY-0213.

## Open Questions
None.
