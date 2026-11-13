---
id: TASK-0052
type: task
title: "Score functions"
status: done
epic: EPIC-013
parents: [STORY-0024]
estimate: 3
labels: [m6, slm]
acceptance_criteria:
  - "Text normalised (NFC, trimmed, whitespace collapsed) before embedding"
  - "Best cosine over reference and accepted answers; rubric points blended 0.6/0.4"
  - "Verdict bands Correct / Partial (within 0.2) / Incorrect; verdict suggests a rating"
  - "An in-house lexical runtime (hashed words, word pairs, character trigrams) works offline with no download"
---

## Context
Scoring contract v1.

## Description
Score functions.

## Acceptance Criteria
- [x] Text normalised (NFC, trimmed, whitespace collapsed) before embedding
- [x] Best cosine over reference and accepted answers; rubric points blended 0.6/0.4
- [x] Verdict bands Correct / Partial (within 0.2) / Incorrect; verdict suggests a rating
- [x] An in-house lexical runtime (hashed words, word pairs, character trigrams) works offline with no download

## Design Notes
Build vs buy (P10): the lexical runtime is about 100 lines and needs no model; it measures shared wording, not meaning, and says so in its docs. `unicode-normalization` is kept for NFC (tables are a hard general problem).

## Dependencies
TASK-0049.

## Open Questions
None.
