# ADR 0012: Markdown to PDF writer

- Status: **Accepted** (2026-11-04, the owner's decisions; the wording is Claude's and the owner may amend it)
- Date: 2026-11-04

## Context

STORY-0082 adds a feature requested by the owner: notes, cards, questions, selections, or entire stacks
can be exported from StudyFrog to PDF.

StudyFrog already has a PDF crate (`crates/studyfrog-pdf`, ADR 0010) that draws, inspects, extracts,
and rearranges PDFs using `hayro`, `pdf_oxide`, and `lopdf`. However, creating a new PDF document from
Markdown requires a layout engine and a PDF document writer.

The system constraints are:
1. **Pure Rust, no native binaries to ship:** No WebKit, Chromium, headless browsers, or C/C++ libraries.
   Must compile cleanly within `cargo build` and keep the AppImage lightweight (~11 MB today).
2. **Strict license compatibility:** MIT OR Apache-2.0. No copyleft (GPL, AGPL) dependencies.
3. **Fidelity with StudyFrog's preview:** Markdown formatting (tables with column alignment from TASK-0197,
   code blocks with language token highlighting from TASK-0214, underline and 6 text hues from TASK-0215,
   table of contents from TASK-0216, task lists, blockquotes, and wikilinks) must render consistently
   with `crates/studyfrog-ui/src/view/preview.rs`.
4. **Searchable and accessible output:** Text must be selectable and searchable with valid Unicode mappings
   (`ToUnicode`), bookmarks (outlines), and document metadata.
5. **Architectural seam:** The converter lives behind `studyfrog-pdf`. The layout pass must be a pure
   function with no I/O (TASK-0229).

## Options

Three primary approaches in the Rust ecosystem were evaluated:

### Option A: In-house layout engine and PDF writer on top of `lopdf` (already in `studyfrog-pdf`) or `printpdf`
- **Architecture:** `studyfrog-md` parses Markdown into structured blocks (`Vec<Block>`) and spans (`Span`).
  A pure in-house layout module (`studyfrog-pdf::write::layout`) calculates line wraps, block heights,
  table grids, and page splits. A writer module (`studyfrog-pdf::write::pdf`) builds PDF objects (`Document`,
  `Pages`, `Contents` streams with PDF text and graphic operators, `Font` with `ToUnicode`, `Annot` for links,
  and `Outlines`) using `lopdf`.
- **Dependencies:** 0 new external dependencies. `lopdf` (0.46) is already vetted and in `Cargo.lock` (ADR 0010).
- **Binary footprint:** Tiny (+ ~150 to 250 KB for layout logic and TrueType font metrics).
- **Licence:** MIT OR Apache-2.0 (`lopdf` is MIT).

### Option B: High-level PDF document crate such as `genpdf` (or `printpdf` document wrapper)
- **Architecture:** Uses `genpdf` (built on `printpdf` and `rusttype`) to handle document composition,
  margins, paragraphs, tables, and page breaking.
- **Evaluation:** `genpdf` is largely abandoned (v0.2.0 released in 2021 with minimal subsequent activity).
  Its styling model is rigid: an entire paragraph or cell has a single style, making heterogeneous inline spans
  (e.g., combining bold, italic, underline, and one of the 6 text hues in one line) difficult without low-level
  workarounds. It lacks native support for code syntax tokens, PDF outline bookmarks, and tagged PDF accessibility.
- **Binary footprint:** Moderate (+ ~1.2 MB including `rusttype` and `printpdf`).
- **Licence:** MIT.

### Option C: `typst` crate / `typst-as-a-library` typesetting engine
- **Architecture:** Transforms StudyFrog Markdown into Typst syntax or Typst AST, using the `typst` typesetting
  engine to render state-of-the-art PDF documents with professional hyphenation, justification, font fallback,
  and PDF/A compliance.
- **Evaluation:** Typst produces peerless typographic quality. However, it is an entire compiler and runtime
  suite. Adding `typst` as a crate dependency pulls in over 110 transitive crates (including `comemo`, `hayagriva`,
  `roxmltree`, `ecow`, etc.) and adds 20 to 35 MB to the stripped binary size, more than tripling the size of
  the StudyFrog AppImage. Clean build times increase by several minutes. Furthermore, mapping StudyFrog's
  Markdown AST to Typst introduces an impedance mismatch that duplicates `studyfrog-md`'s parser and tokenizer.
- **Binary footprint:** Extremely heavy (+ 20 to 35 MB compiled binary size).
- **Licence:** Apache-2.0.

### Summary Comparison

| Criterion | Option A: In-house + `lopdf` | Option B: `genpdf` | Option C: `typst` library |
|---|---|---|---|
| **Output Quality** | High: tailored exactly to notes, tables, code boxes, and cards | Basic: inflexible inline styling, poor code block support | Exceptional: TeX-level microtypography and hyphenation |
| **Markdown AST Alignment** | 100% direct match with `studyfrog-md` `Block` and `Span` | Poor: requires awkward translation layers | Moderate: requires transpiling AST to Typst source |
| **New Dependencies** | **0** (`lopdf` already in workspace) | 2–4 (`genpdf`, `printpdf`, `rusttype`) | **110+** transitive crates |
| **Binary Footprint** | **+ ~200 KB** | + ~1.5 MB | **+ 25–35 MB** (triples AppImage) |
| **Compile Time Impact** | Negligible (< 3 s) | Small (< 10 s) | Severe (+ 2–4 min clean build) |
| **Fonts & Unicode** | Bundled free TrueType font + `ToUnicode` map; basic CJK box fallback | System or loaded TTF; limited font fallback | Full multi-font fallback, OpenType shaping, CJK/RTL |
| **PDF Bookmarks & Links** | Direct control over `/Outlines` and `/Annot` | Not supported in `genpdf` | Supported |
| **Licence** | MIT OR Apache-2.0 | MIT | Apache-2.0 |
| **Maintenance & Risk** | In-house code to maintain; 0 dependency decay | Crate is dormant/unmaintained | Heavy API churn across fast-moving Typst releases |

## Build vs Buy (Rule P10)

Rule P10 states:
> *"If we can write a better implementation than an external crate for our needs (smaller, faster, safer, or exactly
> fitted to our data model), we write it ourselves. A dependency must earn its place; record the comparison in an
> ADR or the task. Prefer well-maintained crates for hard, general problems (SQLite, GUI, ML runtimes, Markdown/CommonMark)."*

Applying Rule P10 to Markdown-to-PDF export:
1. **Document layout for structured notes is not an intractable problem:** StudyFrog notes are structured documents
   consisting of linear vertical stacks of blocks (headings, paragraphs, lists, tables, code blocks, images).
   They do not require TeX-style Knuth-Plass global line-breaking or complex multi-column floating math figures.
   A greedy line-breaking and block-pagination algorithm in pure Rust takes under 1,000 lines of straightforward code.
2. **Third-party crates fail the trade-off test:**
   - `genpdf` is dormant and cannot cleanly represent StudyFrog's inline styling (`Hue`, `underline`, syntax tokens).
   - `typst` would impose a massive 25+ MB binary bloat and 110+ dependencies for a simple document export feature,
     directly violating the project's goal of a compact, fast-starting desktop application.
3. **`lopdf` is already in the project:** `lopdf` is an established, well-maintained crate for low-level PDF
   object generation. It has already earned its place in `studyfrog-pdf` under ADR 0010. Building the layout
   in-house and emitting PDF objects via `lopdf` gives complete control over styling, link annotations, bookmarks,
   and ToUnicode accessibility at virtually zero binary cost.

## Decision

1. **Build the layout engine and PDF writer in-house on top of `lopdf`** inside `crates/studyfrog-pdf` (TASK-0229).
2. **Pure functional layout:** The layout engine is a pure function:
   ```rust
   pub fn layout(blocks: &[Block], options: &ExportOptions, fonts: &FontMetrics) -> DocumentLayout
   ```
   with zero I/O, deterministic output, and complete unit/property testability without touching the filesystem.
3. **PDF serialization via `lopdf`:** The writer translates `DocumentLayout` into a valid `lopdf::Document`,
   writing content streams (`BT`, `Tf`, `Tm`, `Tj`, `ET`, `re`, `f`, `Do`), page trees, `/Annots` (clickable web
   links, internal wikilink jumps, and TOC targets), `/Outlines` (document bookmarks from headings), and `/Info`.
4. **Font embedding and Unicode safety:**
   - Bundle a clean, free open-source sans-serif font (such as *Liberation Sans* or *Inter*) and monospace font
     (such as *JetBrains Mono* or *Liberation Mono*) under SIL Open Font License / Apache 2.0.
   - Embed TrueType font subsets with `/ToUnicode` CMaps so all text remains searchable, selectable, and copyable.
   - For missing glyphs (e.g. CJK or complex non-Latin scripts), emit a fallback placeholder glyph (`□`) without
     crashing or corrupting document layout.
5. **Keep modules modular and small:** Keep layout and writer modules under 250 lines each, adhering to workspace
   lints and clippy rules.

## Consequences

### Positive
+ **Zero new dependencies:** Uses `lopdf` and `image` which are already in `crates/studyfrog-pdf/Cargo.toml`.
+ **Minimal binary growth:** Less than 250 KB added to the compiled AppImage binary, keeping the total AppImage around 11 MB.
+ **Fast compilation:** Clean builds remain fast, avoiding the 2–4 minute compile overhead of Typst.
+ **Exact visual parity:** Directly mirrors `studyfrog-md`'s blocks, syntax tokens, and theme hues as drawn in the preview.
+ **Deterministic and testable:** The layout engine is pure and can be thoroughly tested with golden tests,
  property tests (no line exceeding printable width), and round-trip text extraction via `pdf_oxide`.

### Negative
- **In-house layout maintenance:** We maintain line wrapping, table column measurement, and page-breaking logic.
- **Limited advanced typography:** Features such as hyphenation dictionaries, paragraph justification, and
  bidirectional text shaping (RTL) are not implemented in the first iteration.
- **Non-Latin fallback:** CJK and non-Latin scripts beyond Latin/Cyrillic require font fallback mechanisms if
  users write multilingual notes; extensive CJK font bundling is deferred to avoid bundling 40+ MB font files.
