---
id: STORY-0195
type: story
title: "In rehearsal view, the item takes more of the screen"
status: review
epic: EPIC-065
parents: ['EPIC-065']
estimate: 3
labels: [m14, rehearsal, ui]
acceptance_criteria:
  - 'the item area takes at least 70 percent of the width and most of the height left under the header at 1280 by 800'
  - 'a long item scrolls inside its area instead of pushing the rating buttons off the screen'
  - 'the rating and mark buttons stay in view at every window size the app supports'
---

## Context
The item sits in a narrow column in the middle of the rehearsal view, with its answer and rating buttons below. At 1280 by 800 the item is small and most of the screen is empty.

## Description
Check with the screenshot command on the run screen at 1280 by 800 and 1600 by 1000, in both themes.

## Acceptance Criteria
- [x] the item area takes at least 70 percent of the width and most of the height left under the header at 1280 by 800
- [x] a long item scrolls inside its area instead of pushing the rating buttons off the screen
- [x] the rating and mark buttons stay in view at every window size the app supports

## Design Notes
None.

## Implementation
- The rehearsal card takes the whole width (it was the reading measure, about 450 pt, at 1280 by 800). Its prose may run to 1.6 reading measures (`preview::RUN_PROSE_SHARE`, set for the run card through a typed temporary value), which is about 110 characters a line: a readability trade made for this story, to check with you.
- The item's text scrolls in its own area (`run_card::item_scroll`, height from the window less `ITEM_CHROME`), and the rating buttons sit under it, outside that area.
- Screenshots (`--screenshot run-long --size WxH --theme T`): 1280 by 800 and 1600 by 1000 in both themes, 1280 by 600, and the 640 by 420 minimum. At 640 by 420 the fixed parts of the card take nearly the whole height: the buttons are visible, the item area is nearly collapsed.

## Dependencies
EPIC-065, and the features named above.

## Open Questions
None.
