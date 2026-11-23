---
id: BUG-0039
type: bug
title: "Mock exam from a PDF is labelled 'untitled' and both PDFs in the list read 'untitled'"
status: done
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
- [x] The 'Mock exam from a lecture PDF' list shows each PDF's file name
- [x] The exam made from a PDF is titled with the PDF's file name, not 'untitled'

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/setup/mock_exam/` (the PDF list), `studyfrog-service::exam_save::slide_chunks`.
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0186
- BUG-0009 (reopened by this check)

## Open Questions
- None.

Review (2026-11-22): still open. Both PDFs are listed as "untitled" and the exam reads "Mock exam: untitled" (`/tmp/claude-0/review-e/02-pdf-list.png`, `03-pdf-exam.png`). Status stays backlog.

## Fix (2026-11-23)

Cause: `exec/mock.rs` listed the lecture PDFs by their own title, which is the placeholder "untitled" in both files, without the check that `pdf_names::label` makes. `setup/mock_exam/open.rs` `from_slide` then titled the exam with that title.

Change: the list shows `pdf_names::label(path, title)` (the file name, with the title when it says something), and the exam is titled "Mock exam: <file name>".

Tests: `studyfrog-ui/tests/mock_exam_ui.rs` `an_exam_from_a_pdf_is_titled_with_its_file_name`; `studyfrog-ui/tests/mock_exam_model_ui.rs` `a_lecture_pdf_is_listed_then_made_into_an_exam` (its title expectation is now the file name). `studyfrog-ui/tests/pdf_names.rs` covers `label`.

Verified in the running app (fixed build, 2026-11-23): the list shows "Vorlesung.pdf" and "Vorlesung2.pdf" (`/tmp/claude-0/exam/a-pdf-01-list.png`), and the exam of the second file is "Mock exam: Vorlesung2.pdf" (`/tmp/claude-0/exam/a-pdf-03-title.png`). Before the fix: `/tmp/claude-0/exam/b-pdf-01-list.png` (both "untitled").
