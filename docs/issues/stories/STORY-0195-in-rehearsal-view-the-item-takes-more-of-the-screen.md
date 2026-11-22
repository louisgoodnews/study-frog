---
id: STORY-0195
type: story
title: "In rehearsal view, the item takes more of the screen"
status: done
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
- The rehearsal card takes the whole width (it was the reading measure, about 450 pt, at 1280 by 800). Its prose may run to 1.4 reading measures (`preview::RUN_PROSE_SHARE`, set for the run card through a typed temporary value), which is about 100 characters a line. The owner accepted 100 characters for now (2026-11-22).
- The item's text scrolls in its own area (`run_card::item_scroll`, height from the window less `ITEM_CHROME`), and the rating buttons sit under it, outside that area.
- Screenshots (`--screenshot run-long --size WxH --theme T`): 1280 by 800 and 1600 by 1000 in both themes, 1280 by 600, and the 640 by 420 minimum. At 640 by 420 the fixed parts of the card take nearly the whole height: the buttons are visible, the item area is nearly collapsed.

- Checked in the running app (2026-11-22, `/tmp/claude-0/epic65/`): at 1280 by 800 the card spans 85 percent of the window width and most of the height under the header (`fix6-prev-hard-card1-1280x800.png`); a long item scrolls in its area with the buttons below it (`fix6-long-item-prev-again-red-1280x800.png`); the buttons stay in view at 1600 by 1000 (`fix6-prev-hard-card1-1600x1000.png`) and at 640 by 420 (`fix6-prev-hard-card1-640x420.png`).
- The rating row now reserves the same line above every button, and the item area's height takes that line into account (`run_card::item_scroll`, `rating_row_height`). Before this, a previous rating pushed the Good button down and cut off the hint line at 1280 by 800 (`prev-good-card1-1280x800.png`, pre-fix). Test: `tests/run_previous_mark.rs`, the layout test.

## Dependencies
EPIC-065, and the features named above.

## Open Questions
None.
