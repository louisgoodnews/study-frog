---
id: TASK-0120
type: task
title: Cards and questions from highlighted passages, with the assistant's help
status: backlog
epic: EPIC-022
parents: [STORY-0054]
estimate: 5
labels: [m9, pdf]
acceptance_criteria:
  - "A highlighted passage can become a card or a question with the source link filled in"
  - "The assistant can propose cards and questions from chosen pages; each is checked and accepted one by one as in ADR 0009, and the text sent is shown first"
  - "Items made this way keep the page link and show it while rehearsing as a small 'source' link"
---

## Context
The owner asked for PDF support on 2026-10-31: PDF files are integral to how StudyFrog is used, but today it cannot open them.

## Description
Builds on the assistant's proposal format; adds a page range to the context.

## Acceptance Criteria
- [ ] A highlighted passage can become a card or a question with the source link filled in
- [ ] The assistant can propose cards and questions from chosen pages; each is checked and accepted one by one as in ADR 0009, and the text sent is shown first
- [ ] Items made this way keep the page link and show it while rehearsing as a small "source" link

## Design Notes
A remote model receives the page text, so the consent screen names the document and the page count.

## Dependencies
TASK-0114, TASK-0117 and the assistant (STORY-0049).

## Open Questions
None.
