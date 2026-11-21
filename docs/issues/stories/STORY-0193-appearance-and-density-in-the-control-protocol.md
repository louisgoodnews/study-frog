---
id: STORY-0193
type: story
title: "Appearance and density in the control protocol"
status: backlog
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
- [ ] appearance with theme, density, text size or accent and a value changes that one setting and answers with the new appearance
- [ ] a density change answers with the spacing it produces, so a test can check the spacing rule
- [ ] an unknown value is an error that names the choices

## Design Notes
None.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
