# Issues

Work items are Markdown files with YAML front-matter, versioned with the code.

## Hierarchy and naming
`epic → story → task → subtask`, plus standalone `bug`.

| Type | Directory | File name | ID |
|---|---|---|---|
| epic | `epics/` | `EPIC-001-workspace-and-guardrails.md` | `EPIC-001` |
| story | `stories/` | `STORY-0001-workspace-skeleton.md` | `STORY-0001` |
| task | `tasks/` | `TASK-0001-convert-to-workspace.md` | `TASK-0001` |
| subtask | `subtasks/` | `SUB-0001-....md` | `SUB-0001` |
| bug | `bugs/` | `BUG-0001-....md` | `BUG-0001` |

IDs are zero-padded (epics 3 digits, others 4), assigned sequentially, never reused.
The file name starts with the ID followed by a kebab-case slug of the title.

## Front-matter (mandatory)
```yaml
---
id: STORY-0042
type: story            # epic|story|task|subtask|bug
title: ...
status: backlog        # backlog|ready|in_progress|review|done|blocked
epic: EPIC-003
parents: [TASK-0017]   # direct parent ids (story->epic, task->story, subtask->task)
estimate: 3            # story points
labels: [rehearsal, latency]
acceptance_criteria:
  - ...
---
```
Body sections, in order: **Context**, **Description**, **Acceptance Criteria**,
**Design Notes**, **Dependencies**, **Open Questions**.

## Labels
Area: one per crate (`core`, `md`, `store`, `srs`, `rehearsal`, `slm`, `report`, `achieve`,
`board`, `calendar`, `ui`, `app`), plus `docs`, `tooling`, `perf`, `decision`.
Phase: `m0` to `m8`.

## Workflow
`backlog → ready → in_progress → review → done`; `blocked` may be entered from any state
and must name the blocker under Dependencies.

- Only items of the current phase may be `ready`.
- A story is `done` when all its tasks are `done` and its acceptance criteria hold.
- Estimates use points 1, 2, 3, 5, 8; anything larger is split.
- Commits reference items in the footer (`Refs: TASK-0004`), see
  [COMMIT_CONVENTIONS](../contributing/COMMIT_CONVENTIONS.md).
- Phase scope and the full planned tree live in [ROADMAP](../roadmap/ROADMAP.md).
