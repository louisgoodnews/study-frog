---
id: TASK-0300
type: task
title: "Vault flashcard and question pair extraction and sampling in studyfrog-service"
status: backlog
epic: EPIC-075
parents: ['STORY-0244']
estimate: 2
labels: [m14, rehearse, service]
acceptance_criteria:
  - 'A pure service function reads Card front/back and Question prompt/answer pairs from the open vault filtered by scope (all vault, selected stack folders, due/weak filter)'
  - 'Markdown formatting is stripped into clean plain text and empty or duplicate pairs are skipped'
  - 'A deterministic sampler draws fresh N-pair rounds from the full pair pool using a seed'
---

## Context
Part of `STORY-0244`. Match mode needs real front/back and prompt/answer text from the vault instead of item titles or hardcoded biology cards.

## Description
Add a `match_pairs` module in `studyfrog-service` that loads cards and questions matching the requested scope, strips Markdown syntax into clean tile strings, deduplicates, and returns the pair pool for round sampling.

## Acceptance Criteria
- [ ] A pure service function reads Card front/back and Question prompt/answer pairs from the open vault filtered by scope (all vault, selected stack folders, due/weak filter)
- [ ] Markdown formatting is stripped into clean plain text and empty or duplicate pairs are skipped
- [ ] A deterministic sampler draws fresh N-pair rounds from the full pair pool using a seed

## Design Notes
- Reuse existing `studyfrog-store` item loading and `studyfrog-md` parsing helpers; keep files $\le 250$ LOC (P6).

## Dependencies
- `STORY-0244`

## Open Questions
None.
