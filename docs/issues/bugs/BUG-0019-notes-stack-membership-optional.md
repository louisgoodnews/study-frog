---
id: BUG-0019
type: bug
title: Notes being in stacks should be optional not required
status: done
estimate: 3
labels: [m12, core, service, ui]
acceptance_criteria:
  - "Creating a note without choosing a parent stack succeeds and places the note in the vault root directory"
  - "The note creation modal allows leaving the stack selection empty or selecting 'No stack (Vault root)'"
  - "Loose notes are fully indexed, searchable, editable, and linkable via wikilinks"
---

## Context
Currently, the item creation workflow enforces that all non-stack items (including notes) must belong to a parent stack. Attempting to create a note without specifying a parent stack fails with the validation error `"choose the stack it goes into"`. However, in a knowledge base and study environment, standalone notes (general reference, scratchpads, evergreen notes, or index notes) often live at the root of the vault or outside flashcard decks. Requiring a stack for notes is an artificial friction point.

## Description
Make stack assignment optional for notes in `studyfrog-service` and `studyfrog-ui`:
1. In `crates/studyfrog-service/src/create_more.rs`, update item creation validation so that `item.kind == NewKind::Note` is permitted when `item.parent.is_none()`. Place loose notes in the vault root (`session.root.clone()`).
2. In `crates/studyfrog-ui/src/modal_form.rs` and `modal_create.rs`, allow the stack selector field to be optional for notes (e.g. showing `"(None / Root)"`), and disable the `"Choose the stack it goes into"` validation blocker when `kind == NewKind::Note`.
3. Verify that loose notes roundtrip through the SQLite indexer, file tree navigator, and Markdown editor without error.

## Acceptance Criteria
- [x] Creating a note without choosing a parent stack succeeds and places the note in the vault root directory
- [x] The note creation modal allows leaving the stack selection empty or selecting 'No stack (Vault root)'
- [x] Loose notes are fully indexed, searchable, editable, and linkable via wikilinks

## Design Notes
- Touchpoints: `crates/studyfrog-service/src/create_more.rs`, `crates/studyfrog-ui/src/modal_form.rs`, `crates/studyfrog-ui/src/view/modal_create.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- None.

## Open Questions
- None.
