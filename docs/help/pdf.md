# PDFs

PDF files in your vault folder are part of your library. You can read them, search them, mark them up, rearrange
their pages into new files and turn their text and pictures into notes and cards. StudyFrog never changes a PDF in
place.

## Getting a PDF into the vault

- Put the file anywhere in the vault folder, or
- press **Add PDF ...** on a stack's List tab (under **PDF files**) or in the Create dialog's **More** menu. The file
  is copied into the stack under a safe name.

StudyFrog reads the text of each new or changed PDF in the background, a few files at a time.

## Opening a PDF

- From the search in the top bar: it finds PDFs by title, by the text of their pages (showing the page) and by your
  own highlights and notes.
- From the **PDF files** list of a [[stack-page|stack page]], or the recent PDFs on [[home|Home]].
- From a link in a note: `[page 3](file.pdf#page=3)` opens the viewer at page 3.
- From **Open source** on the item page, or the **Source** link while rehearsing, for an item made from a PDF.

## Reading

- **Previous** and **Next**, or type a **Page number**.
- Zoom with **-** and **+** (25 to 400 %), **Fit width** or **Fit page**.
- **Find in this PDF**: every hit on the page is highlighted; **Previous hit** and **Next hit** go through the pages
  with hits.
- Drag a rectangle over words to select them; **Copy selection** copies the text.
- **Text of this page** gives the page as plain text, also for screen readers.
- The side panel has **Outline**, **Pages** (thumbnails), **Notes** and **Extract**; **Hide** closes it.
- **Back to library** leaves the viewer.

## Highlights and notes

In the **Notes** panel:

- choose a colour under **Mark with** (yellow, green, blue, pink, orange),
- select words and press **Highlight selection**, or press **Place note** and click the page where it belongs,
- the list shows every mark; click one to jump to it, edit its note, or **Delete** it.

Marks are kept in a plain text file next to the PDF, `name.pdf.notes.toml`. They are searchable and part of backups.
If you replace the PDF with a new version, a mark whose text moved is offered on its new page.

A highlight or a selection can become a **Card** or a **Question** with a link back to the page.

## Extracting

In the **Extract** panel:

- **Make a note from the text** of this page, a page range or the selected words, in a stack you choose. The note
  keeps a link back to the page.
- **Save the pictures of this page**, or take a **Snapshot of a region** (drag a rectangle, for a figure or a
  formula). Pictures go into the vault's `assets/` folder and can be put at the end of the note.
- **Card from selection** and **Question from selection**.
- **Ask the assistant about these pages**: opens the [[assistant]] with the pages as extra reading. Nothing is sent
  until you press Send.
- **Made from this PDF** lists the notes made from the file.

## Arranging pages into a new file

**Arrange pages ...** opens a grid of the pages. **Turn left**, **Turn right**, **Move earlier**, **Move later** and
**Delete page**, with **Undo** and **Start over**. **Add a PDF ...** appends another file. **Split into files** cuts
the original at page ranges (for example 1-3, 4-9). Give the new file a name and, if you like, a **Title**,
**Author** and **Subject**, tick **Write my highlights and notes into the copy** to add them as annotations other
PDF readers show, and press **Save as new file**. The original is never changed.

## Links inside a PDF

Links are outlined on the page. A link to another page is followed with a click. A web or mail address is shown to
you first and opens in your browser only if you press **Open**. Nothing else a link asks for is ever run.

## Renaming

**Rename** in a stack's PDF list renames the file, moves its notes file and updates every note, card and link that
names it.

## Limits

- **Scanned pages** (pictures without text) cannot be searched, selected or turned into notes. StudyFrog says how
  many pages are affected. Reading text from pictures (OCR) is not available.
- Files over 200 MB or 5 000 pages, and very large pages, are refused with a message that names the limit.
- Encrypted PDFs, forms, signatures and editing text inside a PDF are not supported.
- A rearranged copy does not keep the outline, the links or the original's own annotations. Marks on turned pages
  are left out of an annotated copy, and the message says how many.

## Related pages

[[library]], [[create-dialog]], [[faq]], [[pdf-export]].
