---
id: STORY-0208
type: story
title: "Classroom session domain model and local storage persistence"
status: done
epic: EPIC-068
parents: ['EPIC-068']
estimate: 5
labels: [m14, core, store]
acceptance_criteria:
  - 'Pure data types and free transition functions in studyfrog-core model classroom sessions, participants (teacher, simulated students, user), chat turns, solo/group assignments, submissions, and grades'
  - 'SQLite migration and repository functions in studyfrog-store persist and reload full classroom sessions without data loss across restarts'
  - 'Sessions can be listed, resumed, deleted, and exported as readable Markdown transcripts'
---

## Context
Classroom simulation sessions (`EPIC-068`) can span multiple lecture segments, Q&A exchanges, and graded assignments. In line with Invariant P8 (local-first data sovereignty), all session state must persist reliably in local storage so users can close the app and resume or review any session later.

## Description
Define the domain model in `studyfrog-core` and local persistence in `studyfrog-store`:
- Session metadata (id, title, topic, source stack/note/PDF references, provider name, created/updated timestamps, status).
- Participants (teacher persona, simulated student personas, and the user) with group memberships.
- Ordered transcript turns (speaker, role, content, timestamp, optional link to a task or group thread).
- Assignments (solo or group), student/user submissions, and teacher grades with structured feedback.

## Acceptance Criteria
- [x] Pure data types and free transition functions in `studyfrog-core` model classroom sessions, participants (teacher, simulated students, user), chat turns, solo/group assignments, submissions, and grades
- [x] SQLite migration and repository functions in `studyfrog-store` persist and reload full classroom sessions without data loss across restarts
- [x] Sessions can be listed, resumed, deleted, and exported as readable Markdown transcripts

## Design Notes
- Follow Invariant P1 (no inherent `impl` blocks; pure free functions) and Invariant P6 ($\le 250$ LOC per file).
- Use ULIDs for session, turn, assignment, and submission IDs consistent with existing `studyfrog-core` types.

## Dependencies
- `EPIC-068`
- Child tasks: `TASK-0287`, `TASK-0288`

## Open Questions
- Whether to also mirror finished sessions as Markdown notes in the vault automatically or on explicit export.
