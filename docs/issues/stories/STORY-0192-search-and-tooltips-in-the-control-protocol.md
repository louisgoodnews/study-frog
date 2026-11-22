---
id: STORY-0192
type: story
title: "Search and tooltips in the control protocol"
status: review
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
- [x] search focus focuses the search field of the screen in front and the answer names the focused field
- [x] help tooltip with a control name answers with the explanation and the shortcut view::tips gives it
- [x] the state snapshot names the focused field

## Design Notes
None.

## Implementation
- `search --focus [TEXT]` sets `chrome.search_focus`; the library's search fields and the PDF find field take the keyboard on their next frame (`view::search_focus::focus_field` with the request) and report it with `ChromeMsg::SearchFocusTaken(name)`. The answer is `focus: {pending, focused}`; `state` carries `focused_field` and `search_focus_pending`.
- `help tooltip NAME` answers the explanation and text of a control from `view::tips` (`explain_of`); an unknown name lists the names it knows.
- Limit: the focus is confirmed by a frame that draws the field; the protocol tests run without a frame, so `pending` stays true there. Not yet checked in the running app.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
