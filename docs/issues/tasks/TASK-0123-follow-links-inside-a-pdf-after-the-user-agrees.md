---
id: TASK-0123
type: task
title: Follow links inside a PDF after the user agrees
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 3
labels: [m9, pdf]
acceptance_criteria:
  - "Links on a page that lead to another page of the file are followed with one click"
  - "A link to a web or mail address is offered with the address shown and is opened only after the user presses Open"
  - "Other kinds of link action (launch, JavaScript, files, other schemes) are never run and the user is told"
  - "Link areas are outlined on the page and tested with named destinations as well as direct ones"
---

## Context
The owner asked on 2026-10-31 for links inside a PDF to be followed with the user's approval, and for the PDF feature to be released as version 0.2.0.

## Description
Reads the link annotations of the shown page together with its words and handles the click in the viewer.

## Acceptance Criteria
- [x] Links on a page that lead to another page of the file are followed with one click
- [x] A link to a web or mail address is offered with the address shown and is opened only after the user presses Open
- [x] Other kinds of link action (launch, JavaScript, files, other schemes) are never run and the user is told
- [x] Link areas are outlined on the page and tested with named destinations as well as direct ones

## Design Notes
Only http, https and mailto are ever offered; nothing is opened without the click on Open.

## Dependencies
TASK-0112.

## Open Questions
None.
