# Export to PDF

You can export a single note, card, or question, a selection of items, or a whole stack to a standalone, printable PDF
document. StudyFrog renders the Markdown directly into a paginated PDF with clickable links, syntax coloring, tables,
and document bookmarks.

## How to export

Open the **Export to PDF** dialog from:

- **Item page**: open the **...** (More actions) menu at the top right and choose **Export to PDF…**.
- **Stack page**: open the **...** (More actions) menu and choose **Export to PDF…** to export the stack and all its
  items (including sub-stacks and referenced items).
- **Library bulk bar**: select one or more items using their checkboxes in the library list, then click **Export to
  PDF…** in the bulk actions bar at the bottom.
- **Import and export**: on [[import-export|Import and export]], choose **PDF document** as the export format.

## Export options

The export dialog provides several layout and presentation options:

- **Page size**: choose between standard **A4** (210 × 297 mm) and **US Letter** (8.5 × 11 in).
- **Margins**: choose **Compact** (12 mm), **Normal** (20 mm), or **Wide** (28 mm) margins to suit printing or screen
  reading.
- **Base font size**: choose **Small** (9 pt), **Medium** (10.5 pt, default), or **Large** (12 pt).
- **Title page**: when enabled, prepends an elegant cover page showing the document title, item count, date, and
  author metadata.
- **Table of contents**: generates an automatic table of contents with page numbers and clickable links jumping to each
  heading.
- **Page numbers**: displays page numbers (such as "Page 1 of 6") in the running footer.
- **Include answers**: for flashcards and practice questions, choose whether to print the answers or leave them out to
  create blank study worksheets.

The dialog displays a **live preview** showing the estimated page count and rendered pages before you write the file.
Press **Export** and choose a target folder. Once written, a notification lets you open the PDF directly in StudyFrog's
built-in [[pdf|PDF viewer]] or reveal it in your file manager.

## Supported formatting

StudyFrog's PDF layout engine renders Markdown formatting faithfully:

- **Headings**: H1 through H6, styled with proportional typography and included in the PDF document bookmarks
  (table of contents in PDF readers).
- **Tables**: pipe tables with left, center, and right column alignment, styled headers, and alternating row shading.
- **Code blocks**: syntax-highlighted code with language detection (Rust, Python, Java, C++, C, JavaScript, TOML, JSON,
  HTML, XML, CSS, SQL, and Shell).
- **Underline and colors**: `<u>underlined</u>` text and the six print-safe palette colors (`red`, `orange`, `green`,
  `blue`, `purple`, and `grey` via `<span style="color:...">`).
- **Task lists and lists**: bullet lists, numbered lists, and interactive task checkboxes (`- [ ]` and `- [x]`).
- **Blockquotes**: quoted passages with a left accent bar.
- **Bookmarks**: hierarchical PDF outline bookmarks generated from note headings for quick navigation.
- **Links**: clickable web addresses (`https://...`) and internal wikilinks (`[[Note Title]]` or `[label](target)`)
  that jump to the target page inside the exported PDF.
- **Pictures**: image attachments (`![alt](path)`) referenced from the vault or `assets/` folder, presented in captioned
  figure blocks.

## What is not supported

To keep layout fast, predictable, and self-contained, certain advanced features are not supported:

- **Arbitrary HTML and CSS**: only `<u>` and `<span style="color:...">` with the six palette hues are interpreted. Any
  other HTML tags are displayed as plain text or ignored.
- **Multi-column layouts**: content flows in a single continuous column; side-by-side text columns are not supported.
- **Complex non-Latin font fallback**: the built-in typography supports Latin-1 / Western European character sets
  with TrueType unicode mapping; complex scripts requiring bidirectional text or OpenType shaping (such as Arabic,
  Hebrew, or CJK ideographs) are not currently supported.

## Related pages

[[pdf]], [[import-export]], [[item-page]], [[stack-page]], [[faq]].
