---
id: TASK-0109
type: task
title: Spike and ADR 0010 to choose the PDF library
status: backlog
epic: EPIC-022
parents: [STORY-0054]
estimate: 5
labels: [m9, pdf]
acceptance_criteria:
  - "Candidates are tried on a test set of PDFs: pdfium-based crates (needs the pdfium library bundled in the AppImage), the pure-Rust rpdfium and pdf_oxide, and lopdf for editing; mupdf is excluded for its AGPL licence unless the owner decides otherwise"
  - "Licence, version and upkeep of each are checked on crates.io, not taken from a blog"
  - "Page render time, text extraction quality (including non-Latin text and columns), image extraction, size added to the AppImage and build requirements are measured"
  - "ADR 0010 records the choice and the build-vs-buy comparison and is accepted by the owner"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
A spike with numbers, then an ADR in the style of 0006 to 0009.

## Acceptance Criteria
- [ ] Candidates are tried on a test set of PDFs: pdfium-based crates (needs the pdfium library bundled in the AppImage), the pure-Rust rpdfium and pdf_oxide, and lopdf for editing; mupdf is excluded for its AGPL licence unless the owner decides otherwise
- [ ] Licence, version and upkeep of each are checked on crates.io, not taken from a blog
- [ ] Page render time, text extraction quality (including non-Latin text and columns), image extraction, size added to the AppImage and build requirements are measured
- [ ] ADR 0010 records the choice and the build-vs-buy comparison and is accepted by the owner

## Design Notes
Keep the library behind the `studyfrog-pdf` crate's functions (TASK-0110) so another can replace it. A search for the options found vendor pages with unverified benchmarks, so measure ourselves.

## Dependencies
None.

## Open Questions
The owner decides the ADR.
