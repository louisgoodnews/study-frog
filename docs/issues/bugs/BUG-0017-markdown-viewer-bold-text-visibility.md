---
id: BUG-0017
type: bug
title: "Boldness of words in markdown viewer not visible"
status: done
estimate: 2
labels: [m12, ui, markdown]
acceptance_criteria:
  - "Bold words (**bold** or __bold__) in markdown paragraphs, cards, questions, and notes are distinctly distinguishable from regular body text"
  - "Bold formatting meets APCA contrast standards and visual hierarchy in both light and dark themes"
---

## Context
In the Markdown viewer (`crates/studyfrog-ui/src/view/preview.rs`), words formatted as bold (`**bold**` or `__bold__`) appear virtually indistinguishable from regular body prose. Because egui's default font family (`Ubuntu-Light`) lacks a dedicated bold weight loaded into `FontFamily::Proportional`, relying solely on `RichText::strong()` alters only text luminance without increasing stroke width or visual weight, causing bold emphasis to be missed by learners during study.

## Description
Ensure bold text stands out prominently in the Markdown viewer:
1. In `crates/studyfrog-ui/src/view/preview.rs` and `preview_rich.rs`, enhance bold rendering for spans marked with `style.bold = true` or headings.
2. Apply distinct high-contrast color styling (`palette.text_strong` / higher brightness) and/or slight visual weight adjustment so that bolded terms, key definitions, and active recall cues are immediately obvious in both light and dark themes.
3. Validate typography rendering and visual clarity across test suites.

## Acceptance Criteria
- [x] Bold words (**bold** or __bold__) in markdown paragraphs, cards, questions, and notes are distinctly distinguishable from regular body text
- [x] Bold formatting meets APCA contrast standards and visual hierarchy in both light and dark themes

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/preview.rs`, `crates/studyfrog-ui/src/theme.rs`, `crates/studyfrog-ui/src/typo.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- None.

## Open Questions
- None.
