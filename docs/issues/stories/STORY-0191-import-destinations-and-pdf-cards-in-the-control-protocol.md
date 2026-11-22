---
id: STORY-0191
type: story
title: "Import destinations and PDF cards in the control protocol"
status: done
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
- [x] import with --root and with --new-stack NAME chooses the vault root or a new stack, as the dialog does
- [x] import of a PDF previews and imports one card per page, and the preview lists the pages left out
- [x] pdf find with a text opens the PDF find field and sets its query

## Design Notes
None.

## Implementation
- `import` takes `--root` or `--new-stack NAME` (one destination at a time); the plan sends the same transfer messages as the dialog. The answer is `import_json` (preview warnings, else the last import's summary with its left-out pages).
- PDF import: `xfer_pdf::page_cards` gives one card per page with text and a warning per page without; tests in `crates/studyfrog-service/tests/xfer_pdf_cards.rs`.
- `pdf find TEXT [--file F]`: optional open of the vault-relative file, then `PdfMsg::Query`; the answer is `pdf_json` (open, path, query, pages found and pages without text, 1-based).
- Build vs buy: no new crates; the PDF text comes from the existing `studyfrog-pdf`.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
