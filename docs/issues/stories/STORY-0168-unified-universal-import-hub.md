---
id: STORY-0168
type: story
title: "Unified Universal Import Entry Point and Redundant Button Consolidation"
status: backlog
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
- [ ] Single top-level Import button in the UI replaces separate card, stack, note, and question buttons
- [ ] Importer wizard auto-detects input file format (CSV, TSV, Markdown, Anki, PDF, JSON)
- [ ] Clear target destination picker (existing stack, new stack, or unstacked vault notes)

## Design Notes
Consolidate UI logic from `modal_transfer_import.rs` and `modal_import.rs` into a unified controller adhering to Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-057

## Open Questions
None.
