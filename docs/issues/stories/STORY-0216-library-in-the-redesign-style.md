---
id: STORY-0216
type: story
title: "Library in the redesign style: headline, card lists and the rail"
status: backlog
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
- [ ] Headline and card lists in the redesign style
- [ ] Controls, keyboard and control protocol unchanged
- [ ] Screenshot check in both themes at 1280 by 800

## Design Notes
Use the shared gradient panel (`view/gradient_panel.rs`) for the headline.

## Dependencies
- STORY-0213, STORY-0215.

## Open Questions
None.
