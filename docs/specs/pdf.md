# PDF files

Implements ADR 0010 (epic EPIC-022). PDFs live inside the vault folder, anywhere under it. StudyFrog reads them,
searches them, lets the user mark them up in a side file, writes new rearranged copies and turns their text into notes.
It never changes a PDF in place.

## Libraries and where they are used
All PDF code is in the `studyfrog-pdf` crate; the rest of the workspace sees plain data. `hayro` draws pages (pure
Rust, no unsafe), `pdf_oxide` reads text, words, pictures and the outline and rearranges pages, `lopdf` prunes unused
objects, writes properties and writes annotations. `pdfium` (GPL-3.0) and `mupdf` (AGPL-3.0) were excluded by licence.
Measurements are in `docs/research/pdf-library-spike.md`.

## Limits (TASK-0113)
File 200 MB, 5 000 pages, a drawn page 36 million pixels, a page at most 14 400 points on a side (200 inches), 30 s
for one call and 180 s for a whole file. A PDF over a limit is refused with a sentence naming it. Every library call
runs on its own thread under `catch_unwind` and a timeout, so a panic comes back as "the PDF library crashed" and a
hang as "took too long"; the window never waits for either. Broken files (not a PDF, truncated, looping page tree,
bad cross-reference table) give an error, never a crash; this is tested in `crates/studyfrog-pdf/tests/hostile.rs`.
JavaScript and launch actions are never run, and links inside a PDF are not followed.

## The index (TASK-0111)
Schema v11: `pdf_doc` (path, size, mtime, hash, pages, title, error, state of the notes file), `pdf_page` (text per
page, whether the page is only a picture) and `pdf_mark` (the text of the user's marks). The index is derived; the files
are authoritative. A pass lists the `.pdf` files (hidden folders and symlinks skipped), reads at most 6 changed files,
hashes them and extracts text only when the hash changed; the rest wait for the next poll. A file modified less than
200 ms ago waits. Search is a case-insensitive substring search over titles, page text and marks (`LIKE`, wildcards
escaped), titles first.

## The viewer (TASK-0112)
One page at a time, drawn on the worker thread at a scale that is a multiple of 0.05 and cached (40 pictures including
thumbnails; thumbnails go first). Zoom: fit width, fit page, 25 to 400 %. Outline, page thumbnails, search with every
hit highlighted and a way through the hit pages, word selection by dragging a rectangle, copy. The text of the shown
page is in the window as plain text for screen readers. Coordinates: words come in points from the top left of the page.

## Highlights and notes (TASK-0114)
`name.pdf.notes.toml` beside the PDF. One `[[mark]]` per mark: `id`, `page` (1-based), `kind` (`highlight`, `note`),
`color` (`yellow`, `green`, `blue`, `pink`, `orange`), `rects` (`[x, y, width, height]` in points from the top left),
`text` (what a highlight covers) and `note`. At most 5 000 marks. The file is written atomically and removed when the
last mark goes; a damaged file is reported with the line and the viewer still opens. A mark is found again after the PDF
is replaced by a revision: if its text is no longer on its page, the first page that holds it is offered. The file is
part of the vault, so backups include it, and the search covers the text and the note.

## New files from old ones (TASK-0115, TASK-0116)
The plan is a list of (source file, page, extra rotation). Pages can be turned by quarter turns, moved, dropped and
repeated; other PDFs can be appended; the original can be split by page ranges into one file per range
(`name-pages-1-3.pdf`). The result is always a new file inside the vault under a name that does not exist yet.
"Write my highlights and notes" writes the first file's marks as standard annotations (`Highlight` with quad points,
`Text`) onto the pages they belong to after the move. Marks on turned pages are left out, and the message says how many.
Title, author and subject can be set. **Not carried:** the outline, links, forms and the original's own annotations
(pages are copied, unused objects are pruned), and marks of appended files.

## Notes from text (TASK-0117)
The text of a page, a page range (at most 100 pages) or a selection becomes a Markdown note in a stack. Headings come
from the font sizes the library detects; lines of a paragraph are joined and a word split by a hyphen at a line end is
rejoined when the next line goes on in lower case. Lists, quotes and tables are left as they are. Columns are read in
the order the library gives them and can interleave on complex layouts. The note has `source: "path.pdf#page=N"` in its
front matter; the item page shows "Open source" and the viewer lists the notes made from the file.

## Pages without text (TASK-0118)
A page with no text and at least one picture is a scan. It is flagged in the index (`image_only`), the viewer says how
many pages cannot be searched or selected, and a note made from such a page says so. **OCR (reading text from pictures)
is deferred and not part of this story.**

## The library (TASK-0121)
"Add PDF ..." (stack page and Create dialog) copies a file into the stack under a name of letters, digits, dots, dashes
and underscores (a taken name gets `-2`, `-3`); the file must start with `%PDF-` and be at most 200 MB. Renaming moves
the notes file and replaces the old path by the new one in every Markdown file of the vault that mentions it (source
keys and links); the next pass indexes the new name. `[text](path.pdf#page=N)` and `![](path.pdf#page=N)` (shown as a
link) open the viewer at page N; a link with a scheme (`https://`) is an ordinary web link. The home page lists the five
PDFs that changed most recently.

## The assistant (TASK-0120)
Pages of one PDF (at most 20, 3 000 characters each, inside the usual 30 000 character budget) are added to the request as
items with ids `pdf:path#page=N`. A proposed card or question may carry `source` with one of those ids; an id the model
was not shown is dropped. The accepted item gets `source: "path#page=N"`.

## Benchmarks and tests
`cargo bench -p studyfrog-pdf --bench pdf` (500 pages; numbers in `docs/contributing/BENCHMARKING.md`). Headless tests:
`crates/studyfrog-ui/tests/pdf_*.rs`, including the accessibility audit `pdf_a11y.rs`.
