# Markdown to PDF export

Implements ADR 0012 (epic EPIC-026, story STORY-0082, TASK-0228). StudyFrog exports notes, cards,
questions, selections, or entire stacks into clean, self-contained, printable PDF documents.
The visual layout and styling match the application's editor preview (`crates/studyfrog-ui/src/view/preview.rs`
and `preview_rich.rs`).

## Architecture and Seams

All PDF layout and writing code lives inside `crates/studyfrog-pdf` behind pure data interfaces:
1. **Layout engine (`studyfrog-pdf::write::layout`):** A pure function with no I/O that turns the parsed Markdown
   blocks of `studyfrog-md` (`Vec<studyfrog_md::blocks::Block>`), document options, and font metrics into
   paginated layout structures (lines, styled text spans, table cells, code boxes, images, and link rectangles).
2. **PDF writer (`studyfrog-pdf::write::pdf`):** Serializes the laid-out pages into valid PDF bytes using `lopdf`,
   embedding TrueType font subsets with `ToUnicode` CMaps, image XObjects, clickable link annotations,
   and document outlines (bookmarks).
3. **Use case layer (`studyfrog-store::export_pdf`):** Reads note files and vault assets, resolves wikilinks
   and images, and passes self-contained data structures into the layout engine (TASK-0230).

Every call runs on a worker thread under `catch_unwind` with a timeout and resource limits (TASK-0113).

## Supported Markdown Syntax

The export supports the exact Markdown dialect parsed by `studyfrog-md` (CommonMark plus extensions from
TASK-0197, TASK-0198, TASK-0214, TASK-0215, and TASK-0216):

| Element | Source Markdown | Rendered Appearance in PDF |
|---|---|---|
| **Heading 1** | `# Title` | 24 pt bold, 18 pt space before, 8 pt space after, keep-with-next |
| **Heading 2** | `## Section` | 18 pt bold, 14 pt space before, 6 pt space after, keep-with-next |
| **Heading 3** | `### Subsection` | 14 pt bold, 10 pt space before, 4 pt space after, keep-with-next |
| **Heading 4-6** | `#### Sub-subsection` | 11 pt bold, 8 pt space before, 3 pt space after, keep-with-next |
| **Paragraph** | Plain prose | 10 pt regular, 14 pt line height, 8 pt space between paragraphs |
| **Bold** | `**text**` or `__text__` | Proportional bold font |
| **Italic** | `*text*` or `_text_` | Proportional italic font |
| **Strikethrough** | `~~text~~` | Centred strikeout line (0.6 pt) through the text run |
| **Inline code** | `` `code` `` | Monospace font (8.5 pt) with light grey background pill (`#f1f3f5`) |
| **Underline** | `<u>text</u>` | Solid stroke (0.6 pt) 1.5 pt below the baseline |
| **Text colour** | `<span style="color:#...">` | Whitelisted 6 hues from `studyfrog_md::hue::Hue` (print-safe palette) |
| **Text macros** | `{color:NAME}`, `{toc}`, `{image}` | Expands to corresponding Markdown/HTML tokens (TASK-0198) |
| **Blockquote** | `> quote text` | Indented 16 pt, 2 pt left vertical rule in `#bdc3c7`, italicized |
| **Bulleted list** | `* item` or `- item` | Indented 18 pt per level, bullet glyph `•`, hanging indent |
| **Numbered list** | `1. item` | Indented 18 pt per level, formatted number `N.`, aligned numerals |
| **Task list** | `- [ ] todo` / `- [x] done` | Unchecked box `☐` (or square outline) / checked box `☑` |
| **Thematic rule** | `---` or `***` | 0.5 pt solid line in `#cbd5e1`, 12 pt margin above and below |
| **Pipe table** | `\| A \| B \|` | Header row in bold with `#f1f5f9` fill, 0.5 pt grid borders, alignments |
| **Code block** | ` ```lang ... ``` ` | Monospace (8.5 pt) in container box (`#f8fafc`), syntax highlighted |
| **Table of contents** | `[TOC]` or `{toc}` | Hierarchical list of headings with dotted leaders and page links |
| **Wikilink** | `[[Target]]` or `[[Target\|label]]` | Styled link; internal `GoTo` if target is in export, styled text otherwise |
| **External URL** | `[label](https://...)` | Blue text (`#2471a3`), clickable PDF `URI` link annotation |
| **Picture** | `![alt](assets/img.png)` | Scaled to column width, aspect ratio preserved, centered |

### Text Colours

Colours use the 6 predefined hues from `studyfrog_md::hue::Hue`. To ensure clear readability on white
paper, print-safe hex values with sufficient contrast (WCAG AA compliant) are emitted:
- `red`: `#c0392b`
- `orange`: `#d35400`
- `green`: `#1e8449`
- `blue`: `#2471a3`
- `purple`: `#7d3c98`
- `grey`: `#596264`

Any color specification outside this whitelist is rejected by the Markdown parser and rendered as raw text.

### Tables

Tables (`Block::Table` from TASK-0197) render as follows:
- Header row with bold text, padded with 4 pt vertical and 6 pt horizontal spacing, with light grey background fill (`#f1f5f9`).
- Column alignments (`Align::Left`, `Align::Center`, `Align::Right`) apply to both header and body cells.
- Alternating zebra row shading (`#ffffff` and `#f8fafc`) for rows to assist visual tracking.
- Subtle horizontal cell borders (0.5 pt in `#e2e8f0`).
- Text wraps inside cell boundaries. If a table exceeds available vertical space on a page, it splits
  across page boundaries, cleanly repeating the table header row at the top of the next page.

### Syntax-Highlighted Code Blocks

Code blocks (`Block::Code` from TASK-0214) render as follows:
- Set in a dedicated box with a 0.5 pt border (`#e2e8f0`) and subtle background fill (`#f8fafc`).
- Font: Monospace font (8.5 pt, 11 pt line height).
- Tokens are colored according to their `Kind` from `studyfrog_md::highlight`:
  - `Keyword`: Dark blue (`#0550ae`, bold)
  - `String`: Dark green / teal (`#0a7a3c`)
  - `Comment`: Muted grey (`#6e7781`, italic)
  - `Number`: Dark purple / blue (`#0550ae`)
  - `Plain`: Dark charcoal (`#1f2328`)
- Long code blocks that exceed page boundaries split between lines with a continued-block visual cue;
  individual lines do not truncate and wrap gracefully if wider than the content box.

### Table of Contents

The `[TOC]` block (TASK-0216) generates a document outline:
- Scans headings in the exported document.
- Renders each entry with indentation corresponding to heading level (`level - 1`).
- Displays the heading title, dotted leader lines (`. . . . . .`), and the target page number.
- Each entry is an interactive clickable PDF link annotation pointing directly to the heading on its page.
- If the document contains no headings, renders: *"This document has no headings yet."*

### Wikilinks and Pictures

- **Wikilinks:** If the target note is included in the current PDF export (such as during a stack export
  or multi-item export), the link is rendered in link blue (`#2471a3`) and carries an internal PDF `/GoTo`
  link annotation jumping to that item's section. If the target note is outside the export, the link renders
  as the formatted target title with a subtle badge or footnote indicating it is an external vault note.
- **Pictures:** Images (`![alt](assets/img.png)`) are read from the vault `assets/` directory (JPEG, PNG, GIF).
  Images are scaled proportionally to fit the printable width (at most 100% of printable content width,
  never upscaled beyond native pixel resolution at 150 DPI). Images are centered on their own block.
  If an image file is missing, corrupted, or uses a disallowed path, a neat placeholder box is rendered
  showing a broken image icon and the text `[Missing picture: <alt>]`, and the export continues.

## Visual Styling and Page Geometry

### Page Sizes and Margins

StudyFrog supports the two standard ISO and US document paper sizes:
- **ISO A4:** 595.28 x 841.89 points (210 x 297 mm) — default for metric locales.
- **US Letter:** 612.0 x 792.0 points (8.5 x 11.0 in) — default for North American locales.

**Margins:** Standard margins are 54 points (0.75 in / 19.05 mm) on all four sides:
- Printable content area for A4: 487.28 pt wide x 733.89 pt high.
- Printable content area for US Letter: 504.0 pt wide x 684.0 pt high.
- Top margin reserves 36 pt for the running header; bottom margin reserves 36 pt for the running footer.

### Font Families and Clean Typography

To avoid dependencies on host system fonts while maintaining crisp typography across platforms:
- **Proportional body font:** Clean open-source sans-serif font (such as *Inter* or *Liberation Sans* under
  SIL Open Font License / Apache 2.0). Bundled variants: Regular, Bold, Italic, and Bold-Italic.
- **Monospace font:** Clean open-source monospace font (such as *JetBrains Mono* or *Liberation Mono* under
  OFL / Apache 2.0). Bundled variants: Regular and Bold.
- **Typographic scale:**
  - Document Title (Cover): 32 pt, bold
  - Heading 1: 24 pt, bold
  - Heading 2: 18 pt, bold
  - Heading 3: 14 pt, bold
  - Heading 4-6: 11 pt, bold
  - Body text: 10 pt, regular (line height 14 pt)
  - Code / Table text: 8.5 pt (line height 11 pt)
  - Header / Footer / Metadata: 8 pt, regular (muted grey `#64748b`)

### Running Headers, Footers, and Page Numbers

Running headers and footers provide context throughout multi-page documents:
- **Header:**
  - Displays document/stack title or current primary section heading.
  - Position: 36 pt from the top edge.
  - Separator: 0.5 pt hairline rule in `#e2e8f0`.
  - Suppressed on title/cover pages.
- **Footer:**
  - Displays vault name or export date on the left.
  - Displays page numbers on the right in the standard format: `Page X of Y` (or simply `X`).
  - Position: 36 pt from the bottom edge.
  - Separator: 0.5 pt hairline rule in `#e2e8f0`.
  - Suppressed on title/cover pages.

## Selectable, Searchable, and Accessible Text

Generated PDFs must meet modern standards for accessibility and text processing:
1. **ToUnicode CMaps:** All embedded TrueType font subsets include a `/ToUnicode` mapping stream. Every glyph
   index maps directly to its canonical UTF-8 / UTF-16 code point. Text can be searched, selected, and copied
   cleanly in any PDF reader (including StudyFrog's own PDF viewer and `pdf_oxide`), without producing garbled
   characters or missing spaces.
2. **Document Metadata:** Standard PDF `/Info` dictionary contains `/Title`, `/Author` (vault owner or "StudyFrog"),
   `/Subject`, `/Creator` ("StudyFrog"), and `/CreationDate` (RFC 3339 formatted).
3. **Document Outlines (Bookmarks):** A hierarchical `/Outlines` dictionary is written at the PDF root,
   mirroring H1, H2, and H3 headings so users can navigate the document via the PDF reader's sidebar.
4. **Logical Structure (Tagged PDF):** PDF structure elements (`/StructTreeRoot`) identify sections, headings,
   paragraphs, tables (`/Table`, `/TR`, `/TH`, `/TD`), and images (`/Figure` with `/Alt` text) for screen
   readers and automated indexers.

## Single Item vs Whole Stack Export

### Single Item Export (Note, Card, Question)

- **Cover page:** Suppressed. The document begins immediately on page 1 with the item's title as Heading 1.
- **Metadata header:** A compact metadata block below the title lists tags, primary stack, and last updated date.
- **Notes:** Flowing body Markdown.
- **Cards and Questions:**
  - Front prompt: Rendered in full with normal formatting.
  - Dividing separator: Decorative callout card or badge (`Prompt / Answer`).
  - Back / Reference answer: Included by default, with an export option to omit answers (for blank practice worksheets).
  - Cards with `reversible: true` display a subtle indicator icon (`⇄ Reversible`).

### Whole Stack Export (Multi-Item or Stack)

- **Cover Page:**
  - Prominent stack title and description (from `_stack.md`).
  - Metadata block: Total item count, note count, card count, question count, export date, and vault name.
  - Clean page break following the cover page.
- **Global Table of Contents:**
  - Lists all included notes, sub-stacks, and items with hierarchical indentation and target page numbers.
  - Interactive clickable links jumping directly to each item's starting page.
- **Layout Flow:**
  - Each primary note or sub-stack section starts on a fresh page.
  - Cards and questions can flow consecutively within their parent stack section or break cleanly as configured.
- **Internal Wikilinks:**
  - Wikilinks referencing items within the stack resolve to internal PDF `/GoTo` links to those items' sections.

## What is Left Out or Sanitized

Security and visual cleanliness dictate strict sanitization during export:

1. **HTML Elements and Scripts:**
   - Only the whitelisted HTML tags (`<u>...</u>` and `<span style="color:#...">...</span>`) are parsed and rendered.
   - All script tags (`<script>`), applets, iframes, `<object>`, and event handlers are strictly discarded.
   - Any unknown HTML tag is escaped and printed as literal text.
2. **Dangerous PDF Actions:**
   - Generated PDFs contain only safe `/URI` links (`http://`, `https://`, `mailto:`) and internal `/GoTo` links.
   - Interactive forms, JavaScript actions (`/JS`), application launch actions (`/Launch`), and external file references
     (`/File`) are completely forbidden.
3. **Filesystem and Resource Safety:**
   - Image paths must be relative to the vault and resolve strictly within `assets/`.
   - Paths containing `..`, absolute root paths (`/`), or UNC network shares are rejected.
   - Missing or oversized images do not crash the engine; they are replaced with a clear placeholder notice.
4. **Vault Internal Data:**
   - Internal database keys (ULIDs, raw SQLite row IDs), spaced-repetition memory weights (`mem_state`, `sm2_state`),
     and rehearsal logs are omitted from standard reading exports.
