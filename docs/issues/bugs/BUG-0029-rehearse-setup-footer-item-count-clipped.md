---
id: BUG-0029
type: bug
title: "In the rehearse setup footer the item count is cut off at the right edge at 1280 by 800"
status: done
estimate: 1
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "At 1280 by 800 the item count in the rehearse setup footer is fully visible beside the Next button"
---

## Context
Found while auditing the Memory game entry point (STORY-0175, 2026-11-22, display :91). The footer of the rehearse setup's What step reads "11 item" with its last letters clipped at the window edge. Screenshots: `/tmp/claude-0/audit/g1/02-rehearse-1280.png` ("11 item"), `/tmp/claude-0/audit/g1/04-after-close.png` ("10 item").

**Fix (2026-11-22).** The setup's footer row wraps (`view/setup.rs` `navigation`), so the item count is never cut off and sits beside Next when there is room. Running app at 1280 by 800: "12 items match" is fully visible beside "Next: configure" (`/tmp/claude-0/fix1/0029-after-1280.png`); at 960 by 640 the Next button is now also in view (`/tmp/claude-0/fix1/0029-after-960.png`). Before: `/tmp/claude-0/fix1/0029-before.png`.

## Description
Let the footer wrap or shrink the chip row so the count fits. Reproduction: open Rehearse at 1280 by 800 with a vault holding ten or more items.

## Acceptance Criteria
- [x] At 1280 by 800 the item count in the rehearse setup footer is fully visible beside the Next button

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
