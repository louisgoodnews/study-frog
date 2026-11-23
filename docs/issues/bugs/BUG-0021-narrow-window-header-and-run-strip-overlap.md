---
id: BUG-0021
type: bug
title: "At the 640 by 420 minimum, the window header and the rehearsal top strip overlap and the hint line is cut off"
status: done
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
- [x] At 640 by 420, the window header's logo, search box and buttons do not overlap each other
- [x] At 640 by 420, in the rehearsal view, the Comments, timer and Edit, Pause and Stop buttons do not overlap, and the keyboard hint line is wrapped or fits the window

## Design Notes
- Fixed on 2026-11-23. Cause: the header's right-hand buttons are laid out first and the brand, search, Create and Import share what is left; at 640 by 420 the brand (vault name, history buttons, logo) and the search ran into each other and under the bell, and the rehearsal top strip put Comments, Edit, Pause and Stop in a nested right-to-left layout that does not wrap, so Comments drew over the position and the mode pill. Fix: `view/navbar.rs` keeps only the sidebar toggle and the logo when the room after the window buttons is under 320 px (the vault switcher, history, vault name and brand text are left out), clips the search section to its room, and `view/menus.rs` `search_cluster` leaves Import out in that narrow header; `view/run.rs` `top_strip` is one wrapped row (`strip_buttons` for the four buttons). Before: `/tmp/claude-0/layout/before-home-640.png` and `/tmp/claude-0/layout/before-rehearse-640.png`. After: `/tmp/claude-0/layout/after4-home-640.png` (header) and `/tmp/claude-0/layout/after3-header-640.png` (rehearsal strip). Tests: `studyfrog-ui/tests/window_fit_layout.rs`: `the_header_brand_search_and_create_do_not_overlap_at_the_minimum_size` and `the_rehearsal_top_strip_buttons_do_not_overlap_at_the_minimum_size`. Remaining at 640 by 420, not an overlap: the Create label is cut at the edge of its section and Import is hidden. The hint line already wrapped at 640 in the before build, so it was not changed; the rehearsal test keeps it inside the window.
- Guardrails: P4 (nesting < 3) and P6 (files <= 250 lines).
- The rating buttons and the item card are not part of this bug; do not change their layout to fix it.

## Dependencies
- EPIC-065

## Open Questions
- None.
