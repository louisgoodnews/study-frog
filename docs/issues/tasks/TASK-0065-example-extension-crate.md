---
id: TASK-0065
type: task
title: Example card type and importer
status: done
epic: EPIC-016
parents: [STORY-0031]
estimate: 5
labels: [m8, ext, ui]
acceptance_criteria:
  - "A tab-separated importer and a cloze card type in `examples/studyfrog-ext-example`, depending on `studyfrog-ext` only (checked by `scripts/check-deps.sh`)"
  - "The UI offers card types in the new-card form and an Import from a file panel; cards are written through the service with one reindex"
  - "Bad lines and sources are reported with their reason"
---

## Context


## Description
Example card type and importer.

## Acceptance Criteria
- [x] A tab-separated importer and a cloze card type in `examples/studyfrog-ext-example`, depending on `studyfrog-ext` only (checked by `scripts/check-deps.sh`)
- [x] The UI offers card types in the new-card form and an Import from a file panel; cards are written through the service with one reindex
- [x] Bad lines and sources are reported with their reason

## Design Notes
`service::import`, `ui::import_ui`, `ui::exec_import`; the application registers the example with one call; the file dialog is injected through `Host`.

## Dependencies
TASK-0064.

## Open Questions
None.
