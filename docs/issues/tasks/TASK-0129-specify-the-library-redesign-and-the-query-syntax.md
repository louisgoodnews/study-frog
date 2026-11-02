---
id: TASK-0129
type: task
title: Specify the library redesign and the query syntax
status: review
epic: EPIC-023
parents: [STORY-0057]
estimate: 3
labels: [m10, docs, decision]
acceptance_criteria:
  - "docs/specs/library.md describes the list layout, the columns, the bulk actions and the main user tasks with the steps each takes today and after"
  - "docs/specs/query.md gives the grammar, the fields, the operators and the error messages with examples"
  - "The owner has accepted the layout (a copy of Jira's list view) and the query syntax"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Write the redesign and the query language down before building them.

## Acceptance Criteria
- [x] docs/specs/library.md describes the list layout, the columns, the bulk actions and the main user tasks with the steps each takes today and after
- [x] docs/specs/query.md gives the grammar, the fields, the operators and the error messages with examples
- [ ] The owner has accepted the layout (a copy of Jira's list view) and the query syntax

## Design Notes
Written 2026-11-02 in `docs/specs/library.md` and `docs/specs/query.md`. The layout (a copy of Jira's list view) and the idea of a Standard and a Query mode were accepted by the owner; the grammar, the field table, the error messages, the column defaults and the place of saved filters (`filters.toml` in the vault, not per profile) are proposals awaiting the owner.

## Dependencies
None.

## Open Questions
Does the owner accept the query syntax and the column defaults? Are saved filters per vault (proposed) or per profile?
