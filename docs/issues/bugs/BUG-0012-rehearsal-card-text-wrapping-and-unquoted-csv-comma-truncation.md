---
id: BUG-0012
type: bug
title: Rehearsal card text wrapping and unquoted CSV comma truncation
status: done
epic: EPIC-031
parents: []
estimate: 2
labels: [m11, ext, service, ui]
acceptance_criteria:
  - "Rehearsal card prompts and answers wrap cleanly within the card container and scroll vertically when taller than the viewport"
  - "CSV rows with unquoted commas in front or back fields (such as LLM-generated 6-column stack,kind,title,front,back,tags rows) reconstruct the full front and back text instead of silently truncating at the first comma"
---

## Context
During rehearsal of cards imported from an LLM-generated CSV (`gemini-code-1793864286356.txt`), long questions and answers appeared cut off mid-sentence (for example, stopping after `...für die Zufallsvariablen X` or `...fast nie nur von einer einzigen`). In addition, the rehearsal card container did not wrap its body in a vertical `ScrollArea` or explicitly enforce `TextWrapMode::Wrap` across markdown spans.

## Description
Root cause:
1. When LLMs generate `stack,kind,title,front,back,tags` CSV files without quoting fields that contain commas (e.g. `...für die Zufallsvariablen X, Y und Z?,E(Y|X, Z) = ...` or `...von einer einzigen, sondern von mehreren...,Multiple_Regression;Grundbegriffe`), splitting strictly on `,` produces more than 6 columns. Both `examples/studyfrog-ext-example/src/delimited.rs` and `crates/studyfrog-service/src/xfer_csv_map.rs` previously indexed `fields[front]` and `fields[back]` directly and ignored the extra middle columns, silently truncating `front` and `back` at the first unquoted comma.
2. In `crates/studyfrog-ui/src/view/run.rs`, `run_card.rs`, and `preview.rs`, the rehearsal card view lacked a vertical `ScrollArea` and explicit `TextWrapMode::Wrap` on the markdown scope.

## Acceptance Criteria
- [x] Rehearsal card prompts and answers wrap cleanly within the card container and scroll vertically when taller than the viewport
- [x] CSV rows with unquoted commas in front or back fields (such as LLM-generated 6-column stack,kind,title,front,back,tags rows) reconstruct the full front and back text instead of silently truncating at the first comma

## Design Notes
- In `delimited.rs` and `xfer_csv_map.rs`, when a comma-delimited row has more fields than the header (or standard 6-column layout), normalize the row by anchoring the leading metadata columns (`stack`, `kind`, `title`) and trailing `tags` column, then splitting the remaining middle fields between `front` and `back` (splitting at the question mark `?` boundary when present, or merging extra comma-separated clauses into `back`).
- In `run.rs` and `run_card.rs`, wrap the rehearsal card area in a vertical `ScrollArea` and enforce `TextWrapMode::Wrap`.

## Dependencies
None.

## Open Questions
None.
