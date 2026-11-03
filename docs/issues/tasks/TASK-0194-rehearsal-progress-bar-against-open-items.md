---
id: TASK-0194
type: task
title: Rehearsal progress bar against the open items
status: done
epic: EPIC-025
parents: [STORY-0072]
estimate: 3
labels: [m10, ui, rehearsal]
acceptance_criteria:
  - "The rehearsal screen shows a full-width bar and the text 'N done · M left', counted from the run's list of items still open"
  - "An item whose answer is showing is still open until it is rated or skipped, and items put back to be repeated count as open and are named"
  - "The bar has an accessible name and its numbers are text, not only a fill"
---

## Context
The owner asked on 2026-11-03, after trying the 0.3.1 build: widgets next to a scroll bar need padding, and the rehearsal view needs a progress bar against the items still open.

## Description
The thin bar under the run header counted the position, treated a revealed but unrated item as done and showed no numbers.

## Acceptance Criteria
- [x] The rehearsal screen shows a full-width bar and the text 'N done · M left', counted from the run's list of items still open
- [x] An item whose answer is showing is still open until it is rated or skipped, and items put back to be repeated count as open and are named
- [x] The bar has an accessible name and its numbers are text, not only a fill

## Design Notes
`run_progress` (pure): done is the cursor, open is the rest of the queue, repeats are summed from the run's requeue counts; the queue grows when an item is put back, so the bar can step back and says why. `kit::progress_with_text` paints the track, the tinted fill and the centred text itself, because egui's bar draws its text in the accent colour at the left. Tested in `tests/run_progress.rs`; seen in a screenshot of a started run (`--screen run`).

## Dependencies
None.

## Open Questions
None.
