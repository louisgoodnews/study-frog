---
id: BUG-0039
type: bug
title: "Mock exam from a PDF is labelled 'untitled' and both PDFs in the list read 'untitled'"
status: backlog
estimate: 1
labels: [m14, rehearse, pdf]
acceptance_criteria:
  - "The 'Mock exam from a lecture PDF' list shows each PDF's file name"
  - "The exam made from a PDF is titled with the PDF's file name, not 'untitled'"
---

## Context
Found while auditing STORY-0186 in the running app (2026-11-22). The vault holds `Vorlesung.pdf` and `Vorlesung2.pdf`. The "Mock exam from a lecture PDF" list shows two buttons, both labelled "untitled". Choosing the first makes "Mock exam: untitled" with 2 questions. BUG-0009 (PDF lists show a placeholder title) is marked done, but the same placeholder shows here.

Screenshots: `/tmp/claude-0/audit/g5/0186-pdf-list.png`, `/tmp/claude-0/audit/g5/0186-pdf-exam.png`.

## Description
Pass the PDF's file name to the list and the exam title, the same way the PDF list in the library does. Check with two PDFs in the vault.

## Acceptance Criteria
- [ ] The 'Mock exam from a lecture PDF' list shows each PDF's file name
- [ ] The exam made from a PDF is titled with the PDF's file name, not 'untitled'

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/setup/mock_exam/` (the PDF list), `studyfrog-service::exam_save::slide_chunks`.
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0186
- BUG-0009 (reopened by this check)

## Open Questions
- None.

Review (2026-11-22): still open. Both PDFs are listed as "untitled" and the exam reads "Mock exam: untitled" (`/tmp/claude-0/review-e/02-pdf-list.png`, `03-pdf-exam.png`). Status stays backlog.
