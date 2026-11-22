---
id: BUG-0021
type: bug
title: "At the 640 by 420 minimum, the window header and the rehearsal top strip overlap and the hint line is cut off"
status: backlog
estimate: 2
labels: [m14, ui]
acceptance_criteria:
  - "At 640 by 420, the window header's logo, search box and buttons do not overlap each other"
  - "At 640 by 420, in the rehearsal view, the Comments, timer and Edit, Pause and Stop buttons do not overlap, and the keyboard hint line is wrapped or fits the window"
---

## Context
Found while checking EPIC-065 in the running app (2026-11-22), at the 640 by 420 minimum window size (`/tmp/claude-0/epic65/long-item-640x420-pre.png`, `/tmp/claude-0/epic65/fix5-prev-hard-card1-640x420.png`):
- the app logo is drawn over the search box in the window header, and the search box is squeezed to "Search it...";
- in the rehearsal view's top strip, the "Comments" button is drawn over the timer pill, and the timer reads "9.9 s" beside it;
- the keyboard hint line under the card is one long line, so its right end ("Esc stop") is cut off at the window edge.

The rehearsal card and its rating buttons stay in view at this size; the overlap is in the header and the top strip, not in the item.

## Description
Find why the header's and the run strip's items are laid out wider than the window at the minimum size (`crates/studyfrog-ui/src/view/run.rs` `top_strip`, and the header in the app shell), and let them wrap or shrink so that nothing overlaps. Wrap the hint line in the rehearsal view. Check at 640 by 420 and at 1280 by 800 with `ctl screenshot`.

## Acceptance Criteria
- [ ] At 640 by 420, the window header's logo, search box and buttons do not overlap each other
- [ ] At 640 by 420, in the rehearsal view, the Comments, timer and Edit, Pause and Stop buttons do not overlap, and the keyboard hint line is wrapped or fits the window

## Design Notes
- Guardrails: P4 (nesting < 3) and P6 (files <= 250 lines).
- The rating buttons and the item card are not part of this bug; do not change their layout to fix it.

## Dependencies
- EPIC-065

## Open Questions
- None.
