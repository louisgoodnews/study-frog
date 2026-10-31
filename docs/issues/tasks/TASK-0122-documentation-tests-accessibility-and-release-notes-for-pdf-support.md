---
id: TASK-0122
type: task
title: Documentation, tests, accessibility and release notes for PDF support
status: backlog
epic: EPIC-022
parents: [STORY-0054]
estimate: 3
labels: [m9, pdf]
acceptance_criteria:
  - "docs/specs/pdf.md describes the formats (sidecar, source links, limits), the library choice and what is not supported"
  - "Headless view tests cover the viewer, annotations, page operations and extraction, including the accessibility audit"
  - "Benchmarks for render time and text extraction of a 500-page file are recorded with their budgets"
  - "CHANGELOG, STATUS and the release notes say what was checked by hand and what only by tests"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Runs alongside the other tasks and closes the story.

## Acceptance Criteria
- [ ] docs/specs/pdf.md describes the formats (sidecar, source links, limits), the library choice and what is not supported
- [ ] Headless view tests cover the viewer, annotations, page operations and extraction, including the accessibility audit
- [ ] Benchmarks for render time and text extraction of a 500-page file are recorded with their budgets
- [ ] CHANGELOG, STATUS and the release notes say what was checked by hand and what only by tests

## Design Notes
A manual pass on a real desktop with a few real-world PDFs (a paper, a scanned page, a slide deck, a large book) is part of the closing.

## Dependencies
All other tasks of STORY-0054.

## Open Questions
None.
