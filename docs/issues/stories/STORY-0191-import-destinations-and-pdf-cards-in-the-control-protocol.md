---
id: STORY-0191
type: story
title: "Import destinations and PDF cards in the control protocol"
status: backlog
epic: EPIC-064
parents: ['EPIC-064']
estimate: 3
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'import with --root and with --new-stack NAME chooses the vault root or a new stack, as the dialog does'
  - 'import of a PDF previews and imports one card per page, and the preview lists the pages left out'
  - 'pdf find with a text opens the PDF find field and sets its query'
---

## Context
The import destinations (as the file says, the vault root, a new stack; STORY-0168) and PDF import as cards are not reachable from the import command.

## Description
Add the flags to the import words and the transfer messages to the plan. The transfer messages for these choices already exist (scope, new stack, name).

## Acceptance Criteria
- [ ] import with --root and with --new-stack NAME chooses the vault root or a new stack, as the dialog does
- [ ] import of a PDF previews and imports one card per page, and the preview lists the pages left out
- [ ] pdf find with a text opens the PDF find field and sets its query

## Design Notes
None.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
