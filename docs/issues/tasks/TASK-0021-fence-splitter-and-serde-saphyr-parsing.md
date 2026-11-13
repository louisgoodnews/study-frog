---
id: TASK-0021
type: task
title: "Fence splitter and serde-saphyr parsing"
status: done
epic: EPIC-005
parents: [STORY-0009]
estimate: 3
labels: [m2, md]
acceptance_criteria:
  - "`split_front_matter` handles `---` fences, CRLF, missing trailing newline and absent front-matter"
  - "Typed deserialisation and serialisation via serde-saphyr"
  - "Key order preserved on rewrite"
---

## Context
P10 applied: in-house splitter, no gray_matter (ADR 0005 amendment). Prototype: tools/repo-lint/src/issues.rs.

## Description
Fence splitter and serde-saphyr parsing.

## Acceptance Criteria
- [x] `split_front_matter` handles `---` fences, CRLF, missing trailing newline and absent front-matter
- [x] Typed deserialisation and serialisation via serde-saphyr
- [x] Key order preserved on rewrite

## Design Notes
**Spike result:** key order is no longer a serde-saphyr concern. Top-level keys are split into verbatim blocks (`fence::split_blocks`); only known keys go through serde-saphyr. Known keys are written in canonical order with defaults omitted; unknown keys (including keys valid only for another kind) are kept byte-for-byte after them in their original relative order. Comments attached to known keys are not preserved on rewrite. serde-saphyr quoting verified against 28 YAML-lookalike scalars plus 1000 random documents.

## Dependencies
TASK-0020.

## Open Questions
None (answered in Design Notes).
