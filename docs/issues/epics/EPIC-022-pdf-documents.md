---
id: EPIC-022
type: epic
title: PDF documents
status: done
parents: []
labels: [m9, pdf]
acceptance_criteria:
  - "PDF files in the vault can be viewed, annotated and rearranged, and their text and pictures can be extracted into notes and cards"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Make PDFs first-class vault content: find them, read them in the app, mark them up, change their pages, and turn what they say into notes, flashcards and questions that point back to the page they came from.

## Acceptance Criteria
- [x] PDF files in the vault can be viewed, annotated and rearranged, and their text and pictures can be extracted into notes and cards

## Design Notes
Out of scope for now: OCR (deferred), form filling, redaction, signing, and editing the text inside a PDF.
One story (STORY-0054) with its tasks. The first task is a spike and an ADR, because everything else depends on which PDF library is used (build vs buy, P10: reading and drawing PDFs is a hard general problem, so the answer is a crate, not our own parser).

## Dependencies
None.

## Open Questions
Deferred by the owner on 2026-10-31: **OCR** for scanned PDFs (Tesseract or a language model). It is not in STORY-0054 and has no tasks yet; it becomes its own story under this epic or a later one once a library or model is chosen and the owner asks for it. Until then scanned pages are only detected (TASK-0118).
