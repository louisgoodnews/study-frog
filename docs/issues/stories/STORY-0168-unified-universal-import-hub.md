---
id: STORY-0168
type: story
title: "Unified Universal Import Entry Point and Redundant Button Consolidation"
status: review
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
- [x] Importer wizard auto-detects input file format (CSV, TSV, Markdown, Anki, PDF, JSON)
- [x] Clear target destination picker (existing stack, new stack, or unstacked vault notes)

## Implementation so far (2026-11-21)
The items toolbar, the stack menu and the modal's More menu each have one "Import…" (the transfer dialog's Import tab, scoped to the open stack) and one "Export…". The duplicate "Import or export…" buttons and the launcher card in the legacy dialog are gone. Extension importers and card types stay behind "Add from extension…", shown only when extensions provide them. The Import tab says what the chosen path looks like before it is read (`studyfrog_service::xfer_adapters::describe_input`; tests in `studyfrog-service/tests/xfer_describe.rs`).

The Import tab's "Import into" menu offers as the file says, the vault root (no stack), each existing stack, and "New stack…", which takes a name. The name and the root go through `import_target`, so the preview and the import agree; an empty new-stack name is refused with the reason. In the service, an empty target is the vault root (`xfer_plan::destination`; tests in `studyfrog-service/tests/xfer_destination.rs`); the UI tests are in `studyfrog-ui/tests/transfer_destination.rs`.

Still open: PDF documents are detected and pointed at "Add PDF…" in a stack, but they are not imported as cards, so the second criterion is only partly met. Format detection covers the built-in importers (JSON, TOML, RON, zip, CSV and TSV, Anki `.apkg`, Obsidian folders).

## Design Notes
Consolidate UI logic from `modal_transfer_import.rs` and `modal_import.rs` into a unified controller adhering to Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-057

## Implementation (2026-11-22)
Built: a Markdown reader (`studyfrog-service::xfer_markdown`, registered as the eighth adapter). A `.md` file is detected by the import dialog ("Looks like Markdown cards…") and read as one card per `## ` heading, or as one note when it has none. Headings inside code fences are text. Tests: `studyfrog-service/tests/xfer_markdown.rs`; the adapter count is checked in `xfer_hostile.rs`.

Correction: the "Still open" note above is out of date for PDF. `xfer_pdf::from_pdf` already reads a PDF as one card per page with text (through `read_source`), and `tests/xfer_pdf.rs` covers it. The second criterion is now met for the six listed formats (CSV, TSV, Markdown, Anki, PDF, JSON) and the others the registry holds (TOML, RON, zip, Obsidian folder).

Not done: the criterion is checked by tests and by reading the dialog text. Whether the Import tab reads well on screen, and whether the single-file Markdown rule (one card per `## `) suits real decks, needs the owner at a window. The `## ` rule is a choice made here; it is not in `docs/specs/interchange.md` before this change.

## Open Questions
None.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): criterion 1 MET: one Import... entry in the items toolbar's More... menu and in the stack page's More actions; no separate card, stack, note or question import buttons (/tmp/claude-0/audit/g5/0168-more-menu.png, /tmp/claude-0/audit/g5/0168-items.png). The entry is under an overflow menu, not top level. Criterion 2 MET for CSV, Markdown, JSON and PDF: "Looks like CSV or TSV cards" (/tmp/claude-0/audit/g5/0168-import-dialog.png), "Looks like Markdown cards (one per ## heading) or one note" (/tmp/claude-0/audit/g5/0168-import-md.png), "Looks like StudyFrog export (JSON)" (/tmp/claude-0/audit/g5/0168-import-json.png), "Looks like PDF document (one card per page)" (/tmp/claude-0/audit/g5/0168-dest-menu.png); a two-line-per-page PDF gave two cards. Anki (.apkg) NOT CHECKABLE: no sample package in the test vault. Criterion 3 MET: the Import into menu offers As the file says, Vault root (no stack), each stack and New stack... (/tmp/claude-0/audit/g5/0168-dest-menu.png). A CSV imported into a chosen stack nested its rows under it, and the menu then listed two look-alike stacks (BUG-0037). Status left at review: the Anki path is not exercised and BUG-0037 is open.

## Review (2026-11-22)

Review (2026-11-22): criterion 1 unticked. As written it asks for a top-level Import button. The app has none: the single Import is an entry in an overflow menu (items toolbar More..., stack page More actions; /tmp/claude-0/review-a/items-more2.png, /tmp/claude-0/review-a/stack-more.png). The duplicate buttons are gone. Whether an overflow entry counts is the owner's call. Criterion 2 MET: a minimal Anki package with no format chosen was detected and imported as two cards in a Physik stack (ctl import, notice "Imported: 2 created"; /tmp/claude-0/review-a/physik.apkg); the earlier audit saw the CSV, Markdown, JSON and PDF hints. The "Looks like Anki" hint text was not seen on screen. Criterion 3 MET: the Import into menu lists As the file says, Vault root (no stack) and the stacks (/tmp/claude-0/review-a/import-dest.png). Status stays review (criterion 1 open). BUG-0037 (nested stack) is still open.
