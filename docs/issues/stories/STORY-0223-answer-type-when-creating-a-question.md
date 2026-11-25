---
id: STORY-0223
type: story
title: "Answer type when creating a question: open text, single choice or multiple select"
status: ready
epic: EPIC-072
parents: ['EPIC-072']
estimate: 5
labels: [m14, ui, md, rehearsal]
acceptance_criteria:
  - "The Create dialog for a question offers the answer types Open text, Single choice and Multiple select"
  - "Choosing a choice type writes the option template (two or more options with `- [ ]` and `- [x]` markers) into the new item's body, so the item is a choice item from the start"
  - "The control protocol accepts the same choice: `item create question <title> --answer-type open|single|multiple`"
  - "A created choice item appears as a choice item in the list, the viewer and the rehearsal (see STORY-0222)"
---

## Context
Questions can be created with open text only. The owner reported (2026-11-25) that single-select and multiple-select items
cannot be created, edited, viewed or rehearsed.

## Description
Add the answer type to the question creation path (the dialog and the control command), using the syntax of EPIC-072.

## Acceptance Criteria
- [ ] Answer type in the Create dialog for questions
- [ ] Option template written for choice types
- [ ] Control command accepts the answer type
- [ ] Created choice items are recognised everywhere

## Design Notes
The answer type is not a separate field: it is derived from the option markers in the body (STORY-0220). The Create
dialog only writes the template. Open question below.

## Dependencies
- STORY-0220 (parser), STORY-0221 (grading) for rehearsal.

## Open Questions
- Should the answer type also be stored in the front matter, so that it survives an edit that removes the markers?
