---
id: STORY-0193
type: story
title: "Appearance and density in the control protocol"
status: review
epic: EPIC-064
parents: ['EPIC-064']
estimate: 2
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'appearance with theme, density, text size or accent and a value changes that one setting and answers with the new appearance'
  - 'a density change answers with the spacing it produces, so a test can check the spacing rule'
  - 'an unknown value is an error that names the choices'
---

## Context
Theme, density, text size and accent can only be changed in Settings. The density scale changed in STORY-0170.

## Description
The settings are the appearance struct; the messages are the appearance messages. The setting is saved, so the command is a write.

## Acceptance Criteria
- [x] appearance with theme, density, text size or accent and a value changes that one setting and answers with the new appearance
- [x] a density change answers with the spacing it produces, so a test can check the spacing rule
- [x] an unknown value is an error that names the choices

## Implementation (2026-11-22)
`appearance theme|density|text|accent <value>` (`control_words::appearance`) is one write: it sends the appearance message the settings screen sends, so the setting is saved like a click. The answer (`control_reply::appearance`) is the appearance now: theme, density, the spacing factor of that density (`theme_space::density_scale`: 1.0 compact, 1.25 comfortable, 1.5 spacious), the text size and the accent. A value outside the choices, or a text size outside 70 to 200 percent, is an error that names the choices. The text size is snapped to the nearest step by the settings screen, as a click is; the answer shows the step it got.

Tests: `studyfrog-ui/tests/control_appearance.rs`.

## Design Notes
None.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
