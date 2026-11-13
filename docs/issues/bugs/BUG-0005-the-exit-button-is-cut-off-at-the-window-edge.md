---
id: BUG-0005
type: bug
title: "The exit button is cut off at the window edge"
status: done
epic: EPIC-024
parents: [STORY-0073]
estimate: 2
labels: [m11, ui]
acceptance_criteria:
  - "The window buttons are anchored to the window's right edge and are never pushed out of the window by the rest of the bar"
  - "A gap separates close from maximise so that a click aimed at one cannot land on the other"
---

## Context
The owner sent this after trying 0.3.1 on 2026-11-03.

## Description
On the owner's screen the red close button was cut off at the right edge (screenshot): the bar's sections were laid out left to right with fixed widths, so wider content on the left pushed the right section out of the window.

## Acceptance Criteria
- [x] The window buttons are anchored to the window's right edge and are never pushed out of the window by the rest of the bar
- [x] A gap separates close from maximise so that a click aimed at one cannot land on the other

## Design Notes
The right-hand buttons are laid out first, right to left, and the brand and search take the room that is left (`view/navbar.rs`). Test: the close button lies inside the window and has a gap to maximise.

## Dependencies
None.

## Open Questions
None.
