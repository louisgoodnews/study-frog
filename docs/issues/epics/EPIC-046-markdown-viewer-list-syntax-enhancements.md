---
id: EPIC-046
type: epic
title: Markdown viewer list syntax enhancements
status: done
parents: []
labels: [m12, markdown, core]
acceptance_criteria:
  - "The markdown viewer correctly parses and renders unordered lists using both '-' and '*' markers"
  - "The markdown viewer supports numbered lists and alphabetical lettered lists (a, b, c, or (a), (b), (c))"
  - "List items align with correct baseline spacing and indentation hierarchy"
---

## Context
Learners format study notes and flashcard answers using a variety of list conventions: bullet points with `-` or `*`, numbered steps (`1.`, `2.`), and multi-choice / lettered sub-lists (`a.`, `b.`, `c.` or `(a)`, `(b)`). While standard CommonMark handles basic numeric and dash lists, alphabetical lists and alternate list marker styles need first-class recognition in StudyFrog.

## Description
Enhance list syntax and rendering in `crates/studyfrog-md` and `crates/studyfrog-ui`:
1. **Multi-marker list recognition (`STORY-0145`):** Extend block parsing in `crates/studyfrog-md/src/blocks_build.rs` to recognize alphabetical list sequences (`a.`, `b.`, `(a)`, `(b)`) alongside unordered dashes/asterisks and ordered integers.
2. **List rendering:** Ensure consistent marker alignment, indent levels, and task-list checkbox compatibility in `crates/studyfrog-ui/src/view/preview.rs`.

## Acceptance Criteria
- [x] The markdown viewer correctly parses and renders unordered lists using both '-' and '*' markers
- [x] The markdown viewer supports numbered lists and alphabetical lettered lists (a, b, c, or (a), (b), (c))
- [x] List items align with correct baseline spacing and indentation hierarchy

## Design Notes
- Touchpoints: `crates/studyfrog-md/src/blocks_build.rs`, `crates/studyfrog-ui/src/view/preview.rs`.
- Child story: STORY-0145.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- None.

## Open Questions
- None.
