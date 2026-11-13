---
id: TASK-0092
type: task
title: "Native folder dialog for the vault"
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, followup]
acceptance_criteria:
  - "A Choose folder button on the welcome screen opens the native dialog and opens the chosen folder"
  - "The dialog is supplied by the app binary, so the UI crate and its tests do not link it"
---

## Context
A decision from the owner review.

## Description
Native folder dialog for the vault.

## Acceptance Criteria
- [x] A Choose folder button on the welcome screen opens the native dialog and opens the chosen folder
- [x] The dialog is supplied by the app binary, so the UI crate and its tests do not link it

## Design Notes
`ui::host::Host` carries the picker; `studyfrog-app::picker` uses `rfd` through the XDG desktop portal (no GTK). Cannot be exercised headlessly: a fake picker is tested; the real portal dialog has not been tried on a desktop session.

## Dependencies
None.

## Open Questions
None.
