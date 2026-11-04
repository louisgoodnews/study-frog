# StudyFrog 0.4.0

Export notes to PDF, study diary and self-assessment calibration, Obsidian-style folders view, and multi-stack item references. The full list is in `CHANGELOG.md` under `[0.4.0]`.

## What is new
- **Export notes to PDF (STORY-0082, EPIC-026).** Export a single note, card, or question, a chosen selection, or a whole stack to a standalone, publication-ready PDF file directly from the app.
  - Built-in layout engine paginating Markdown into pages with wrapped text, proportional headings, pipe tables with column alignments, syntax-highlighted code blocks, blockquotes, task lists, underline, and six print-safe palette hues.
  - Produces searchable PDFs with TrueType `/ToUnicode` CMaps, document outline bookmarks (`/Outlines`), clickable external web links, and internal wikilinks between notes.
  - Modal export dialog with configurable page size (A4, US Letter), margins (compact, normal, wide), font sizes, cover title page, table of contents with page references, page numbers, and card/question answer toggles.
  - Live page count estimation and multi-page preview before exporting.
  - Accessible entry points on the item page, stack page, library bulk bar, and Transfer screen; completion notification opens the exported file directly in StudyFrog's built-in PDF viewer.
  - Comprehensive documentation, help topic (`[[pdf-export]]`), FAQ entries, and full showcase example in `examples/pdf-export/complete-note.md`.
- **Study Diary and Rehearsal Reflection (EPIC-024).** Log daily study thoughts, track 13 standard self-assessment scales (energy, focus, comprehension, retention), configure custom reflection scales, and answer quick post-rehearsal check-ins (`Esc` to dismiss).
- **Calibration Reports (TASK-0174).** The Reports screen now features a dedicated "Self-assessment" tab correlating subjective self-ratings against objective review accuracy and item familiarity to detect overconfidence or underconfidence bias.
- **Folders and Items Views (STORY-0076).** Obsidian-modelled file tree explorer in the sidebar with create, inline rename (`F2`), drag-and-drop moves, folder delete, cross-stack move confirmation dialogs, and rollback snapshots.
- **Multi-Stack References and Independent Memory (STORY-0079).** Items can be referenced by multiple stacks (`referenced_by`), each stack maintaining its own independent review schedule, due date, and familiarity rating.
- **Enhanced Markdown Editing (STORY-0074).** Pipe tables with alignment, syntax highlighting across 13 programming languages, inline underline and palette color spans, auto-expanding list continuation, and text macros (`{table}`, `{code}`, `{toc}`).
- **Interchange formats.** Export and import now support JSON, TOML, and RON with identical data fidelity.

## Verification
- **Automated test suite:** Headless unit, service, and UI tests cover PDF layout, lopdf writer primitives, safe path handling, dialog interactions, notification routing, and text extraction round-trips.
- **Document element test (`crates/studyfrog-service/tests/pdf_export_docs.rs`):** Exports `examples/pdf-export/complete-note.md` and verifies all formatting elements (headings, 6 hues, underline, tables, code blocks, lists, task checkboxes, blockquotes, bookmarks, links, picture captions) are extracted cleanly.
- **Manual window verification:** Export dialog opened in a real window environment on Linux X11/Wayland (`studyfrog --screenshot` and manual CLI run), generating a verified 4-page PDF with title page, table of contents leader lines, styled typography, code blocks, and outline bookmarks.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.4.0-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.4.0-x86_64.AppImage
./StudyFrog-0.4.0-x86_64.AppImage
```

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.4.0-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.4.0-x86_64.tar.gz
cd StudyFrog-0.4.0
./bin/studyfrog
```
