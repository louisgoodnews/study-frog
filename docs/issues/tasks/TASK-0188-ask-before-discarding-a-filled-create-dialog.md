---
id: TASK-0188
type: task
title: Ask before discarding a filled Create dialog
status: done
epic: EPIC-025
parents: [STORY-0072]
estimate: 2
labels: [m10, ui]
acceptance_criteria:
  - "When any field has content or a changed value, Cancel, the close button, Escape and a click outside ask 'Discard what you wrote?' with Keep editing and Discard"
  - "An untouched dialog closes at once"
---

## Context
The owner tested 0.3.0 for a short while on 2026-11-03 and sent five screenshots with a list of observations: missing padding, a crowded Create dialog header, no confirmation when cancelling a filled dialog, no import suggestions, hard-to-read window buttons, no in-app exit button, a cramped streak and an overall dense UI.

## Description
Add a 'dirty' check on the form and a confirmation inside the dialog.

## Acceptance Criteria
- [x] When any field has content or a changed value, Cancel, the close button, Escape and a click outside ask 'Discard what you wrote?' with Keep editing and Discard
- [x] An untouched dialog closes at once

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
