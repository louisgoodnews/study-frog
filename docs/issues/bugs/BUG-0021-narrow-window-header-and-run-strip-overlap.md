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

## Fix (2026-11-23, second round)

- The rest at 640 by 420 is fixed. The search cluster moved to `view/menus_search.rs` (new). In a narrow header Create is laid out first at the right edge and the search field takes what is left, so "+ Create" is whole and the search field is not drawn under it. Import is not drawn in the narrow header; it is the first entry of the Profile menu ("Import...", `view/menus.rs` `profile`), which opens the same transfer dialog on the Import tab.
- Checked in the running app: before `/tmp/claude-0/g1/shots/before-home-640.png` ("+ Cre" cut at its edge, no Import); after `/tmp/claude-0/g1/shots/after1-home-640.png` (Create whole, no overlap), `after1-profile-640.png` (Import in the Profile menu), `after1-import-640.png` (it opens the dialog), `after1-rehearse-640.png` (the run strip wraps, nothing overlaps, the hint line wraps); at 1280 by 800 the header keeps Import (`after1-home-1280.png`).
- Known: at 640 by 420 the search field is about 40 px wide, a stub. Holding a full search field there would need the logo or the sidebar toggle to go, or Create to be shortened; that is a design choice for the owner.
- Tests: `studyfrog-ui/tests/window_fit_layout.rs` (both tests pass); the dialog at 640 is wider than the window (`after1-import-640.png`), a separate layout problem not in this bug.
