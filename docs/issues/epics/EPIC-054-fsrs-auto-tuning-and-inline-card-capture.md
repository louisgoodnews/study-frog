---
id: EPIC-054
type: epic
title: "FSRS Auto-Tuning and Inline Card Capture"
status: backlog
parents: []
labels: [m13, srs, md]
acceptance_criteria:
  - 'Adam gradient descent optimizer personalizes 21 FSRS-6 parameters on local review history'
  - 'Log-loss minimization and L2 regularization prevent overfitting on small review sets'
  - 'Inline card syntax scanner extracts Term :: Definition pairs from markdown documents'
  - 'Virtual item indexing syncs inline cards into SQLite without creating separate files'
---

## Context
Default FSRS parameters assume average human memory. Power users benefit from personalized model weights, and taking notes in markdown should naturally yield cards without modals.

## Description
Provide a local Adam optimizer for FSRS-6 parameter training on SQLite review history and an inline Term :: Definition syntax scanner in markdown notes.

## Acceptance Criteria
- [ ] Adam gradient descent optimizer personalizes 21 FSRS-6 parameters on local review history
- [ ] Log-loss minimization and L2 regularization prevent overfitting on small review sets
- [ ] Inline card syntax scanner extracts Term :: Definition pairs from markdown documents
- [ ] Virtual item indexing syncs inline cards into SQLite without creating separate files

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
None.

## Open Questions
None.
