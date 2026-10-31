# StudyFrog 0.2.0

PDF files are now part of the vault. They live in the vault folder like everything else; StudyFrog never changes a
PDF in place.

## What is new
- **Read.** Open a PDF from the search, a stack page, the home page, a note's link or a card's Source link. One page at a
  time with fit width, fit page and 25 to 400 % zoom, an outline, page thumbnails, search inside the file with every hit
  highlighted, selecting words by dragging, copy, and the text of the page as plain text for screen readers.
- **Search.** The global search finds PDFs by title, by the text of their pages (with the page) and by your own notes.
  Text is extracted once per change, a few files at a time in the background.
- **Mark up.** Highlights in five colours and notes pinned to a point, with a list to jump to, edit and delete them.
  They are stored in `name.pdf.notes.toml` next to the PDF, so they are plain text, travel with backups and survive a
  rename of the folder; if the PDF is replaced by a new revision, a mark whose text moved is offered on its new page.
- **Rearrange.** Turn, move, drop and repeat pages with undo, append other PDFs, split by page ranges, and write the
  result as a new file with new title, author and subject and, if you want, your highlights and notes as standard
  annotations other readers show. The original is never touched.
- **Extract.** The text of a page, a range or a selection becomes a Markdown note with a link back to the page;
  pictures on a page and any dragged rectangle (a figure, a formula) are saved into `assets/` and can be put in the note.
  A highlighted passage or a selection becomes a card or a question. The assistant can read chosen pages (you see
  the text first and nothing is sent until you press Send) and propose cards that name their page.
- **Links.** Links inside a PDF are outlined on the page. A link to another page is followed with a click; a web or mail
  address is shown to you and opened in the browser only after you press Open. Nothing else a link can ask for is ever run.
- **Organise.** "Add PDF ..." copies a file into a stack; renaming a PDF moves its notes file and rewrites every note,
  card and link that names it. `[text](file.pdf#page=3)` opens the viewer at that page.

## Not included
- **Reading text from pictures (OCR).** Scanned pages are flagged and said to have no text; this is deferred.
- Editing text or images inside a PDF, forms, signatures, encrypted PDFs, links that launch programs or run scripts.
- The outline, links and the original's own annotations are not carried into a rearranged copy; marks on turned
  pages are left out of an annotated copy (the message says how many).

## Safety
Files over 200 MB or 5 000 pages, pages over 200 inches, drawings over 36 million pixels and calls over 30 s are
refused with a sentence that names the limit. A crash or a hang inside the PDF code is contained on its own thread.
Broken files (truncated, looping page tree, wrong cross-reference table, random bytes) are tested to give an error.

## What was checked how
- **By tests only (headless):** the viewer screens and panels, selection, marks, arranging, extraction, library, home
  and accessibility names (`crates/studyfrog-ui/tests/pdf_*.rs`), the index and the file operations
  (`crates/studyfrog-service/tests/pdf*.rs`), the library code and hostile files (`crates/studyfrog-pdf/tests`).
- **Measured:** 500 pages: structure 2.1 ms, text of all pages 374 ms, one page with word boxes 52 ms, drawing a page
  6 ms, rewriting all pages 26 ms (`docs/contributing/BENCHMARKING.md`). Library choice: `docs/research/pdf-library-spike.md`.
- **Not looked at in a real window:** how the viewer, the thumbnails, the highlights and the arrange grid look and feel
  with a real mouse. Not tried with real-world PDFs beyond the generated ones and the spike's samples. Rotated pages
  in annotated copies, and PDFs with non-embedded fonts, are the places most likely to show problems.

Format and limits in detail: `docs/specs/pdf.md`.
