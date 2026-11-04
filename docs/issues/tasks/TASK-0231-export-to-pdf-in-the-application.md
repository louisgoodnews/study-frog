---
id: TASK-0231
type: task
title: Export to PDF in the application
status: done
epic: EPIC-026
parents: [STORY-0082]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "Export to PDF is offered on the item page (More actions), in the library's bulk bar, on the stack page and in the Export screen, with a dialog for the options of TASK-0230 and a file picker"
  - "The dialog shows a preview of the first pages before the file is written, tells the user where the file went, and the notification opens the PDF in the viewer or its folder"
  - "Every action has an accessible name and a keyboard route, and the dialog follows docs/specs/ui-calm.md in both themes"
  - "Headless UI tests cover the dialog, the options and the notification"
---

## Context
The owner asked on 2026-11-04 for a new feature: notes can be exported to PDF, which needs a converter from Markdown to PDF.

## Description
The user's way to the export.

## Acceptance Criteria
- [x] Export to PDF is offered on the item page (More actions), in the library's bulk bar, on the stack page and in the Export screen, with a dialog for the options of TASK-0230 and a file picker
- [x] The dialog shows a preview of the first pages before the file is written, tells the user where the file went, and the notification opens the PDF in the viewer or its folder
- [x] Every action has an accessible name and a keyboard route, and the dialog follows docs/specs/ui-calm.md in both themes
- [x] Headless UI tests cover the dialog, the options and the notification

## Design Notes
Builds on the Export screen and the bulk bar. The preview uses the PDF page renderer that already exists.

## Dependencies
TASK-0230.

## Open Questions
None.
