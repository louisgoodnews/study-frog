---
id: BUG-0018
type: bug
title: Code block indentations not visible in markdown viewer
status: done
estimate: 2
labels: [m12, ui, markdown]
acceptance_criteria:
  - "Indented code blocks in markdown preview preserve all leading spaces and tab stops accurately"
  - "Code snippets with Python, Rust, C, or generic indented code maintain structural indentation without collapsing"
---

## Context
When viewing code blocks in the Markdown preview (`crates/studyfrog-ui/src/view/preview_rich.rs::code`), code indentations (such as Python block indents or multi-space/tab indented code lines) are collapsed or not visible. This occurs because tab characters (`\t`) in egui's default font have zero advance width, and leading whitespace can be compressed during wrapped label layout jobs.

## Description
Preserve code block indentation in `crates/studyfrog-ui/src/view/preview_rich.rs`:
1. Expand tab characters (`\t`) into standardized 4-space indentations before computing character positions and token slicing in `code(...)`.
2. Ensure `egui::Label::new(job)` or monospace frame formatting preserves leading whitespace and does not trim or collapse indentation at the beginning of lines.
3. Add automated test coverage verifying that indented code lines retain their full indentation width in rendered output.

## Acceptance Criteria
- [x] Indented code blocks in markdown preview preserve all leading spaces and tab stops accurately
- [x] Code snippets with Python, Rust, C, or generic indented code maintain structural indentation without collapsing

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/preview_rich.rs`, `crates/studyfrog-ui/tests/md_rich_ui.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- None.

## Open Questions
- None.
