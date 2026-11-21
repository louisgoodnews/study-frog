---
id: STORY-0192
type: story
title: "Search and tooltips in the control protocol"
status: backlog
epic: EPIC-064
parents: ['EPIC-064']
estimate: 2
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'search focus focuses the search field of the screen in front and the answer names the focused field'
  - 'help tooltip with a control name answers with the explanation and the shortcut view::tips gives it'
  - 'the state snapshot names the focused field'
---

## Context
Ctrl+F and the slash key (STORY-0188) and the tooltip texts (STORY-0169) are not reachable. An agent cannot tell what a control does or whether a field has the keyboard.

## Description
The focused field is the egui memory's focused id; the protocol reads it on the UI thread.

## Acceptance Criteria
- [ ] search focus focuses the search field of the screen in front and the answer names the focused field
- [ ] help tooltip with a control name answers with the explanation and the shortcut view::tips gives it
- [ ] the state snapshot names the focused field

## Design Notes
None.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
