---
id: STORY-0250
type: story
title: "Expand prose and card text wrapping to full available container width"
status: done
epic: EPIC-073
parents: ['EPIC-073']
estimate: 2
labels: [m14, ui]
acceptance_criteria:
  - 'Rendered card prompts, card answers, and Markdown previews in Rehearse and Item views wrap to the full available card/column width instead of wrapping prematurely at ~45% width'
  - 'Text wrapping adapts responsively on window resize with balanced padding inside cards'
  - 'Headless UI tests verify full-width text wrapping in Rehearse cards and previews'
---

## Context
Reported by the user with a screenshot of the Rehearse screen: card answer text (`"Nicht beobachtbare Parameter, die sich mit einer bestimmten Wahrscheinlichkeit im Antwortverhalten von Personen zeigen."`) wraps prematurely at roughly half the card width, leaving a wide blank region on the right of the full-width Rehearse card.

## Description
Remove overly narrow `typo::measure(ui)` width caps on rendered Markdown/card text blocks so text expands cleanly across the full available width of its enclosing card or panel (`TASK-0313`).

## Acceptance Criteria
- [x] Rendered card prompts, card answers, and Markdown previews in Rehearse and Item views wrap to the full available card/column width instead of wrapping prematurely at ~45% width
- [x] Text wrapping adapts responsively on window resize with balanced padding inside cards
- [x] Headless UI tests verify full-width text wrapping in Rehearse cards and previews

## Design Notes
- Child task: `TASK-0313`
- Review any existing reading-measure tests (e.g. `reading_measure_ui.rs`, `editor_measure_ui.rs`) and align expectations with the full-width wrapping requirement.

## Dependencies
- `EPIC-073`

## Open Questions
None.
