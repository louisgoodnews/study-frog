---
id: TASK-0313
type: task
title: "Remove narrow reading-measure wrap cap in preview, Rehearse cards, and editors so text uses full available width"
status: backlog
epic: EPIC-073
parents: ['STORY-0250']
estimate: 2
labels: [m14, ui]
acceptance_criteria:
  - 'crates/studyfrog-ui/src/view/preview.rs wraps rendered Markdown text to ui.available_width() instead of clamping to typo::measure(ui)'
  - 'Rehearse prompt and answer blocks, Note Body editors (md_editor/mod.rs), and centered columns expand text wrapping cleanly across the full card/container width'
  - 'Headless UI tests verify text fills the card width at 1280x800 and 960x640 without premature half-width wrapping'
---

## Context
Part of `STORY-0250`. Currently `crates/studyfrog-ui/src/view/preview.rs:162` clamps Markdown text wrapping to `typo::measure(ui) * ...` (around ~500–600 px) inside full-width Rehearse cards, causing text to wrap at roughly half the card width.

## Description
Update text wrapping across `preview.rs`, `md_editor/mod.rs`, and related card/prose views so text wraps to the full available width of its container, and update/add headless UI tests verifying the new full-width wrapping behavior.

## Acceptance Criteria
- [ ] `crates/studyfrog-ui/src/view/preview.rs` wraps rendered Markdown text to `ui.available_width()` instead of clamping to `typo::measure(ui)`
- [ ] Rehearse prompt and answer blocks, Note Body editors (`md_editor/mod.rs`), and centered columns expand text wrapping cleanly across the full card/container width
- [ ] Headless UI tests verify text fills the card width at 1280x800 and 960x640 without premature half-width wrapping

## Design Notes
- Affected files:
  - `crates/studyfrog-ui/src/view/preview.rs`
  - `crates/studyfrog-ui/src/view/md_editor/mod.rs`
  - `crates/studyfrog-ui/src/view/kit.rs` (check `reading_column` callers)
  - `crates/studyfrog-ui/tests/reading_measure_ui.rs` & `editor_measure_ui.rs` (update assertions if they enforce the old narrow measure)

## Dependencies
- `STORY-0250`

## Open Questions
None.
