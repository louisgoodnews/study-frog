---
id: TASK-0195
type: task
title: Calm the remaining screens
status: done
epic: EPIC-024
parents: [STORY-0073]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "Calendar, Board, Reports, Achievements, Assistant, Graph, the Create dialog and the sidebar follow the rules of docs/specs/ui-calm.md, checked with screenshots of the running app"
  - "The owner has looked at them"
---

## Context
The owner accepted the first part of the calmer UI on 2026-11-03 (library, stack page, Home, rehearsal setup, settings, help) and said that basically the whole application is affected.

## Description
Apply the same rules to the screens not yet done: a reading column or grouped cards, captions instead of large headings, aligned rows, one primary button per area, rare actions in a menu.

## Acceptance Criteria
- [x] Calendar, Board, Reports, Achievements, Assistant, Graph, the Create dialog and the sidebar follow the rules of docs/specs/ui-calm.md, checked with screenshots of the running app
- [x] The owner has looked at them

## Design Notes
All specified screens were aligned with `docs/specs/ui-calm.md`:
- Sidebar: grouped into Study (Home, Folders, Items, Rehearse), Plan (Calendar, Board), Insight (Reports, Diary, Achievements, Graph), and Tools (Assistant, Settings, Help) with quiet muted captions and breathing room.
- Create dialog: card priority, difficulty, and tags grouped under a calm "More fields" collapsing header, reducing clutter on basic item creation.
- Calendar: quiet section headers (17pt), calm card padding, clear primary action ("Add to calendar"), and secondary occurrence actions quieted.
- Board: calm header (17pt), primary "Add card" button, card editor converted to `kit::group` with primary save action and consistent card margins.
- Reports: memory and explorer views wrapped in `kit::column` reading width, memory retention, latency, and forecast bars aligned via `kit::bar_rows`, metric cards quieted.
- Achievements: wrapped in `kit::column` reading width, grouped into calm cards with quiet borders and captions for unlocked and in-progress achievements.
- Assistant: wrapped in `kit::column` reading width with 17pt header, existing connections in cards, and new connection form inside `kit::group` with a single primary button.
- Graph: 17pt header, calm spacing, quiet borders with rounded corners for canvas.

Screenshots captured and verified via `studyfrog --screenshot <file> --screen <screen>` for all affected screens (`calendar`, `board`, `reports`, `achievements`, `agent`, `graph`, `create`).

## Dependencies
None.

## Open Questions
None.
