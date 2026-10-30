# M5: Calendar and board

**Goal:** Planning and task management.

**Entry criteria:** M4 exit met.

## Epics, stories, tasks

- **EPIC-010** Calendar and planning
  - **STORY-0020** Session planning
    - TASK-0044 Session model and recurrence
    - TASK-0078 Local time zone and local-day windows
    - TASK-0079 Persist sessions
    - TASK-0045 Calendar view
- **EPIC-011** Scrum/kanban board
  - **STORY-0021** Board model
    - TASK-0046 Columns and WIP limits
    - TASK-0047 Move and push semantics
    - TASK-0080 Persist the board
  - **STORY-0022** Board UI
    - TASK-0048 Board view with drag and drop

## Exit criteria

- [x] Sessions can be scheduled, linked to stacks, and recur
- [x] Board enforces WIP limits (property-tested)
- [x] Drag and push semantics work in the UI
- [x] Calendar and board persist across restarts

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
