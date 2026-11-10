---
id: STORY-0145
type: story
title: Markdown list parser support for dash asterisk numbers and alphabetical markers
status: done
epic: EPIC-046
parents: [EPIC-046]
estimate: 3
labels: [m12, markdown, ui]
acceptance_criteria:
  - "Unordered lists beginning with '-' or '*' render with neat bullet markers"
  - "Ordered lists beginning with numbers (1., 2.) or letters (a., b., c. or (a), (b)) render with ordered markers and proper horizontal alignment"
  - "Nested lists with mixed markers maintain proper indentation levels"
---

## Context
When drafting flashcard questions or study outlines, users frequently employ alphabetical lists for multiple-choice options (`a.`, `b.`, `c.` or `(a)`, `(b)`) as well as alternating bullet markers (`-` and `*`). Standard markdown parsing often lumps alphabetical lines into a single paragraph block rather than treating them as a coherent list.

## Description
Deliver robust list parsing and formatting in `crates/studyfrog-md` and `crates/studyfrog-ui`:
1. In `crates/studyfrog-md/src/blocks_build.rs`, detect alphabetical list markers at line starts and group them into structured `Block::List` items.
2. In `crates/studyfrog-ui/src/view/preview.rs`, support rendering alphabetical marker sequences with correct alignment to the line height.
3. Verify list parsing and rendering in `tests/md_rich_ui.rs`.

## Acceptance Criteria
- [x] Unordered lists beginning with '-' or '*' render with neat bullet markers
- [x] Ordered lists beginning with numbers (1., 2.) or letters (a., b., c. or (a), (b)) render with ordered markers and proper horizontal alignment
- [x] Nested lists with mixed markers maintain proper indentation levels

## Design Notes
- Touchpoints: `crates/studyfrog-md/src/blocks.rs`, `crates/studyfrog-md/src/blocks_build.rs`, `crates/studyfrog-ui/src/view/preview.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-046 (Markdown viewer list syntax enhancements).

## Open Questions
- None.
