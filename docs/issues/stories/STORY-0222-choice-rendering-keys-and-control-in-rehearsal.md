---
id: STORY-0222
type: story
title: "Choice rendering in the rehearsal card: options, number keys, submit, and the control protocol"
status: review
epic: EPIC-072
parents: ['EPIC-072']
estimate: 5
labels: [m14, ui, rehearsal]
acceptance_criteria:
  - "The rehearsal card shows the options as radio buttons (single) or checkboxes (multiple), with the prompt above and the Submit button below"
  - "Number keys 1 to 9 choose options; Enter submits; Space shows the answer when no choice is pending"
  - "After submitting, the card shows the correct options and the user's choices, with the same colours as the mock exam result"
  - "`rehearse choose <n>` and `rehearse submit` answer through the control protocol"
  - "Light and dark both pass a screenshot check at 1280 by 800"
---

## Context
The choice data and grading come from STORY-0220 and STORY-0221; this story puts them on screen.

## Description
Draw the choice in the rehearsal card in the redesign style (EPIC-070), handle the keys, and add the two control commands.

## Acceptance Criteria
- [ ] Radio and checkbox options, Submit, Enter and number keys
- [ ] Result view with correct options marked
- [ ] Control commands `rehearse choose` and `rehearse submit`
- [ ] Screenshot check in both themes

## Design Notes
Use the palette tokens and the gradient panel, not colour literals; the card itself keeps its band and centred buttons.

## Dependencies
- STORY-0220, STORY-0221.

## Open Questions
None.
