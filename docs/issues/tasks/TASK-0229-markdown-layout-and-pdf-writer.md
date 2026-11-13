---
id: TASK-0229
type: task
title: "Markdown layout and PDF writer"
status: done
epic: EPIC-026
parents: [STORY-0082]
estimate: 8
labels: [m11, pdf, md]
acceptance_criteria:
  - "A pure function turns the parsed Markdown of an item into pages (lines, blocks and page breaks) for a given page size, margins and font size, with no I/O"
  - "A writer turns the pages into PDF bytes with embedded fonts, selectable text, pictures, links and an outline built from the headings; the output opens in the viewer of StudyFrog and parses with pdf_oxide"
  - "Headings, paragraphs, bold, italic, underline, colour, lists, tasks, quotes, rules, tables, code blocks with their highlighting, pictures and links are laid out; long lines wrap, long tables and code split across pages, nothing is cut off at an edge"
  - "Tests: golden layouts for each element, a property test that no line is wider than the text area, and a text-extraction round trip that finds every word of the source; a benchmark for a 100-page note"
---

## Context
The owner asked on 2026-11-04 for a new feature: notes can be exported to PDF, which needs a converter from Markdown to PDF.

## Description
The converter itself, as data in and bytes out.

## Acceptance Criteria
- [x] A pure function turns the parsed Markdown of an item into pages (lines, blocks and page breaks) for a given page size, margins and font size, with no I/O
- [x] A writer turns the pages into PDF bytes with embedded fonts, selectable text, pictures, links and an outline built from the headings; the output opens in the viewer of StudyFrog and parses with pdf_oxide
- [x] Headings, paragraphs, bold, italic, underline, colour, lists, tasks, quotes, rules, tables, code blocks with their highlighting, pictures and links are laid out; long lines wrap, long tables and code split across pages, nothing is cut off at an edge
- [x] Tests: golden layouts for each element, a property test that no line is wider than the text area, and a text-extraction round trip that finds every word of the source; a benchmark for a 100-page note

## Design Notes
Lives behind `studyfrog-pdf` (new `write` modules; keep files under 250 lines) and uses the Markdown parser of `studyfrog-md` as it is, so the preview and the PDF agree. Functional free functions only. Build or buy as decided in TASK-0228.

## Dependencies
TASK-0228, TASK-0197, TASK-0214, TASK-0215 and TASK-0216.

## Open Questions
None.
