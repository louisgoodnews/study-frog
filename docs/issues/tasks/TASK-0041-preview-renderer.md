---
id: TASK-0041
type: task
title: Preview renderer
status: backlog
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
- [ ] CommonMark preview with wikilinks as clickable links
- [ ] Renders cards as front/back

## Design Notes
No per-frame allocation: cache the parsed preview.

## Dependencies
TASK-0040.

## Open Questions
None.
