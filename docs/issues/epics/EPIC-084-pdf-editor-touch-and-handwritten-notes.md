---
id: EPIC-084
type: epic
title: "PDF editor with touch support: handwritten notes on PDFs, converted into Markdown notes"
status: backlog
parents: []
labels: [m14, pdf, ui, md]
acceptance_criteria:
  - 'The PDF viewer accepts pen and touch input to write and draw on a page; strokes are stored per page, survive reopening the file, and never change the PDF itself'
  - 'Handwritten strokes can be converted into Markdown text, and the user reviews and edits the result before it is saved as a note linked to the PDF page'
  - 'The viewer works on a touch screen: pinch to zoom, one-finger pan, and a writing mode that does not scroll the page'
  - 'Handwriting recognition runs locally by default; any cloud recognition is opt-in and follows the assistant''s provider rules'
  - 'Research into recognition options and the PDF stroke format is recorded in docs/research/ before the stories start, with the chosen approach and its reasons'
---

## Context
The PDF viewer (M9, EPIC-022) displays and navigates PDFs but offers no way to write on them. Learners often annotate printed material by hand. Turning those handwritten notes into Markdown would bring them into the vault, where they can be searched, linked and rehearsed.

## Description
- Research: stroke storage alongside the PDF (a sidecar file, not a change to the PDF), handwriting recognition options that run locally, and conversion of recognised text into the Markdown the vault reads.
- Input: pen and touch handling in the PDF viewer, with a writing mode and zoom and pan that work on touch.
- Conversion: recognise the strokes, show the Markdown for review, then save it as a note linked to the page it came from.
- Privacy: the strokes and recognised text stay local unless the user opts in to a cloud recogniser.

## Acceptance Criteria
- [ ] Pen and touch annotations are stored per page as a sidecar and survive reopening
- [ ] Handwriting converts to Markdown that the user reviews before saving as a linked note
- [ ] Touch support: pinch zoom, one-finger pan, a writing mode that does not scroll
- [ ] Local recognition by default; cloud recognition is opt-in
- [ ] Research is recorded in docs/research/ before the stories start

## Design Notes
- The PDF file is never modified; annotations live beside it.
- Recognition is a hard general problem: a runtime or model is acceptable per P10, with the comparison recorded.
- The pdf crate and the ui crate own the viewer; the md crate owns the note format.

## Dependencies
- EPIC-022 (PDF documents) for the viewer this extends.
- EPIC-083 shares the local-model research for recognition.

## Decisions
_None yet._

## Open Questions
- Which tablet or pen input must work first: a Windows touch screen, a Wacom-style pen, or finger only?
- Should the converted note keep the handwriting image beside the text?
