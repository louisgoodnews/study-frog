---
id: TASK-0197
type: task
title: "Tables and more Markdown in the editor and the preview"
status: done
epic: EPIC-026
parents: [STORY-0074]
estimate: 5
labels: [m11, ui, md]
acceptance_criteria:
  - "The parser reads pipe tables (with column alignment) and the preview draws them with a header row"
  - "The toolbar has a Table button, and buttons for strikethrough, underline and text colour next to bold and italic"
  - "Bold, italic, strikethrough, quotes (also nested) and inline images keep working and are checked in the preview and in the cards of a run"
  - "Everything round-trips through save: the file stays readable in other Markdown tools"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Tables are missing and the toolbar lacks some formatting the owner wants. Owner's list (2026-11-03): tables, italic, bold, underline, overstrike (strikethrough), text colour, code with specific languages, quotes, inline images, an automatic table of contents.

## Acceptance Criteria
- [x] The parser reads pipe tables (with column alignment) and the preview draws them with a header row
- [x] The toolbar has a Table button, and buttons for strikethrough, underline and text colour next to bold and italic
- [x] Bold, italic, strikethrough, quotes (also nested) and inline images keep working and are checked in the preview and in the cards of a run
- [x] Everything round-trips through save: the file stays readable in other Markdown tools

## Design Notes
Done as `Block::Table` and `Block::Toc` in `studyfrog-md` (`blocks_table.rs`, `blocks_text.rs`), `view/preview_rich.rs` (drawn as a grid; a centred or right-aligned cell is one text, so links in such a cell are not clickable), and the toolbar buttons (`md_edit.rs`, `md_edit_table.rs`). The open question about inline images (pasting, width, alignment, a caption) is left for later: pictures work as before.

Already there: bold, italic, code, quote, links, wikilinks, rule and pictures (`md_edit.rs`, `blocks.rs`), strikethrough in the preview but not on the toolbar. New: `Block::Table` (header, alignments, rows) in `studyfrog-md`, drawn as a grid; the toolbar buttons. Underline and colour are not Markdown: write them as the small HTML a reader of the file can still understand (`<u>text</u>` and `<span style="color:#c0392b">text</span>`), accept only that whitelist and show anything else as text. Code blocks with languages are TASK-0214, the table of contents TASK-0216, underline and colour details TASK-0215. Inline images exist (`![alt](path)` and the Picture button); see the question.

## Dependencies
None.

## Open Questions
Inline images are there today. What is missing: pasting from the clipboard, a width, alignment, a caption?
