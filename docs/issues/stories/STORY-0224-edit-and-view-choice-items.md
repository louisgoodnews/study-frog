---
id: STORY-0224
type: story
title: "Edit and view choice items: options and correct answers in the editor and the item viewer"
status: review
epic: EPIC-072
parents: ['EPIC-072']
estimate: 5
labels: [m14, ui, md]
acceptance_criteria:
  - "The item editor for a choice item shows the options as a list with a correct/wrong switch and add/remove controls, and writes the same Markdown the parser reads"
  - "The item viewer and the Details view show the options, marking the correct ones, for single and multiple choice items"
  - "An edit that leaves a choice item with no correct option is reported in the editor, not saved silently"
  - "Changing a choice item to open text removes the option markers only after a confirmation"
---

## Context
Choice items are data in the Markdown body (STORY-0220), so the editor shows the raw text and nothing shows the choices
as choices. The owner reported that choice items cannot be edited or viewed.

## Description
Show choice items as options in the editor and the viewer, and keep the Markdown as the stored form.

## Acceptance Criteria
- [ ] Option list editing with correct/wrong switch, writing Markdown
- [ ] Viewer and Details show the options and the correct ones
- [ ] No silent save with zero correct options
- [ ] Confirmation before a choice item becomes open text

## Design Notes
The Markdown stays the source of truth; the option list is a view of the same text. Reuse the editor's existing save path.

## Dependencies
- STORY-0220 (parser).

## Open Questions
- None.
