---
id: STORY-0238
type: story
title: "Prompt library in the redesign style"
status: ready
epic: EPIC-073
parents: ['EPIC-073']
estimate: 3
labels: [m14, ui]
acceptance_criteria:
  - "The screen uses the redesign components: a gradient headline (view/gradient_panel.rs), redesign cards (view/home_hero.rs card) and the palette and style tokens (theme/style.rs)"
  - "Every control, action, message and control-protocol answer of the screen keeps its behaviour"
  - "Every text button has a tooltip (tests/text_button_tooltips.rs)"
  - "Light and dark both pass a screenshot check at 1280 by 800"
---

## Context
EPIC-073 restyles every screen of the application. Screens already done: Home, stack page, library, rehearsal, reports, settings, help, calendar, board, diary, achievements, graph, assistant and classroom. This story covers a screen that is still in the old look.

## Description
The Prompts screen: the prompt categories, the cards and the run and copy actions, in the redesign style.

## Acceptance Criteria
- [ ] Redesign components on the screen
- [ ] Behaviour and control-protocol answers unchanged
- [ ] Tooltips on every text button
- [ ] Screenshot check in both themes at 1280 by 800

## Design Notes
Presentation only. Colours come from palette_of or style_of; no literal font sizes.

## Dependencies
- STORY-0226 (palette tokens, done).

## Open Questions
None.
