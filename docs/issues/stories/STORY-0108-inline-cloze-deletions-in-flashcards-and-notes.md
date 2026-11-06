---
id: STORY-0108
type: story
title: Inline cloze deletions in flashcards and notes
status: backlog
epic: EPIC-035
parents: [EPIC-035]
estimate: 5
labels: [m12, core, md, rehearsal, service, ui]
acceptance_criteria:
  - "Markdown cards and notes support {{c1::answer}} and {{c1::answer::hint}} cloze deletion syntax across multiple numbered cloze groups (c1, c2, ...)"
  - "During rehearsal, active cloze groups render as a styled placeholder ([...] or [hint]) before reveal and highlight the target answer upon reveal or after typed submission"
  - "The Markdown editor toolbar and keyboard shortcuts include a quick wrap-as-cloze action that auto-increments or reuses the cloze index"
---

## Context
In academic study workflows (definitions, biochemical pathways, mathematical theorems, psychological diagnostic criteria, and programming syntax), **Cloze Deletion**—masking one or more key phrases inside a sentence (`{{c1::multiple Determiniertheit::Begriff}}`)—is faster to author and provides richer contextual retrieval cues than splitting every sentence into a front/back question. Supporting native cloze syntax in StudyFrog aligns it with Anki, RemNote, and Mochi while keeping vault files 100% plain-text Markdown.

## Description
Implement inline cloze deletion parsing, editor authoring helpers, and rehearsal rendering:
1. **Cloze Syntax & Parser (`crates/studyfrog-md`):**
   - Parse `{{c<N>::<answer>}}` and `{{c<N>::<answer>::<hint>}}` spans (where `<N>` is a positive integer `1..=99`) within card fronts/bodies and note bodies.
   - Extract distinct cloze indices `[1, 2, ...]` and provide pure rendering helpers:
     - `mask_cloze(text, active_index)`: replaces matching `c<active_index>` spans with `[...]` (or `[<hint>]`) while rendering non-active `c<M>` spans as normal unmasked text.
     - `reveal_cloze(text, active_index)`: renders `c<active_index>` with a distinct highlight/bold accent and extracts the target `<answer>` string(s) for typed answer comparison (`studyfrog-slm`).
2. **Editor Authoring Toolbar & Shortcut (`crates/studyfrog-ui/src/view/editor.rs`):**
   - Add a `[...]` **Cloze** button in the Markdown editor formatting bar that wraps the current selection in `{{c1::selection}}` (incrementing `cN` based on existing clozes in the item).
3. **Rehearsal Engine & Importer Support:**
   - Support cloze cards in `Flip cards`, `Write answers`, and `Exam` run modes (comparing typed input against the active cloze `<answer>`).
   - Allow CSV/Anki imports containing `{{c1::...}}` syntax to render natively during rehearsal.

## Acceptance Criteria
- [ ] Markdown cards and notes support {{c1::answer}} and {{c1::answer::hint}} cloze deletion syntax across multiple numbered cloze groups (c1, c2, ...)
- [ ] During rehearsal, active cloze groups render as a styled placeholder ([...] or [hint]) before reveal and highlight the target answer upon reveal or after typed submission
- [ ] The Markdown editor toolbar and keyboard shortcuts include a quick wrap-as-cloze action that auto-increments or reuses the cloze index

## Design Notes
- Touchpoints:
  - `crates/studyfrog-md/src/cloze.rs`: pure cloze parser, masker, and answer extractor.
  - `crates/studyfrog-ui/src/view/editor.rs` & `preview.rs`: toolbar action and styled cloze pill rendering.
  - `crates/studyfrog-ui/src/view/rehearse.rs`: render masked prompt before flip/submit and highlighted answer after reveal.
- Comply with P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-035.

## Open Questions
None.
