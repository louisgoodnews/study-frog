---
id: BUG-0029
type: bug
title: "In the rehearse setup footer the item count is cut off at the right edge at 1280 by 800"
status: backlog
estimate: 1
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "At 1280 by 800 the item count in the rehearse setup footer is fully visible beside the Next button"
---

## Context
Found while auditing the Memory game entry point (STORY-0175, 2026-11-22, display :91). The footer of the rehearse setup's What step reads "11 item" with its last letters clipped at the window edge. Screenshots: `/tmp/claude-0/audit/g1/02-rehearse-1280.png` ("11 item"), `/tmp/claude-0/audit/g1/04-after-close.png` ("10 item").

## Description
Let the footer wrap or shrink the chip row so the count fits. Reproduction: open Rehearse at 1280 by 800 with a vault holding ten or more items.

## Acceptance Criteria
- [ ] At 1280 by 800 the item count in the rehearse setup footer is fully visible beside the Next button

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
