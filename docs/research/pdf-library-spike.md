# PDF library spike (TASK-0109)

Measured on 2026-10-31 on the development machine (i9-11900K, Linux, rustc 1.99.1, release builds, one file at a
time at low priority). The harness is throwaway code outside this repository; this note keeps what it found.

## What was tried
- **Corpus:** 65 real PDFs found on the machine: 5 documentation PDFs (a 477-page Ghostscript manual, LaTeX papers, a
  Skia/PDF manual, a LaTeX-to-PDF spec) and 60 sampled by producer (Word 2010 to 2019, Excel, Adobe PDF Library,
  Ghostscript, LibreOffice, pdfTeX, Photoshop scans and others; 1 to 477 pages; mostly German and English). Four
  of them have no text at all (images only). Plus five hand-made broken files: truncated, random bytes, a broken
  cross-reference table, a page tree that points at itself, a one-million-point page.
- **Reference:** poppler (`pdftotext`, `pdftoppm`) for text and for the look of the rendered page.
- **Text score:** word-bag F1 against poppler's text. **Render score:** RMSE of the 144 dpi render against poppler's,
  plus looking at the pages.
- **Candidates** (versions checked on crates.io on the day): `hayro` 0.8.0, `pdf_oxide` 0.3.78, `lopdf` 0.46.0,
  `pdf-extract` 0.12.1, `pdfium-render` 0.9.4 with the prebuilt pdfium of chromium/8125.
- **Excluded without measuring:** `pdfium` 0.10.4 (GPL-3.0), `mupdf` 0.8.0 (AGPL-3.0): both are incompatible with
  the project's MIT OR Apache-2.0 licence. `rpdfium` 7676.6.4 has 301 downloads in total and was not tried.

## Results
| | hayro | pdf_oxide | pdf-extract | lopdf | pdfium-render |
|---|---|---|---|---|---|
| Licence | Apache-2.0 OR MIT | MIT OR Apache-2.0 | MIT | MIT | MIT OR Apache-2.0 (+ pdfium: BSD-3) |
| Unsafe code | forbidden | not audited | not audited | not audited | native C++ library |
| Renders pages | yes | yes (feature `rendering`) | no | no | yes |
| Text with positions | no | yes (chars, words, spans) | text only | no | yes |
| Pictures, outline | no | yes | no | structure only | yes |
| Page operations, merge, split | no | yes | no | yes | yes |
| Opened and handled all 65 files | yes | yes | no: 3 panics | not run on the corpus | not fully measured |
| Text F1 vs poppler (median, mean) | n/a | 0.995, 0.956 | 0.998, 0.954 | n/a | 0.999, 0.932 |
| Render time, 144 dpi, first / p90 / max | 8 ms / 37 ms / 137 ms | 39 ms / 424 ms / 628 ms | n/a | n/a | not measured |
| Added to a stripped binary alone | 6.8 MB | 8.4 MB (without rendering) | n/a | 1.4 MB | 2.9 MB + 7.7 MB `libpdfium.so` |

Details that matter:
- **pdf_oxide renders wrong glyphs** for text in fonts that are not embedded (two of the pages I looked at come out as
  Greek-looking nonsense) and is clearly further from poppler than hayro on 7 of 65 pages. hayro rendered the same
  pages correctly. Text extraction from the same files is right, so the fault is in drawing, not in reading.
- **Text:** all libraries are within a point of each other against poppler; the three files under 0.9 are single-page
  schematics. Speed is not a problem: pdf_oxide needs 3.5 ms per page at the median and 1.8 s for the 477-page
  manual (poppler 0.85 s, pdf-extract 0.17 s). Image-only files give no text, as poppler does.
- **Editing with pdf_oxide** worked on three test files: rotating a page, reversing and selecting pages, extracting
  pages 0 and 1, merging a file into another; poppler read every result with the right page count and text, and the
  reversed file starts with the original fourth page. Results keep objects they no longer use (a 4-page file of the
  55-page manual stays 1.4 MB); lopdf's delete-and-prune gave a clean 54-page file.
- **Broken files:** pdf_oxide and hayro return errors for truncated, random, broken-xref and self-referencing files
  and never hung (all under half a second). hayro **panics** inside its rasteriser on a one-million-point page
  (a `None` unwrap in `vello_common`); the work therefore has to run behind a size limit and `catch_unwind` or on its
  own thread (TASK-0113). pdf_oxide returned an error for the same file and rendered it at 4001 x 4001.
- **hayro lacks:** encrypted PDFs and some blending features (its README says so). pdf_oxide reads legacy-encrypted files.
- **pdfium:** the engine of Chrome, so its output is the reference, but the crate needs a native library next to the
  binary (3.8 MB download, 7.7 MB on disk), its bindings are tied to a pdfium version (the crate's newest feature
  set is for build 7881 and the current release is 8125; enabling both bindings sets does not compile), and it loads
  the library at run time. Its text was measured (above). **Its render time and quality were not measured:** running the
  downloaded library was blocked by the tool's permission check partway through, and I did not work around that.

## Not covered
Non-Latin text (no CJK or right-to-left PDFs in the corpus), form fields, signatures, very large single pages in the
wild, password-protected files, and any measure of memory use. Annotations were not written or read.

## Maintenance signals (GitHub, same day)
hayro: 832 stars, pushed that morning; pdf_oxide: 1151 stars, pushed that night, 349 open issues, vendor benchmark
claims not checked; lopdf: 2289 stars, pushed the day before; pdfium-render: 717 stars, last push 2026-10-08.
