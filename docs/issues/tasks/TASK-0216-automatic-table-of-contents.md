---
id: TASK-0216
type: task
title: "Automatic table of contents"
status: done
epic: EPIC-026
parents: [STORY-0074]
estimate: 3
labels: [m11, ui, md]
acceptance_criteria:
  - "{toc} in a note draws a list of the note's headings as links that scroll to them, and updates as the headings change"
  - "The file keeps a marker that other Markdown tools show as text, and no copy of the headings is stored"
  - "Headings of every level are listed with their indent; a note without headings says so"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
An automatic table of contents for long notes.

## Acceptance Criteria
- [x] {toc} in a note draws a list of the note's headings as links that scroll to them, and updates as the headings change
- [x] The file keeps a marker that other Markdown tools show as text, and no copy of the headings is stored
- [x] Headings of every level are listed with their indent; a note without headings says so

## Design Notes
Done as `Block::Toc` for a line `[TOC]` and `preview_rich::toc`; clicking an entry scrolls to the heading with the same text. `{toc}` comes with the macros (TASK-0198).

The marker is a line `[TOC]`; the macro {toc} inserts it. The preview collects the headings from the parsed blocks and draws them above the rest; clicking scrolls the preview.

## Dependencies
TASK-0197, TASK-0198.

## Open Questions
None.
