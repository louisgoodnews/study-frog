---
id: TASK-0192
type: task
title: "Plan to make the UI calmer"
status: done
epic: EPIC-025
parents: [STORY-0072]
estimate: 3
labels: [m10, ui, docs, decision]
acceptance_criteria:
  - "A short spec in docs/specs lists where the UI shows too much at once and what to cut, move or group, with the screenshots as the reference"
  - "The owner has accepted it"
---

## Context
The owner tested 0.3.0 for a short while on 2026-11-03 and sent five screenshots with a list of observations: missing padding, a crowded Create dialog header, no confirmation when cancelling a filled dialog, no import suggestions, hard-to-read window buttons, no in-app exit button, a cramped streak and an overall dense UI.

## Description
The owner finds the UI convoluted and cramped: one searches for information while being flooded with it. Write down a plan before changing screens.

## Acceptance Criteria
- [x] A short spec in docs/specs lists where the UI shows too much at once and what to cut, move or group, with the screenshots as the reference
- [x] The owner has accepted it

## Design Notes
The plan is in `docs/specs/ui-calm.md`; the owner said on 2026-11-03 that the library, the dashboard, the rehearsal setup, the settings and help are affected most, then the whole application. The first part is built (see the spec) and the owner accepted it on 2026-11-03 ("looks far better"); the remaining screens are TASK-0195. Ideas to put to the owner: more space between groups, fewer controls per row, a quieter sidebar, collapsible side lists, and progressive disclosure on the stack and Home pages.

## Dependencies
None.

## Open Questions
Which screens bother the owner most?
