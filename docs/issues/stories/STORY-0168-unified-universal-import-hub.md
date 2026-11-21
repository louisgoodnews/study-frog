---
id: STORY-0168
type: story
title: "Unified Universal Import Entry Point and Redundant Button Consolidation"
status: in_progress
epic: EPIC-057
parents: ['EPIC-057']
estimate: 3
labels: [m14, ui, ext]
acceptance_criteria:
  - 'Single top-level Import button in the UI replaces separate card, stack, note, and question buttons'
  - 'Importer wizard auto-detects input file format (CSV, TSV, Markdown, Anki, PDF, JSON)'
  - 'Clear target destination picker (existing stack, new stack, or unstacked vault notes)'
---

## Context
Currently, users are confronted with redundant, competing import buttons depending on whether they are importing cards or stacks, creating confusion and visual clutter.

## Description
Consolidate all import workflows into a single unified entry point:
- Remove redundant duplicate buttons across the items view, stack page, and navigation bar.
- Provide a single prominent "Import" action that opens a modal with drag-and-drop or file picker.
- Automatically detect the file type (CSV/TSV tables, Markdown card decks, Anki `.apkg`, or PDF documents) and present clear options for target stack or vault location.

## Acceptance Criteria
- [x] Single top-level Import button in the UI replaces separate card, stack, note, and question buttons
- [ ] Importer wizard auto-detects input file format (CSV, TSV, Markdown, Anki, PDF, JSON)
- [x] Clear target destination picker (existing stack, new stack, or unstacked vault notes)

## Implementation so far (2026-11-21)
The items toolbar, the stack menu and the modal's More menu each have one "Import…" (the transfer dialog's Import tab, scoped to the open stack) and one "Export…". The duplicate "Import or export…" buttons and the launcher card in the legacy dialog are gone. Extension importers and card types stay behind "Add from extension…", shown only when extensions provide them. The Import tab says what the chosen path looks like before it is read (`studyfrog_service::xfer_adapters::describe_input`; tests in `studyfrog-service/tests/xfer_describe.rs`).

The Import tab's "Import into" menu offers as the file says, the vault root (no stack), each existing stack, and "New stack…", which takes a name. The name and the root go through `import_target`, so the preview and the import agree; an empty new-stack name is refused with the reason. In the service, an empty target is the vault root (`xfer_plan::destination`; tests in `studyfrog-service/tests/xfer_destination.rs`); the UI tests are in `studyfrog-ui/tests/transfer_destination.rs`.

Still open: PDF documents are detected and pointed at "Add PDF…" in a stack, but they are not imported as cards, so the second criterion is only partly met. Format detection covers the built-in importers (JSON, TOML, RON, zip, CSV and TSV, Anki `.apkg`, Obsidian folders).

## Design Notes
Consolidate UI logic from `modal_transfer_import.rs` and `modal_import.rs` into a unified controller adhering to Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-057

## Open Questions
None.
