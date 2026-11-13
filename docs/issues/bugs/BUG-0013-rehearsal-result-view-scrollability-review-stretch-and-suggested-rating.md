---
id: BUG-0013
type: bug
title: "Rehearsal result view scrollability, stretched review area, and suggested rating visibility"
status: done
epic: EPIC-031
parents: []
estimate: 2
labels: [m11, rehearsal, ui]
acceptance_criteria:
  - "The entire rehearsal result (summary) view is wrapped in an outer vertical ScrollArea so all sections remain accessible on any window height"
  - "The Review section stretches dynamically in height with its own inner vertical ScrollArea instead of being capped at 140px"
  - "Every review item card displays a Suggested rating value (along with reading word count and time when available) even when no rating change is needed"
---

## Context
On the rehearsal result screen ("Run finished"), the overall view was not scrollable, the "Review" area was restricted to `max_height(140.0)` (showing barely two items at a time), and "Suggested rating" appeared to be missing whenever `row.suggestion` was `None`.

## Description
Root cause:
1. `crates/studyfrog-ui/src/view/summary.rs` rendered all summary widgets directly without an outer `egui::ScrollArea::vertical()`, and hardcoded `.max_height(140.0)` on the Review `ScrollArea`.
2. `crates/studyfrog-ui/src/view/summary_items.rs` only rendered `"Suggested rating: ..."` and the reading speed metrics (`"{words} words in {s} s"`) inside `if let (Some(s), Some(rated)) = (row.suggestion, row.rated)`, leaving items without an override suggestion with no `Suggested rating:` label at all.

## Acceptance Criteria
- [x] The entire rehearsal result (summary) view is wrapped in an outer vertical ScrollArea so all sections remain accessible on any window height
- [x] The Review section stretches dynamically in height with its own inner vertical ScrollArea instead of being capped at 140px
- [x] Every review item card displays a Suggested rating value (along with reading word count and time when available) even when no rating change is needed

## Design Notes
- Wrap `summary::draw` in an outer `egui::ScrollArea::vertical().id_salt("summary-outer-scroll").auto_shrink([false, false])`.
- Compute the inner Review `ScrollArea` height dynamically from `ui.available_height()` (stretching to fill available vertical space while keeping a generous minimum height) with `.id_salt("summary-review-scroll")`.
- In `summary_items::card`, always show `"Suggested rating: ..."` (showing the suggested override rating when `row.suggestion` is `Some`, or the confirmed/current rating when no change is suggested) and display reading word count/time when available.

## Dependencies
None.

## Open Questions
None.
