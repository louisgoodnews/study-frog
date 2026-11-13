---
id: TASK-0041
type: task
title: "Preview renderer"
status: done
epic: EPIC-009
parents: [STORY-0018]
estimate: 3
labels: [m4, ui, md]
acceptance_criteria:
  - "CommonMark preview with wikilinks as clickable links"
  - "Renders cards as front/back"
---

## Context
pulldown-cmark per ADR 0005 (hard general problem, P10).

## Description
Preview renderer.

## Acceptance Criteria
- [x] CommonMark preview with wikilinks as clickable links
- [x] Renders cards as front/back

## Design Notes
`studyfrog-md::blocks::parse_blocks` turns Markdown into a block-and-span model (pulldown-cmark + wikilinks via `split_wikilinks`; links inside code or ordinary links stay literal). The editor caches the blocks and re-parses only the field that changed, so drawing allocates no parse results per frame. Wikilinks resolve by title (case-insensitive); aliases are not indexed yet. Cards render as front/back areas in the editor preview.

## Dependencies
TASK-0040.

## Open Questions
None.
