---
id: TASK-0312
type: task
title: "Verify and fix question and answer-type (open, single, multiple, case, cloze) import functionality across all supported formats"
status: backlog
epic: EPIC-072
parents: ['STORY-0249']
estimate: 3
labels: [m14, import, service, qa]
acceptance_criteria:
  - 'Audit every supported import format (Markdown, Obsidian, CSV, JSON, TOML, RON, ZIP, Anki, PDF) for how Question items and answer types (open, single-choice, multiple-choice, case-study, cloze) are parsed, mapped, and persisted'
  - 'Verify that front-matter fields (such as answer_type: open | single | multiple, choice option lists, reference answers, and extra metadata) survive import and round-trip export -> import without loss or coercion into plain cards'
  - 'Fix any gaps or data loss discovered during the audit (e.g. missing answer_type preservation in xfer_import.rs / xfer_export.rs / xfer_model.rs or CSV/Markdown/Obsidian adapters)'
  - 'Add end-to-end service integration tests covering importing questions across every answer type and format, verifying the resulting vault files parse and rehearse cleanly'
---

## Context
StudyFrog supports `Question` items with multiple answer types (`open`, `single` choice, `multiple` choice from `EPIC-072`, plus `case` study questions and `cloze` items) and imports from multiple formats (`xfer_markdown`, `xfer_obsidian`, `xfer_csv`, `xfer_json`, `xfer_toml`, `xfer_ron`, `xfer_zip`, `xfer_anki`, `xfer_pdf`).

An initial grep of `crates/studyfrog-service/src/xfer_*.rs` shows `answer_type` is not referenced in the transfer pipeline (`xfer_import.rs`, `xfer_export.rs`, `xfer_plan.rs`), meaning imported or round-tripped choice/special questions may silently lose their `answer_type` or fail to import cleanly.

## Description
Audit and verify the full import pipeline for `Question` items and all supported answer types:
1. **Format Audit (`crates/studyfrog-service/src/xfer_*.rs`)**:
   - Check how each importer constructs `ItemKind::Question` vs `ItemKind::Card` and whether `answer_type` (`open`, `single`, `multiple`), choice options (`- [x]` / `- [ ]` or structured choices), reference answers, and `extra` front-matter fields are preserved.
   - Check round-trip fidelity (`export -> import`) for JSON, TOML, RON, ZIP, Markdown, and CSV.
2. **Fix Gaps**:
   - Preserve `answer_type` and choice/case/cloze metadata through `xfer_model`, `xfer_export`, `xfer_import`, and format adapters (`xfer_csv`, `xfer_markdown`, `xfer_obsidian`, `xfer_json`, `xfer_toml`, `xfer_ron`, `xfer_anki`).
3. **Automated Verification**:
   - Add service-level integration tests verifying each question answer type imports into a real vault, parses back into the expected `Doc::Question` with intact `answer_type` and options, and loads cleanly via `load_item_content`.

## Acceptance Criteria
- [ ] Audit every supported import format (Markdown, Obsidian, CSV, JSON, TOML, RON, ZIP, Anki, PDF) for how Question items and answer types (`open`, `single`, `multiple`, `case`, `cloze`) are parsed, mapped, and persisted
- [ ] Verify that front-matter fields (such as `answer_type: open | single | multiple`, choice option lists, reference answers, and extra metadata) survive import and round-trip export -> import without loss or coercion into plain cards
- [ ] Fix any gaps or data loss discovered during the audit (e.g. missing `answer_type` preservation in `xfer_import.rs` / `xfer_export.rs` / `xfer_model.rs` or CSV/Markdown/Obsidian adapters)
- [ ] Add end-to-end service integration tests covering importing questions across every answer type and format, verifying the resulting vault files parse and rehearse cleanly

## Design Notes
- Key files to inspect:
  - `crates/studyfrog-service/src/xfer_model.rs`
  - `crates/studyfrog-service/src/xfer_import.rs`
  - `crates/studyfrog-service/src/xfer_export.rs`
  - `crates/studyfrog-service/src/xfer_plan.rs`
  - `crates/studyfrog-service/src/xfer_csv.rs`, `xfer_markdown.rs`, `xfer_obsidian.rs`, `xfer_json.rs`, `xfer_toml.rs`, `xfer_ron.rs`, `xfer_anki.rs`
- Follow Invariant P1 (zero inherent `impl` blocks) and Invariant P6 ($\le 250$ LOC per file).

## Dependencies
- Related: `EPIC-072` (Choice questions), `STORY-0065` (Import/export formats), `STORY-0168` (Universal import hub)

## Open Questions
None.
