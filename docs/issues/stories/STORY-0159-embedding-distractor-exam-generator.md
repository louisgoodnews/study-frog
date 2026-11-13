---
id: STORY-0159
type: story
title: "Embedding-Powered Distractor Generator for Multiple-Choice Exams"
status: backlog
epic: EPIC-053
parents: ['EPIC-053']
estimate: 5
labels: [m13, rehearsal]
acceptance_criteria:
  - 'Semantic distractor extractor finds nearest-neighbor answers using MiniLM embeddings'
  - 'Multiple-choice question builder generates 4 options (1 correct, 3 plausible distractors)'
  - 'Exam simulator integrates multiple-choice sections with automated scoring'
---

## Context
Authentic multiple-choice exam preparation with semantically challenging distractors.

## Description
Generate challenging multiple-choice tests by finding semantically related distractor answers across the active stack using MiniLM vector embeddings.

## Acceptance Criteria
- [ ] Semantic distractor extractor finds nearest-neighbor answers using MiniLM embeddings
- [ ] Multiple-choice question builder generates 4 options (1 correct, 3 plausible distractors)
- [ ] Exam simulator integrates multiple-choice sections with automated scoring

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-053

## Open Questions
None.
