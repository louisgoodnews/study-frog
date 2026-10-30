---
id: EPIC-005
type: epic
title: Markdown and front-matter
status: done
parents: []
labels: [m2, md]
acceptance_criteria:
  - "Every item kind parses from and renders to Markdown with front-matter"
  - "`parse(render(x)) == x` holds; unknown keys preserved"
  - "Bad files produce typed errors with file and line, never panics"
---

## Context
Markdown is the source of truth for content (ADR 0003, 0005).

## Description
Parsing and rendering in `studyfrog-md`, pure functions over text.

## Acceptance Criteria
- [x] Every item kind parses from and renders to Markdown with front-matter
- [x] `parse(render(x)) == x` holds; unknown keys preserved
- [x] Bad files produce typed errors with file and line, never panics

## Design Notes
Children: STORY-0009, STORY-0010.

## Dependencies
M1 exit.

## Open Questions
Which CommonMark crate if GFM tables are needed in previews (pulldown-cmark first, per ADR 0005)?
