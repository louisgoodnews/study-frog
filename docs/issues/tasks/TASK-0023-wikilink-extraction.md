---
id: TASK-0023
type: task
title: Wikilink extraction
status: done
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
- [x] `[[Title]]`, `[[Title|label]]`, `[[Title#Heading]]` extracted
- [x] Links inside code spans and fences are ignored

## Design Notes
Own scanner (`links.rs`): single pass, borrows from the input, handles code spans of any tick length, fenced blocks (``` and ~~~), backslash escapes; `[[Target#Heading|label]]` combination also supported; links with an empty target (e.g. `[[#Heading]]`) are ignored. pulldown-cmark is not needed for this, so no micro-benchmark was required.

## Dependencies
TASK-0021.

## Open Questions
None.
