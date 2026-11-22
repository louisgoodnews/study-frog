---
id: STORY-0196
type: story
title: "In rehearsal view, the previous rating shows on the rating button in its colour"
status: done
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
- [x] the rating button that matches the item's previous difficulty shows it: green for easy, orange for medium, red for hard
- [x] an item with no previous difficulty shows no mark on any button
- [x] the colour is not the only cue: each mark also carries its word, so colour-blind learners can read it

## Design Notes
None.

## Implementation
- The rating button of the item's previous rating (`ItemContent.previous`) gets a line above it, "Last: <word>", in the colour of its difficulty: Again is Difficult (red), Hard is Medium (orange), Good and Easy are Easy (green). The word carries the meaning as well as the colour. The line is in the body size and one line in every column; at the 640 by 420 minimum "Last time: <word>" was cut off, so the prefix is "Last:". No line appears when the item has no previous rating (every column keeps a blank line, so the buttons stay level).
- Assumption: the story's "previous difficulty" is the difficulty the previous rating stood for; the story's colours are kept, but the mapping is my reading of it, to check with you.
- Checked in the running app (2026-11-22): red "Last: Difficult" above Again (`fix6-long-item-prev-again-red-1280x800.png`), orange "Last: Medium" above Hard (`fix6-prev-hard-card1-1280x800.png`, `fix6-prev-hard-card1-640x420.png`), green "Last: Easy" above Good (`light-prev-teal-card1-1280x800.png`, light theme). Before the fix, Good's button sat lower than its neighbours (`prev-good-card1-1280x800.png`).
- Ratings come from the item's saved reviews, so the line appears only for an item rated in a run that updates the schedule; a practice-only run does not write a review. Tests in `tests/run_previous_mark.rs` check the words and the layout.
- Open question for the owner: the Description says the previous difficulty is "the item's mark" (the Easy, Medium, Difficult control), but the build uses the previous rating, as the title and the epic say ("previous rating"). Confirm which one is meant.

## Dependencies
EPIC-065, and the features named above.

## Open Questions
None.
