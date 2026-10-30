---
id: TASK-0023
type: task
title: Wikilink extraction
status: backlog
epic: EPIC-005
parents: [STORY-0010]
estimate: 3
labels: [m2, md]
acceptance_criteria:
  - "`[[Title]]`, `[[Title|label]]`, `[[Title#Heading]]` extracted"
  - "Links inside code spans and fences are ignored"
---

## Context
Single pass over the body; borrow slices (no allocation per link where possible).

## Description
Wikilink extraction.

## Acceptance Criteria
- [ ] `[[Title]]`, `[[Title|label]]`, `[[Title#Heading]]` extracted
- [ ] Links inside code spans and fences are ignored

## Design Notes
pulldown-cmark text events or own scanner: decide with a micro-benchmark (P10).

## Dependencies
TASK-0021.

## Open Questions
None.
