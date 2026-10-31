# ADR 0010: PDF libraries behind one crate

- Status: **Proposed** (2026-10-31; waiting for the owner. Nothing else in EPIC-022 starts before it is accepted.)
- Date: 2026-10-31

## Context
EPIC-022 needs to open, render, search, annotate, rearrange and extract from PDFs. Reading and drawing PDFs is a hard
general problem, so the answer is a crate (P10: build in-house only when it beats the crate; it would not). The
numbers are in [docs/research/pdf-library-spike.md](../research/pdf-library-spike.md): 65 real PDFs plus five broken files,
poppler as the reference.

## Options
| | all in pdf_oxide | pdfium-render + pdfium | hayro (draw) + pdf_oxide (read, edit) |
|---|---|---|---|
| One library for everything | yes | yes | no, two |
| Pure Rust, no native binary to ship | yes | no (7.7 MB `libpdfium.so`, run-time loading, bindings tied to a pdfium build) | yes |
| Page rendering | wrong glyphs for non-embedded fonts, worse than poppler on 7 of 65 pages, p90 424 ms | reference quality (not measured here) | correct on the same pages, median 8 ms, p90 37 ms |
| Text, pictures, outline | yes | yes | yes (pdf_oxide) |
| Rotate, reorder, delete, merge, split | yes | yes | yes (pdf_oxide) |
| Licence | MIT OR Apache-2.0 | MIT OR Apache-2.0, pdfium BSD-3 | Apache-2.0 OR MIT, MIT OR Apache-2.0 |
| Weak point | drawing quality | shipping and updating a C++ library | two young libraries; hayro has no encryption |

Excluded: `pdfium` (GPL-3.0) and `mupdf` (AGPL-3.0) for their licences; `pdf-extract` (panicked on 3 of 65 files, no
positions); `rpdfium` (301 downloads in total).

## Decision
1. **hayro draws pages; pdf_oxide reads and edits.** hayro for rendering (fast, accurate on the corpus, forbids unsafe
   code), pdf_oxide for text with positions, pictures, outline, annotations, and rotate, reorder, delete, extract,
   merge and split. `lopdf` stays available to prune unused objects from a saved copy if pdf_oxide's output is too big.
2. **Both live behind `studyfrog-pdf`** (TASK-0110). The rest of the workspace sees plain data: page sizes, text
   blocks with rectangles, pictures, an outline, and RGBA pixels. Replacing either library, or adding pdfium as a third
   renderer, changes that crate only.
3. **Hostile input is the crate's job** (TASK-0113): a size limit before rendering, `catch_unwind` or a separate
   thread around hayro and pdf_oxide, a time limit, and no execution of anything inside a PDF.
4. **Corpus tests:** TASK-0110 keeps a small set of generated PDFs, including the broken ones from the spike, so a
   library upgrade that changes behaviour is caught.
5. **Revisit** if users' PDFs show drawing defects in hayro that pdfium would not have, or if pdf_oxide's editing
   output proves unreliable on real files; the fallback is `pdfium-render` behind the same crate, with the library
   bundled in the AppImage.

## Consequences
+ No native library to bundle, sign or update; one `cargo build`.
+ Rendering is fast enough to draw a page per frame budget without a spinner for typical files.
+ The seam keeps the choice reversible.
- Two young libraries to follow (pdf_oxide has 349 open issues, hayro is at 0.8); upgrades need the corpus tests.
- The AppImage grows: the two libraries add about 6.8 MB and 8.4 MB to a stripped binary on their own (the current
  AppImage is 11 MB); the real figure is recorded when TASK-0110 links them.
- Password-protected PDFs will not draw until hayro supports them; the viewer says so instead of failing silently.
- Not measured: non-Latin text, memory use, annotations, and pdfium's render time (see the spike note).
