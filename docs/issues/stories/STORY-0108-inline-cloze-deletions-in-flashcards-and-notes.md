---
id: STORY-0108
type: story
title: Inline cloze deletions in flashcards and notes
status: done
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
- [x] Markdown cards and notes support {{c1::answer}} and {{c1::answer::hint}} cloze deletion syntax across multiple numbered cloze groups (c1, c2, ...)
- [x] During rehearsal, active cloze groups render as a styled placeholder ([...] or [hint]) before reveal and highlight the target answer upon reveal or after typed submission
- [x] The Markdown editor toolbar and keyboard shortcuts include a quick wrap-as-cloze action that auto-increments or reuses the cloze index

## Design Notes
- Touchpoints:
  - `crates/studyfrog-md/src/cloze.rs`: pure cloze parser, masker, and answer extractor.
  - `crates/studyfrog-ui/src/view/editor.rs` & `preview.rs`: toolbar action and styled cloze pill rendering.
  - `crates/studyfrog-ui/src/view/rehearse.rs`: render masked prompt before flip/submit and highlighted answer after reveal.
- Comply with P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and verified:
- `crates/studyfrog-md/src/cloze.rs`: spans, groups, `mask_cloze`, `reveal_cloze`, `preview_cloze`, `blank_cloze`,
  answers, next/last number, the turn-taking `active_group`; unit and property tests (any text safe, masking hides
  exactly the active answers, cards and notes with gaps survive parse/render) in `crates/studyfrog-md/tests/cloze.rs`.
- Service: `content_cloze.rs` turns a card's or note's content into the masked prompt, highlighted answer and the
  words to compare (`ItemContent.expected`); `compare::score_item_answer_with` compares with the words the run
  showed; a cloze card needs no back and its default title blanks the gaps. Fixture-vault tests
  `crates/studyfrog-service/tests/cloze_content.rs` (groups take turns with reviews, `cloze_group`, notes, scoring).
- Window: the run shows the placeholder and the highlighted answer (Flip, Type answers and Exam use the same
  content; the typed answer goes to scoring with the gap's words); the `[...]` toolbar button and Ctrl+Shift+C /
  Ctrl+Shift+Alt+C; the editor preview shows gaps. Harness tests `crates/studyfrog-ui/tests/recall_cloze_ui.rs`,
  `md_edit.rs`; screenshots `--screen run`, `--screen run-answer` and `--screen item:<ULID>` looked at.
- Spec `docs/specs/cloze.md`.

Design decision: the groups of one item share its schedule and take turns (one file, one schedule); a
`cloze_group` key gives a copy its own group. Anki cloze notes keep importing as one pre-masked card per gap.
Build vs buy: no crate; the scanner is a few dozen lines.

## Dependencies
EPIC-035.

## Open Questions
None.
