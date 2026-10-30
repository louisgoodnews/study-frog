---
id: TASK-0021
type: task
title: Fence splitter and serde-saphyr parsing
status: ready
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
- [ ] `split_front_matter` handles `---` fences, CRLF, missing trailing newline and absent front-matter
- [ ] Typed deserialisation and serialisation via serde-saphyr
- [ ] Key order preserved on rewrite

## Design Notes
Property test: random bodies never confuse the splitter.

## Dependencies
TASK-0020.

## Open Questions
Does serde-saphyr preserve key order and comments on re-serialisation? Spike first.
