---
id: STORY-0144
type: story
title: Macro escaping syntax to disable expansion
status: done
epic: EPIC-045
parents: [EPIC-045]
estimate: 2
labels: [m12, markdown, ui]
acceptance_criteria:
  - "Typing a macro name prefixed with '#' (such as {#code}, {#table}, or {#rule}) suppresses expansion and leaves the raw text as typed"
  - "Escaped macro strings remain unaffected by caret movement or subsequent typing"
---

## Context
When writing technical documentation, software development notes, or discussing StudyFrog's macro system itself, learners need a way to write `{code}` or `{table}` without the editor immediately intercepting the text and expanding it into a multi-line snippet.

## Description
Implement macro suppression in `crates/studyfrog-ui/src/md_macros.rs`:
1. In `macro_before(text: &str, caret: usize)`, check if the macro trigger starts with `#` (e.g. `{#code}`).
2. If prefixed with `#`, return `None` so that the macro expansion is bypassed and the literal text `{#code}` remains in the document.
3. Add unit test coverage in `tests/md_macros.rs` verifying that `{#...}` never triggers an expansion.

## Acceptance Criteria
- [x] Typing a macro name prefixed with '#' (such as {#code}, {#table}, or {#rule}) suppresses expansion and leaves the raw text as typed
- [x] Escaped macro strings remain unaffected by caret movement or subsequent typing

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/md_macros.rs`, `crates/studyfrog-ui/tests/md_macros.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-045 (Macro library expansion and escaping syntax).

## Open Questions
- None.
