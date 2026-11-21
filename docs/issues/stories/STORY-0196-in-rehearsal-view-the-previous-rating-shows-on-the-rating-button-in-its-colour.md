---
id: STORY-0196
type: story
title: "In rehearsal view, the previous rating shows on the rating button in its colour"
status: backlog
epic: EPIC-065
parents: ['EPIC-065']
estimate: 3
labels: [m14, rehearsal, ui]
acceptance_criteria:
  - 'the rating button that matches the item''s previous difficulty shows it: green for easy, orange for medium, red for hard'
  - 'an item with no previous difficulty shows no mark on any button'
  - 'the colour is not the only cue: each mark also carries its word, so colour-blind learners can read it'
---

## Context
The rating buttons (Again, Hard, Good, Easy) and the difficulty marks (Easy, Medium, Difficult) do not show how the item was rated last time, so the learner cannot compare.

## Description
The previous difficulty is the item's mark (the review's current mark). Map easy to the success token, medium to the warning token and difficult to the danger token of the palette.

## Acceptance Criteria
- [ ] the rating button that matches the item's previous difficulty shows it: green for easy, orange for medium, red for hard
- [ ] an item with no previous difficulty shows no mark on any button
- [ ] the colour is not the only cue: each mark also carries its word, so colour-blind learners can read it

## Design Notes
None.

## Dependencies
EPIC-065, and the features named above.

## Open Questions
None.
