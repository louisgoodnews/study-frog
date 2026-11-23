---
id: TASK-0288
type: task
title: "SQLite schema migration and local classroom session store in studyfrog-store"
status: backlog
epic: EPIC-068
parents: ['STORY-0208']
estimate: 3
labels: [m14, store]
acceptance_criteria:
  - 'A new SQLite migration in studyfrog-store creates normalized tables for classroom sessions, participants, chat turns, assignments, group memberships, submissions, and grades'
  - 'Repository free functions save, load, list, update, and delete full classroom sessions atomically, plus export a session to Markdown'
  - 'Integration tests verify round-trip persistence across store reopen and cascading deletes'
---

## Context
Part of `STORY-0208`. Classroom sessions must persist locally across app restarts so students can resume lectures, finish assignments, and review past feedback.

## Description
Add the next sequential SQLite migration and `classroom` persistence module in `studyfrog-store`:
- Store session headers, source references, participant rosters, ordered chat turns, solo/group task assignments, group members, submissions, and grades.
- Provide pure Markdown export rendering a session's transcript, assignments, submissions, and grades.

## Acceptance Criteria
- [ ] A new SQLite migration in `studyfrog-store` creates normalized tables for classroom sessions, participants, chat turns, assignments, group memberships, submissions, and grades
- [ ] Repository free functions save, load, list, update, and delete full classroom sessions atomically, plus export a session to Markdown
- [ ] Integration tests verify round-trip persistence across store reopen and cascading deletes

## Design Notes
- Use parameterized SQL queries and transactions so partial writes never corrupt a session.
- Keep migration idempotent and compatible with existing vault databases.

## Dependencies
- `STORY-0208`, `TASK-0287`

## Open Questions
None.
